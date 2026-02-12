use rusqlite::params;
use serde::Serialize;
use uuid::Uuid;

use super::database::Database;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
	pub id: String,
	pub url: String,
	pub title: Option<String>,
	pub visit_count: i64,
	pub last_visited: String,
	pub first_visited: String,
}

impl Database {
	/// Record a page visit — upserts by URL (increments visit_count if exists)
	pub fn history_add_visit(&self, url: &str, title: Option<&str>) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();

		// Try to update existing entry first
		let updated = conn
			.execute(
				"UPDATE history SET visit_count = visit_count + 1, last_visited = CURRENT_TIMESTAMP, title = COALESCE(?2, title) WHERE url = ?1",
				params![url, title],
			)
			.map_err(|e| e.to_string())?;

		if updated == 0 {
			// Insert new entry
			let id = Uuid::new_v4().to_string();
			conn.execute(
				"INSERT INTO history (id, url, title) VALUES (?1, ?2, ?3)",
				params![id, url, title],
			)
			.map_err(|e| e.to_string())?;
		}

		Ok(())
	}

	/// Search history by URL or title substring
	pub fn history_search(&self, query: &str, limit: i64) -> Result<Vec<HistoryEntry>, String> {
		let conn = self.conn.lock().unwrap();
		let pattern = format!("%{}%", query);

		let mut stmt = conn
			.prepare(
				"SELECT id, url, title, visit_count, last_visited, first_visited
				 FROM history
				 WHERE url LIKE ?1 OR title LIKE ?1
				 ORDER BY last_visited DESC
				 LIMIT ?2",
			)
			.map_err(|e| e.to_string())?;

		let entries = stmt
			.query_map(params![pattern, limit], |row| {
				Ok(HistoryEntry {
					id: row.get(0)?,
					url: row.get(1)?,
					title: row.get(2)?,
					visit_count: row.get(3)?,
					last_visited: row.get(4)?,
					first_visited: row.get(5)?,
				})
			})
			.map_err(|e| e.to_string())?
			.filter_map(|r| r.ok())
			.collect();

		Ok(entries)
	}

	/// Get recent history entries
	pub fn history_get_recent(&self, limit: i64) -> Result<Vec<HistoryEntry>, String> {
		let conn = self.conn.lock().unwrap();

		let mut stmt = conn
			.prepare(
				"SELECT id, url, title, visit_count, last_visited, first_visited
				 FROM history
				 ORDER BY last_visited DESC
				 LIMIT ?1",
			)
			.map_err(|e| e.to_string())?;

		let entries = stmt
			.query_map(params![limit], |row| {
				Ok(HistoryEntry {
					id: row.get(0)?,
					url: row.get(1)?,
					title: row.get(2)?,
					visit_count: row.get(3)?,
					last_visited: row.get(4)?,
					first_visited: row.get(5)?,
				})
			})
			.map_err(|e| e.to_string())?
			.filter_map(|r| r.ok())
			.collect();

		Ok(entries)
	}

	/// Delete a single history entry by ID
	pub fn history_delete(&self, id: &str) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();
		conn.execute("DELETE FROM history WHERE id = ?1", params![id])
			.map_err(|e| e.to_string())?;
		Ok(())
	}

	/// Clear history by timeframe
	/// timeframe: "hour", "day", "week", "all"
	pub fn history_clear(&self, timeframe: &str) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();

		let sql = match timeframe {
			"hour" => "DELETE FROM history WHERE last_visited >= datetime('now', '-1 hour')",
			"day" => "DELETE FROM history WHERE last_visited >= datetime('now', '-1 day')",
			"week" => "DELETE FROM history WHERE last_visited >= datetime('now', '-7 days')",
			"all" => "DELETE FROM history",
			_ => return Err(format!("Invalid timeframe: {}", timeframe)),
		};

		conn.execute(sql, []).map_err(|e| e.to_string())?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn test_db() -> Database {
		Database::open_in_memory().unwrap()
	}

	#[test]
	fn add_visit_creates_entry() {
		let db = test_db();
		db.history_add_visit("https://example.com", Some("Example"))
			.unwrap();

		let entries = db.history_get_recent(10).unwrap();
		assert_eq!(entries.len(), 1);
		assert_eq!(entries[0].url, "https://example.com");
		assert_eq!(entries[0].title.as_deref(), Some("Example"));
		assert_eq!(entries[0].visit_count, 1);
	}

	#[test]
	fn add_visit_increments_count_on_revisit() {
		let db = test_db();
		db.history_add_visit("https://example.com", Some("Example"))
			.unwrap();
		db.history_add_visit("https://example.com", Some("Example - Updated"))
			.unwrap();

		let entries = db.history_get_recent(10).unwrap();
		assert_eq!(entries.len(), 1);
		assert_eq!(entries[0].visit_count, 2);
		assert_eq!(entries[0].title.as_deref(), Some("Example - Updated"));
	}

	#[test]
	fn search_finds_by_url() {
		let db = test_db();
		db.history_add_visit("https://example.com", Some("Example"))
			.unwrap();
		db.history_add_visit("https://other.com", Some("Other"))
			.unwrap();

		let results = db.history_search("example", 10).unwrap();
		assert_eq!(results.len(), 1);
		assert_eq!(results[0].url, "https://example.com");
	}

	#[test]
	fn search_finds_by_title() {
		let db = test_db();
		db.history_add_visit("https://example.com", Some("My Favourite Page"))
			.unwrap();

		let results = db.history_search("Favourite", 10).unwrap();
		assert_eq!(results.len(), 1);
	}

	#[test]
	fn search_respects_limit() {
		let db = test_db();
		for i in 0..5 {
			db.history_add_visit(&format!("https://site{}.com", i), None)
				.unwrap();
		}

		let results = db.history_search("site", 3).unwrap();
		assert_eq!(results.len(), 3);
	}

	#[test]
	fn get_recent_returns_correct_count() {
		let db = test_db();
		db.history_add_visit("https://first.com", None).unwrap();
		db.history_add_visit("https://second.com", None).unwrap();
		db.history_add_visit("https://third.com", None).unwrap();

		let entries = db.history_get_recent(10).unwrap();
		assert_eq!(entries.len(), 3);

		// With limit
		let limited = db.history_get_recent(2).unwrap();
		assert_eq!(limited.len(), 2);
	}

	#[test]
	fn delete_removes_entry() {
		let db = test_db();
		db.history_add_visit("https://example.com", None).unwrap();

		let entries = db.history_get_recent(10).unwrap();
		let id = entries[0].id.clone();

		db.history_delete(&id).unwrap();

		let entries = db.history_get_recent(10).unwrap();
		assert!(entries.is_empty());
	}

	#[test]
	fn clear_all_removes_everything() {
		let db = test_db();
		db.history_add_visit("https://a.com", None).unwrap();
		db.history_add_visit("https://b.com", None).unwrap();

		db.history_clear("all").unwrap();

		let entries = db.history_get_recent(10).unwrap();
		assert!(entries.is_empty());
	}

	#[test]
	fn clear_invalid_timeframe_errors() {
		let db = test_db();
		assert!(db.history_clear("invalid").is_err());
	}
}
