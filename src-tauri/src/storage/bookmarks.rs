use rusqlite::params;
use serde::Serialize;
use uuid::Uuid;

use super::database::Database;

#[derive(Debug, Clone, Serialize)]
pub struct Bookmark {
	pub id: String,
	pub parent_id: Option<String>,
	pub title: String,
	pub url: Option<String>,
	pub is_folder: bool,
	pub position: i64,
	pub created_at: String,
	pub updated_at: String,
}

/// Well-known root folder IDs
pub const BOOKMARKS_BAR_ID: &str = "bookmarks-bar";
pub const OTHER_BOOKMARKS_ID: &str = "other-bookmarks";

impl Database {
	/// Seed the root bookmark folders (idempotent)
	pub fn seed_bookmarks(&self) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();
		conn.execute(
			"INSERT OR IGNORE INTO bookmarks (id, parent_id, title, url, is_folder, position) VALUES (?1, NULL, ?2, NULL, TRUE, 0)",
			params![BOOKMARKS_BAR_ID, "Bookmarks Bar"],
		).map_err(|e| e.to_string())?;
		conn.execute(
			"INSERT OR IGNORE INTO bookmarks (id, parent_id, title, url, is_folder, position) VALUES (?1, NULL, ?2, NULL, TRUE, 1)",
			params![OTHER_BOOKMARKS_ID, "Other Bookmarks"],
		).map_err(|e| e.to_string())?;
		Ok(())
	}

	/// Add a bookmark (file or folder)
	pub fn bookmark_add(
		&self,
		parent_id: &str,
		title: &str,
		url: Option<&str>,
		is_folder: bool,
	) -> Result<Bookmark, String> {
		let conn = self.conn.lock().unwrap();
		let id = Uuid::new_v4().to_string();

		// Get the next position within the parent
		let position: i64 = conn
			.query_row(
				"SELECT COALESCE(MAX(position), -1) + 1 FROM bookmarks WHERE parent_id = ?1",
				params![parent_id],
				|row| row.get(0),
			)
			.map_err(|e| e.to_string())?;

		conn.execute(
			"INSERT INTO bookmarks (id, parent_id, title, url, is_folder, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			params![id, parent_id, title, url, is_folder, position],
		)
		.map_err(|e| e.to_string())?;

		let bookmark = conn
			.query_row(
				"SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at FROM bookmarks WHERE id = ?1",
				params![id],
				|row| {
					Ok(Bookmark {
						id: row.get(0)?,
						parent_id: row.get(1)?,
						title: row.get(2)?,
						url: row.get(3)?,
						is_folder: row.get(4)?,
						position: row.get(5)?,
						created_at: row.get(6)?,
						updated_at: row.get(7)?,
					})
				},
			)
			.map_err(|e| e.to_string())?;

		Ok(bookmark)
	}

	/// Update a bookmark's title and/or URL
	pub fn bookmark_update(
		&self,
		id: &str,
		title: Option<&str>,
		url: Option<&str>,
	) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();
		if let Some(title) = title {
			conn.execute(
				"UPDATE bookmarks SET title = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
				params![id, title],
			)
			.map_err(|e| e.to_string())?;
		}
		if let Some(url) = url {
			conn.execute(
				"UPDATE bookmarks SET url = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
				params![id, url],
			)
			.map_err(|e| e.to_string())?;
		}
		Ok(())
	}

	/// Delete a bookmark and all its children (recursive)
	pub fn bookmark_delete(&self, id: &str) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();
		// Recursively delete children first
		let child_ids: Vec<String> = conn
			.prepare("SELECT id FROM bookmarks WHERE parent_id = ?1")
			.map_err(|e| e.to_string())?
			.query_map(params![id], |row| row.get(0))
			.map_err(|e| e.to_string())?
			.filter_map(|r| r.ok())
			.collect();

		drop(conn);
		for child_id in child_ids {
			self.bookmark_delete(&child_id)?;
		}

		let conn = self.conn.lock().unwrap();
		conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])
			.map_err(|e| e.to_string())?;
		Ok(())
	}

	/// Move a bookmark to a new parent and position
	pub fn bookmark_move(
		&self,
		id: &str,
		new_parent_id: &str,
		new_position: i64,
	) -> Result<(), String> {
		let conn = self.conn.lock().unwrap();
		// Shift existing items at or after the new position
		conn.execute(
			"UPDATE bookmarks SET position = position + 1 WHERE parent_id = ?1 AND position >= ?2 AND id != ?3",
			params![new_parent_id, new_position, id],
		)
		.map_err(|e| e.to_string())?;

		conn.execute(
			"UPDATE bookmarks SET parent_id = ?2, position = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
			params![id, new_parent_id, new_position],
		)
		.map_err(|e| e.to_string())?;
		Ok(())
	}

	/// Get children of a folder, ordered by position
	pub fn bookmark_get_children(&self, parent_id: &str) -> Result<Vec<Bookmark>, String> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn
			.prepare(
				"SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at
				 FROM bookmarks WHERE parent_id = ?1 ORDER BY position",
			)
			.map_err(|e| e.to_string())?;

		let entries = stmt
			.query_map(params![parent_id], |row| {
				Ok(Bookmark {
					id: row.get(0)?,
					parent_id: row.get(1)?,
					title: row.get(2)?,
					url: row.get(3)?,
					is_folder: row.get(4)?,
					position: row.get(5)?,
					created_at: row.get(6)?,
					updated_at: row.get(7)?,
				})
			})
			.map_err(|e| e.to_string())?
			.filter_map(|r| r.ok())
			.collect();

		Ok(entries)
	}

	/// Get a single bookmark by ID
	pub fn bookmark_get(&self, id: &str) -> Result<Option<Bookmark>, String> {
		let conn = self.conn.lock().unwrap();
		let result = conn
			.query_row(
				"SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at FROM bookmarks WHERE id = ?1",
				params![id],
				|row| {
					Ok(Bookmark {
						id: row.get(0)?,
						parent_id: row.get(1)?,
						title: row.get(2)?,
						url: row.get(3)?,
						is_folder: row.get(4)?,
						position: row.get(5)?,
						created_at: row.get(6)?,
						updated_at: row.get(7)?,
					})
				},
			)
			.map(Some)
			.unwrap_or(None);
		Ok(result)
	}

	/// Check if a URL is bookmarked (returns the bookmark ID if found)
	pub fn bookmark_is_bookmarked(&self, url: &str) -> Result<Option<String>, String> {
		let conn = self.conn.lock().unwrap();
		let result = conn
			.query_row(
				"SELECT id FROM bookmarks WHERE url = ?1 LIMIT 1",
				params![url],
				|row| row.get(0),
			)
			.map(Some)
			.unwrap_or(None);
		Ok(result)
	}

	/// Search bookmarks by title or URL
	pub fn bookmark_search(&self, query: &str, limit: i64) -> Result<Vec<Bookmark>, String> {
		let conn = self.conn.lock().unwrap();
		let pattern = format!("%{}%", query);
		let mut stmt = conn
			.prepare(
				"SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at
				 FROM bookmarks
				 WHERE (title LIKE ?1 OR url LIKE ?1) AND is_folder = FALSE
				 ORDER BY title
				 LIMIT ?2",
			)
			.map_err(|e| e.to_string())?;

		let entries = stmt
			.query_map(params![pattern, limit], |row| {
				Ok(Bookmark {
					id: row.get(0)?,
					parent_id: row.get(1)?,
					title: row.get(2)?,
					url: row.get(3)?,
					is_folder: row.get(4)?,
					position: row.get(5)?,
					created_at: row.get(6)?,
					updated_at: row.get(7)?,
				})
			})
			.map_err(|e| e.to_string())?
			.filter_map(|r| r.ok())
			.collect();

		Ok(entries)
	}

	/// Get all bookmarks as a flat list (for export/full tree)
	pub fn bookmark_get_all(&self) -> Result<Vec<Bookmark>, String> {
		let conn = self.conn.lock().unwrap();
		let mut stmt = conn
			.prepare(
				"SELECT id, parent_id, title, url, is_folder, position, created_at, updated_at
				 FROM bookmarks ORDER BY position",
			)
			.map_err(|e| e.to_string())?;

		let entries = stmt
			.query_map([], |row| {
				Ok(Bookmark {
					id: row.get(0)?,
					parent_id: row.get(1)?,
					title: row.get(2)?,
					url: row.get(3)?,
					is_folder: row.get(4)?,
					position: row.get(5)?,
					created_at: row.get(6)?,
					updated_at: row.get(7)?,
				})
			})
			.map_err(|e| e.to_string())?
			.filter_map(|r| r.ok())
			.collect();

		Ok(entries)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn test_db() -> Database {
		let db = Database::open_in_memory().unwrap();
		db.seed_bookmarks().unwrap();
		db
	}

	#[test]
	fn seed_creates_root_folders() {
		let db = test_db();
		let bar = db.bookmark_get(BOOKMARKS_BAR_ID).unwrap().unwrap();
		assert_eq!(bar.title, "Bookmarks Bar");
		assert!(bar.is_folder);

		let other = db.bookmark_get(OTHER_BOOKMARKS_ID).unwrap().unwrap();
		assert_eq!(other.title, "Other Bookmarks");
		assert!(other.is_folder);
	}

	#[test]
	fn seed_is_idempotent() {
		let db = test_db();
		db.seed_bookmarks().unwrap();
		let children = db.bookmark_get_children(BOOKMARKS_BAR_ID).unwrap();
		// Should not have duplicates
		assert_eq!(
			db.bookmark_get_all()
				.unwrap()
				.iter()
				.filter(|b| b.id == BOOKMARKS_BAR_ID)
				.count(),
			1
		);
		let _ = children;
	}

	#[test]
	fn add_bookmark_to_bar() {
		let db = test_db();
		let bm = db
			.bookmark_add(BOOKMARKS_BAR_ID, "Google", Some("https://google.com"), false)
			.unwrap();
		assert_eq!(bm.title, "Google");
		assert_eq!(bm.url.as_deref(), Some("https://google.com"));
		assert_eq!(bm.parent_id.as_deref(), Some(BOOKMARKS_BAR_ID));
		assert_eq!(bm.position, 0);
	}

	#[test]
	fn add_multiple_preserves_order() {
		let db = test_db();
		db.bookmark_add(BOOKMARKS_BAR_ID, "A", Some("https://a.com"), false)
			.unwrap();
		db.bookmark_add(BOOKMARKS_BAR_ID, "B", Some("https://b.com"), false)
			.unwrap();
		db.bookmark_add(BOOKMARKS_BAR_ID, "C", Some("https://c.com"), false)
			.unwrap();

		let children = db.bookmark_get_children(BOOKMARKS_BAR_ID).unwrap();
		assert_eq!(children.len(), 3);
		assert_eq!(children[0].title, "A");
		assert_eq!(children[1].title, "B");
		assert_eq!(children[2].title, "C");
	}

	#[test]
	fn add_folder_and_child() {
		let db = test_db();
		let folder = db
			.bookmark_add(BOOKMARKS_BAR_ID, "Dev", None, true)
			.unwrap();
		assert!(folder.is_folder);

		let bm = db
			.bookmark_add(&folder.id, "GitHub", Some("https://github.com"), false)
			.unwrap();
		assert_eq!(bm.parent_id.as_deref(), Some(folder.id.as_str()));
	}

	#[test]
	fn update_bookmark() {
		let db = test_db();
		let bm = db
			.bookmark_add(BOOKMARKS_BAR_ID, "Test", Some("https://test.com"), false)
			.unwrap();

		db.bookmark_update(&bm.id, Some("Updated"), Some("https://updated.com"))
			.unwrap();

		let updated = db.bookmark_get(&bm.id).unwrap().unwrap();
		assert_eq!(updated.title, "Updated");
		assert_eq!(updated.url.as_deref(), Some("https://updated.com"));
	}

	#[test]
	fn delete_bookmark() {
		let db = test_db();
		let bm = db
			.bookmark_add(BOOKMARKS_BAR_ID, "Test", Some("https://test.com"), false)
			.unwrap();

		db.bookmark_delete(&bm.id).unwrap();
		assert!(db.bookmark_get(&bm.id).unwrap().is_none());
	}

	#[test]
	fn delete_folder_cascades() {
		let db = test_db();
		let folder = db
			.bookmark_add(BOOKMARKS_BAR_ID, "Folder", None, true)
			.unwrap();
		let child = db
			.bookmark_add(&folder.id, "Child", Some("https://child.com"), false)
			.unwrap();

		db.bookmark_delete(&folder.id).unwrap();
		assert!(db.bookmark_get(&folder.id).unwrap().is_none());
		assert!(db.bookmark_get(&child.id).unwrap().is_none());
	}

	#[test]
	fn is_bookmarked() {
		let db = test_db();
		assert!(db
			.bookmark_is_bookmarked("https://example.com")
			.unwrap()
			.is_none());

		let bm = db
			.bookmark_add(BOOKMARKS_BAR_ID, "Example", Some("https://example.com"), false)
			.unwrap();

		assert_eq!(
			db.bookmark_is_bookmarked("https://example.com")
				.unwrap()
				.unwrap(),
			bm.id
		);
	}

	#[test]
	fn search_bookmarks() {
		let db = test_db();
		db.bookmark_add(BOOKMARKS_BAR_ID, "Google", Some("https://google.com"), false)
			.unwrap();
		db.bookmark_add(BOOKMARKS_BAR_ID, "GitHub", Some("https://github.com"), false)
			.unwrap();

		let results = db.bookmark_search("goo", 10).unwrap();
		assert_eq!(results.len(), 1);
		assert_eq!(results[0].title, "Google");
	}

	#[test]
	fn move_bookmark() {
		let db = test_db();
		let bm = db
			.bookmark_add(BOOKMARKS_BAR_ID, "Test", Some("https://test.com"), false)
			.unwrap();

		db.bookmark_move(&bm.id, OTHER_BOOKMARKS_ID, 0).unwrap();

		let moved = db.bookmark_get(&bm.id).unwrap().unwrap();
		assert_eq!(moved.parent_id.as_deref(), Some(OTHER_BOOKMARKS_ID));
	}
}
