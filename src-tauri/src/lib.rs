mod commands;
mod state;
mod storage;

use state::tab_state::TabManager;
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
            commands::tabs::__tab_title_update,
            commands::tabs::__tab_nav_state_update,
            // Navigation commands
            commands::navigation::navigate_to,
            commands::navigation::navigate_back,
            commands::navigation::navigate_forward,
            commands::navigation::navigate_refresh,
            commands::navigation::navigate_stop,
            commands::navigation::navigate_get_url,
        ])
        .setup(|app| {
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

            // Listen for window resize — resize the UI webview width + all content webviews
            let app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(size) = event {
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
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Aero");
}
