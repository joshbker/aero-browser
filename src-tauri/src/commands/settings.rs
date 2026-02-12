use std::collections::HashMap;
use tauri::{command, AppHandle, Manager};

use crate::storage::database::Database;

/// Get a single setting by key
#[command]
pub fn settings_get(app: AppHandle, key: String) -> Result<Option<String>, String> {
	let db = app.state::<Database>();
	db.settings_get(&key)
}

/// Set a single setting
#[command]
pub fn settings_set(app: AppHandle, key: String, value: String) -> Result<(), String> {
	let db = app.state::<Database>();
	db.settings_set(&key, &value)
}

/// Get all settings as a key-value map
#[command]
pub fn settings_get_all(app: AppHandle) -> Result<HashMap<String, String>, String> {
	let db = app.state::<Database>();
	db.settings_get_all()
}
