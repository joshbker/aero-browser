use tauri::{command, AppHandle, Manager};

use crate::storage::database::Database;
use crate::storage::history::HistoryEntry;

/// Search history by URL or title
#[command]
pub fn history_search(
	app: AppHandle,
	query: String,
	limit: Option<i64>,
) -> Result<Vec<HistoryEntry>, String> {
	let db = app.state::<Database>();
	db.history_search(&query, limit.unwrap_or(50))
}

/// Get recent history entries
#[command]
pub fn history_get_recent(
	app: AppHandle,
	limit: Option<i64>,
) -> Result<Vec<HistoryEntry>, String> {
	let db = app.state::<Database>();
	db.history_get_recent(limit.unwrap_or(100))
}

/// Delete a single history entry
#[command]
pub fn history_delete(app: AppHandle, id: String) -> Result<(), String> {
	let db = app.state::<Database>();
	db.history_delete(&id)
}

/// Clear history by timeframe (hour, day, week, all)
#[command]
pub fn history_clear(app: AppHandle, timeframe: String) -> Result<(), String> {
	let db = app.state::<Database>();
	db.history_clear(&timeframe)
}
