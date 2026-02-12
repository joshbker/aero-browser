use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;

/// Current schema version — bump this when adding migrations
const SCHEMA_VERSION: u32 = 1;

/// Thread-safe wrapper around a SQLite connection
pub struct Database {
	pub conn: Mutex<Connection>,
}

impl Database {
	/// Open (or create) the database file and run migrations
	pub fn open(path: &str) -> SqlResult<Self> {
		let conn = Connection::open(path)?;

		// Enable WAL mode for better concurrent read performance
		conn.pragma_update(None, "journal_mode", "WAL")?;
		// Enable foreign keys
		conn.pragma_update(None, "foreign_keys", "ON")?;

		let db = Self {
			conn: Mutex::new(conn),
		};
		db.migrate()?;
		Ok(db)
	}

	/// Open an in-memory database (for tests)
	#[cfg(test)]
	pub fn open_in_memory() -> SqlResult<Self> {
		let conn = Connection::open_in_memory()?;
		conn.pragma_update(None, "foreign_keys", "ON")?;
		let db = Self {
			conn: Mutex::new(conn),
		};
		db.migrate()?;
		Ok(db)
	}

	/// Run schema migrations using PRAGMA user_version
	fn migrate(&self) -> SqlResult<()> {
		let conn = self.conn.lock().unwrap();
		let current_version: u32 =
			conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

		if current_version < 1 {
			self.apply_v1(&conn)?;
		}

		// Future migrations go here:
		// if current_version < 2 { self.apply_v2(&conn)?; }

		conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
		Ok(())
	}

	/// V1: Initial schema — all Phase 2 tables
	fn apply_v1(&self, conn: &Connection) -> SqlResult<()> {
		conn.execute_batch(
			"
			-- Settings (key-value store)
			CREATE TABLE IF NOT EXISTS settings (
				key TEXT PRIMARY KEY,
				value TEXT NOT NULL
			);

			-- History
			CREATE TABLE IF NOT EXISTS history (
				id TEXT PRIMARY KEY,
				url TEXT NOT NULL,
				title TEXT,
				visit_count INTEGER DEFAULT 1,
				last_visited DATETIME DEFAULT CURRENT_TIMESTAMP,
				first_visited DATETIME DEFAULT CURRENT_TIMESTAMP
			);
			CREATE INDEX IF NOT EXISTS idx_history_url ON history(url);
			CREATE INDEX IF NOT EXISTS idx_history_last_visited ON history(last_visited DESC);

			-- Bookmarks
			CREATE TABLE IF NOT EXISTS bookmarks (
				id TEXT PRIMARY KEY,
				parent_id TEXT REFERENCES bookmarks(id),
				title TEXT NOT NULL,
				url TEXT,
				is_folder BOOLEAN DEFAULT FALSE,
				position INTEGER NOT NULL,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
			);

			-- Permissions (per-origin)
			CREATE TABLE IF NOT EXISTS permissions (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				origin TEXT NOT NULL,
				permission_type TEXT NOT NULL,
				state TEXT NOT NULL DEFAULT 'prompt',
				updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				UNIQUE(origin, permission_type)
			);

			-- Autofill profiles
			CREATE TABLE IF NOT EXISTS autofill_profiles (
				id TEXT PRIMARY KEY,
				profile_type TEXT NOT NULL,
				data TEXT NOT NULL,
				created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
				updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
			);
			",
		)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn open_in_memory_succeeds() {
		let db = Database::open_in_memory().unwrap();
		let conn = db.conn.lock().unwrap();
		let version: u32 = conn
			.pragma_query_value(None, "user_version", |row| row.get(0))
			.unwrap();
		assert_eq!(version, SCHEMA_VERSION);
	}

	#[test]
	fn tables_exist_after_migration() {
		let db = Database::open_in_memory().unwrap();
		let conn = db.conn.lock().unwrap();

		let tables: Vec<String> = conn
			.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
			.unwrap()
			.query_map([], |row| row.get(0))
			.unwrap()
			.filter_map(|r| r.ok())
			.collect();

		assert!(tables.contains(&"settings".to_string()));
		assert!(tables.contains(&"history".to_string()));
		assert!(tables.contains(&"bookmarks".to_string()));
		assert!(tables.contains(&"permissions".to_string()));
		assert!(tables.contains(&"autofill_profiles".to_string()));
	}

	#[test]
	fn migration_is_idempotent() {
		let db = Database::open_in_memory().unwrap();
		// Run migrate again — should not error
		db.migrate().unwrap();

		let conn = db.conn.lock().unwrap();
		let version: u32 = conn
			.pragma_query_value(None, "user_version", |row| row.get(0))
			.unwrap();
		assert_eq!(version, SCHEMA_VERSION);
	}

	#[test]
	fn can_insert_and_read_settings() {
		let db = Database::open_in_memory().unwrap();
		let conn = db.conn.lock().unwrap();

		conn.execute(
			"INSERT INTO settings (key, value) VALUES (?1, ?2)",
			["theme", "dark"],
		)
		.unwrap();

		let value: String = conn
			.query_row("SELECT value FROM settings WHERE key = ?1", ["theme"], |row| {
				row.get(0)
			})
			.unwrap();
		assert_eq!(value, "dark");
	}

	#[test]
	fn foreign_keys_enforced_on_bookmarks() {
		let db = Database::open_in_memory().unwrap();
		let conn = db.conn.lock().unwrap();

		// Insert with a non-existent parent_id should fail
		let result = conn.execute(
			"INSERT INTO bookmarks (id, parent_id, title, position) VALUES ('b1', 'nonexistent', 'Test', 0)",
			[],
		);
		assert!(result.is_err());
	}

	#[test]
	fn history_indexes_exist() {
		let db = Database::open_in_memory().unwrap();
		let conn = db.conn.lock().unwrap();

		let indexes: Vec<String> = conn
			.prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='history'")
			.unwrap()
			.query_map([], |row| row.get(0))
			.unwrap()
			.filter_map(|r| r.ok())
			.collect();

		assert!(indexes.contains(&"idx_history_url".to_string()));
		assert!(indexes.contains(&"idx_history_last_visited".to_string()));
	}
}
