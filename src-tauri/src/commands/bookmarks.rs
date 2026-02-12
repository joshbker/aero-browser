use tauri::{command, AppHandle, Emitter, Manager};

use crate::state::chrome_height::ChromeHeight;
use crate::storage::bookmarks::Bookmark;
use crate::storage::database::Database;

/// Add a bookmark
#[command]
pub fn bookmark_add(
	app: AppHandle,
	parent_id: String,
	title: String,
	url: Option<String>,
	is_folder: bool,
) -> Result<Bookmark, String> {
	let db = app.state::<Database>();
	db.bookmark_add(&parent_id, &title, url.as_deref(), is_folder)
}

/// Update a bookmark's title and/or URL
#[command]
pub fn bookmark_update(
	app: AppHandle,
	id: String,
	title: Option<String>,
	url: Option<String>,
) -> Result<(), String> {
	let db = app.state::<Database>();
	db.bookmark_update(&id, title.as_deref(), url.as_deref())
}

/// Delete a bookmark (and children if folder)
#[command]
pub fn bookmark_delete(app: AppHandle, id: String) -> Result<(), String> {
	let db = app.state::<Database>();
	db.bookmark_delete(&id)
}

/// Move a bookmark to a new parent/position
#[command]
pub fn bookmark_move(
	app: AppHandle,
	id: String,
	new_parent_id: String,
	new_position: i64,
) -> Result<(), String> {
	let db = app.state::<Database>();
	db.bookmark_move(&id, &new_parent_id, new_position)
}

/// Get children of a folder
#[command]
pub fn bookmark_get_children(
	app: AppHandle,
	parent_id: String,
) -> Result<Vec<Bookmark>, String> {
	let db = app.state::<Database>();
	db.bookmark_get_children(&parent_id)
}

/// Check if a URL is bookmarked (returns bookmark ID or null)
#[command]
pub fn bookmark_is_bookmarked(
	app: AppHandle,
	url: String,
) -> Result<Option<String>, String> {
	let db = app.state::<Database>();
	db.bookmark_is_bookmarked(&url)
}

/// Search bookmarks
#[command]
pub fn bookmark_search(
	app: AppHandle,
	query: String,
	limit: Option<i64>,
) -> Result<Vec<Bookmark>, String> {
	let db = app.state::<Database>();
	db.bookmark_search(&query, limit.unwrap_or(50))
}

/// Get all bookmarks (flat list)
#[command]
pub fn bookmark_get_all(app: AppHandle) -> Result<Vec<Bookmark>, String> {
	let db = app.state::<Database>();
	db.bookmark_get_all()
}

/// Get a single bookmark by ID
#[command]
pub fn bookmark_get(app: AppHandle, id: String) -> Result<Option<Bookmark>, String> {
	let db = app.state::<Database>();
	db.bookmark_get(&id)
}

/// Toggle bookmarks bar visibility — updates chrome height and resizes webviews
#[command]
pub fn bookmark_toggle_bar(app: AppHandle, visible: bool) -> Result<(), String> {
	let chrome_height = app.state::<ChromeHeight>();
	chrome_height.set_bookmarks_bar(visible);

	// Save the setting
	let db = app.state::<Database>();
	db.settings_set("show_bookmarks_bar", if visible { "true" } else { "false" })?;

	// Emit event so UI can update, then resize all tab webviews
	let _ = app.emit("chrome_height_changed", chrome_height.get());
	let _ = super::tabs::tab_resize_all(app.clone());

	// Resize the UI webview to the new chrome height
	if let Some(ui) = app.get_webview("browser-ui") {
		let window = app.get_window("main").ok_or("Main window not found")?;
		let size = window.inner_size().map_err(|e| e.to_string())?;
		let scale = window.scale_factor().map_err(|e| e.to_string())?;
		let width = size.width as f64 / scale;
		let _ = ui.set_size(tauri::LogicalSize::new(width, chrome_height.get()));
	}

	Ok(())
}
