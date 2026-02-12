mod commands;
mod state;
mod storage;

use state::tab_state::TabManager;
use storage::database::Database;
use tauri::{LogicalPosition, LogicalSize, Manager, WebviewUrl};

/// Chrome height — must match CHROME_HEIGHT in commands/tabs.rs and frontend constants.js
const CHROME_HEIGHT: f64 = 76.0;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(TabManager::new())
        .invoke_handler(tauri::generate_handler![
            // Tab commands
            commands::tabs::tab_create,
            commands::tabs::tab_close,
            commands::tabs::tab_set_active,
            commands::tabs::tab_get_all,
            commands::tabs::tab_get_active,
            commands::tabs::tab_resize_all,
            commands::tabs::tab_duplicate,
            commands::tabs::tab_reorder,
            commands::tabs::__tab_title_update,
            commands::tabs::__tab_favicon_update,
            commands::tabs::ui_focus,
            commands::tabs::ui_set_height,
            commands::tabs::show_context_menu,
            commands::tabs::close_context_menu,
            // Navigation commands
            commands::navigation::navigate_to,
            commands::navigation::navigate_back,
            commands::navigation::navigate_forward,
            commands::navigation::navigate_refresh,
            commands::navigation::navigate_stop,
            commands::navigation::navigate_get_url,
            // Find commands
            commands::find::find_in_page,
            commands::find::find_clear,
            commands::find::__find_result,
            // Settings commands
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::settings::settings_get_all,
        ])
        .setup(|app| {
            // Open the database in {app_data_dir}/default/browser.db
            let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
            let profile_dir = app_data.join("default");
            std::fs::create_dir_all(&profile_dir)
                .map_err(|e| format!("Failed to create profile dir: {}", e))?;
            let db_path = profile_dir.join("browser.db");
            let db = Database::open(
                db_path
                    .to_str()
                    .ok_or_else(|| "Invalid DB path".to_string())?,
            )
            .map_err(|e| format!("Failed to open database: {}", e))?;
            db.seed_settings()
                .map_err(|e| format!("Failed to seed settings: {}", e))?;
            app.manage(db);

            let width = 1280.0_f64;
            let height = 800.0_f64;

            // Create the main window (no decorations — we render our own title bar)
            let window = tauri::window::WindowBuilder::new(app, "main")
                .title("Aero")
                .inner_size(width, height)
                .min_inner_size(400.0, 300.0)
                .decorations(false)
                .resizable(true)
                .center()
                .build()?;

            // Browser UI webview (SvelteKit app) — only covers the chrome area (top 76px)
            let ui_webview = tauri::webview::WebviewBuilder::new(
                "browser-ui",
                WebviewUrl::App("index.html".into()),
            )
            .transparent(true)
            .auto_resize();

            window.add_child(
                ui_webview,
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(width, CHROME_HEIGHT),
            )?;

            // Listen for window events — resize webviews and close context menu on move
            let app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Resized(size) => {
                        let scale = app_handle
                            .get_window("main")
                            .and_then(|w| w.scale_factor().ok())
                            .unwrap_or(1.0);
                        let new_width = size.width as f64 / scale;

                        // Resize UI webview width (height stays CHROME_HEIGHT)
                        if let Some(ui) = app_handle.get_webview("browser-ui") {
                            let _ = ui.set_size(LogicalSize::new(new_width, CHROME_HEIGHT));
                        }

                        // Resize all content webviews
                        let _ = commands::tabs::tab_resize_all(app_handle.clone());

                        // Close context menu popup if open
                        if let Some(w) = app_handle.get_window("ctx-menu") {
                            let _ = w.close();
                        }
                    }
                    tauri::WindowEvent::Moved(_) => {
                        // Close context menu popup when main window moves
                        if let Some(w) = app_handle.get_window("ctx-menu") {
                            let _ = w.close();
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Aero");
}
