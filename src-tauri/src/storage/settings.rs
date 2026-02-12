use rusqlite::params;
use std::collections::HashMap;

use super::database::Database;

/// Default settings seeded on first run
const DEFAULTS: &[(&str, &str)] = &[
	("search_engine", "https://www.google.com/search?q="),
	("homepage", "https://www.google.com"),
	("new_tab_page", "https://www.google.com"),
	("restore_on_startup", "new_tab"),
	("theme", "dark"),
	("show_bookmarks_bar", "true"),
	("show_status_bar", "true"),
	("default_zoom", "100"),
	("download_path", "~/Downloads"),
	("ask_download_location", "false"),
];

impl Database {
	/// Seed default settings (only inserts if key doesn't already exist)
	pub fn seed_settings(&self) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn
			.prepare("INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)")
			.map_err(|e| e.to_string())?;

		for (key, value) in DEFAULTS {
			stmt.execute(params![key, value])
				.map_err(|e| e.to_string())?;
		}
		Ok(())
	}

	/// Get a single setting by key
	pub fn settings_get(&self, key: &str) -> Result<Option<String>, String> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn
			.prepare("SELECT value FROM settings WHERE key = ?1")
			.map_err(|e| e.to_string())?;

		let result = stmt
			.query_row(params![key], |row| row.get(0))
			.map(Some)
			.unwrap_or(None);

		Ok(result)
	}

	/// Set a single setting
	pub fn settings_set(&self, key: &str, value: &str) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();
		conn.execute(
			"INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
			params![key, value],
		)
		.map_err(|e| e.to_string())?;
		Ok(())
	}

	/// Get all settings as a HashMap
	pub fn settings_get_all(&self) -> Result<HashMap<String, String>, String> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn
			.prepare("SELECT key, value FROM settings")
			.map_err(|e| e.to_string())?;

		let rows = stmt
			.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
			.map_err(|e| e.to_string())?;

		let mut map = HashMap::new();
		for row in rows {
			let (key, value) = row.map_err(|e| e.to_string())?;
			map.insert(key, value);
		}
		Ok(map)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn test_db() -> Database {
		let db = Database::open_in_memory().unwrap();
		db.seed_settings().unwrap();
		db
	}

	#[test]
	fn seed_settings_populates_defaults() {
		let db = test_db();
		let all = db.settings_get_all().unwrap();
		assert!(all.len() >= DEFAULTS.len());
		assert_eq!(all.get("theme").unwrap(), "dark");
		assert_eq!(
			all.get("search_engine").unwrap(),
			"https://www.google.com/search?q="
		);
	}

	#[test]
	fn seed_settings_is_idempotent() {
		let db = test_db();
		// Change a setting, then re-seed — should not overwrite
		db.settings_set("theme", "light").unwrap();
		db.seed_settings().unwrap();
		assert_eq!(db.settings_get("theme").unwrap().unwrap(), "light");
	}

	#[test]
	fn get_returns_none_for_missing_key() {
		let db = test_db();
		assert!(db.settings_get("nonexistent").unwrap().is_none());
	}

	#[test]
	fn set_and_get_round_trip() {
		let db = test_db();
		db.settings_set("custom_key", "custom_value").unwrap();
		assert_eq!(
			db.settings_get("custom_key").unwrap().unwrap(),
			"custom_value"
		);
	}

	#[test]
	fn set_overwrites_existing() {
		let db = test_db();
		db.settings_set("theme", "light").unwrap();
		assert_eq!(db.settings_get("theme").unwrap().unwrap(), "light");
		db.settings_set("theme", "system").unwrap();
		assert_eq!(db.settings_get("theme").unwrap().unwrap(), "system");
	}

	#[test]
	fn get_all_returns_map() {
		let db = test_db();
		let all = db.settings_get_all().unwrap();
		assert!(all.contains_key("homepage"));
		assert!(all.contains_key("default_zoom"));
	}
}
