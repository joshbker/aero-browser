use tauri::{command, AppHandle, Emitter, Manager, WebviewUrl};
use tauri::{LogicalPosition, LogicalSize};

use crate::state::tab_state::{next_tab_label, TabInfo, TabManager};

/// Chrome height in logical pixels (tab bar + toolbar)
/// Keep in sync with CHROME_HEIGHT in src/lib/utils/constants.js
const CHROME_HEIGHT: f64 = 76.0;

/// Helper: get the content area size (below the chrome)
fn get_content_size(app: &AppHandle) -> Result<(f64, f64), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let width = size.width as f64 / scale;
    let height = size.height as f64 / scale;
    Ok((width, (height - CHROME_HEIGHT).max(0.0)))
}

/// Create a new tab webview and register it in state.
/// MUST be async to avoid WebView2 deadlock on Windows.
#[command]
pub async fn tab_create(
    app: AppHandle,
    url: Option<String>,
) -> Result<TabInfo, String> {
    let label = next_tab_label();
    let url = url.unwrap_or_else(|| "https://www.google.com".to_string());

    let webview_url = if url.starts_with("http://") || url.starts_with("https://") {
        WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?)
    } else {
        let full_url = format!("https://{}", url);
        WebviewUrl::External(
            full_url
                .parse()
                .map_err(|e| format!("Invalid URL: {}", e))?,
        )
    };

    let window = app
        .get_window("main")
        .ok_or("Main window not found")?;

    let (width, content_height) = get_content_size(&app)?;

    // Clone for event handlers
    let label_for_load = label.clone();
    let app_for_load = app.clone();

    let webview = tauri::webview::WebviewBuilder::new(&label, webview_url)
        .on_page_load(move |webview, payload| {
            let loading = match payload.event() {
                tauri::webview::PageLoadEvent::Started => true,
                tauri::webview::PageLoadEvent::Finished => false,
            };

            let tab_manager = app_for_load.state::<TabManager>();
            let url_str = payload.url().to_string();
            let label_clone = label_for_load.clone();

            tab_manager.update_tab(&label_clone, |tab| {
                tab.is_loading = loading;
                tab.url = url_str.clone();
            });

            let _ = app_for_load.emit("tab_updated", serde_json::json!({
                "label": label_clone,
                "loading": loading,
                "url": url_str,
            }));

            // When page finishes loading, grab the title
            if !loading {
                let app_clone = app_for_load.clone();
                let label_clone2 = label_clone.clone();
                // Inject JS to get the document title and send it back
                let _ = webview.eval(&format!(
                    r#"
                    (function() {{
                        if (window.__aeroTitleObserver) return;
                        function sendTitle() {{
                            window.__TAURI_INTERNALS__?.invoke('__tab_title_update', {{
                                label: '{}',
                                title: document.title || ''
                            }}).catch(function(){{}});
                        }}
                        sendTitle();
                        window.__aeroTitleObserver = new MutationObserver(sendTitle);
                        var titleEl = document.querySelector('title');
                        if (titleEl) {{
                            window.__aeroTitleObserver.observe(titleEl, {{ childList: true }});
                        }}
                    }})();
                    "#,
                    label_clone2
                ));
            }
        })
        .on_navigation(|_url| true);

    window
        .add_child(
            webview,
            LogicalPosition::new(0.0, CHROME_HEIGHT),
            LogicalSize::new(width, content_height),
        )
        .map_err(|e| format!("Failed to create tab webview: {}", e))?;

    let tab_info = TabInfo {
        label: label.clone(),
        url: url.clone(),
        title: "New Tab".to_string(),
        is_loading: true,
        favicon: None,
    };

    let tab_manager = app.state::<TabManager>();
    tab_manager.add_tab(tab_info.clone());

    // Hide all other tabs and show this one
    let all_labels = tab_manager.get_tab_labels();
    for l in &all_labels {
        if let Some(wv) = app.get_webview(l) {
            if *l == label {
                let _ = wv.show();
            } else {
                let _ = wv.hide();
            }
        }
    }
    tab_manager.set_active_tab(Some(label.clone()));

    let _ = app.emit("tab_created", &tab_info);

    Ok(tab_info)
}

/// Close a tab and its webview. Opens a new tab if this was the last one.
#[command]
pub async fn tab_close(app: AppHandle, label: String) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();

    let adjacent = tab_manager.get_adjacent_tab(&label);
    let was_active = tab_manager.get_active_tab() == Some(label.clone());

    tab_manager.remove_tab(&label);

    if let Some(webview) = app.get_webview(&label) {
        webview.close().map_err(|e| e.to_string())?;
    }

    let _ = app.emit("tab_closed", serde_json::json!({ "label": label }));

    if was_active {
        if let Some(next_label) = adjacent {
            let all_labels = tab_manager.get_tab_labels();
            for l in &all_labels {
                if let Some(wv) = app.get_webview(l) {
                    if *l == next_label {
                        let _ = wv.show();
                    } else {
                        let _ = wv.hide();
                    }
                }
            }
            tab_manager.set_active_tab(Some(next_label.clone()));
            if let Some(tab) = tab_manager.get_tab(&next_label) {
                let _ = app.emit("tab_activated", &tab);
            }
        } else {
            tab_create(app.clone(), None).await?;
        }
    }

    Ok(())
}

/// Switch the active/visible tab
#[command]
pub async fn tab_set_active(app: AppHandle, label: String) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();

    if tab_manager.get_tab(&label).is_none() {
        return Err(format!("Tab {} not found", label));
    }

    let all_labels = tab_manager.get_tab_labels();
    for l in &all_labels {
        if let Some(wv) = app.get_webview(l) {
            if *l == label {
                wv.show().map_err(|e| e.to_string())?;
            } else {
                wv.hide().map_err(|e| e.to_string())?;
            }
        }
    }

    tab_manager.set_active_tab(Some(label.clone()));

    if let Some(tab) = tab_manager.get_tab(&label) {
        let _ = app.emit("tab_activated", &tab);
    }

    Ok(())
}

/// Get all open tabs
#[command]
pub fn tab_get_all(app: AppHandle) -> Result<Vec<TabInfo>, String> {
    let tab_manager = app.state::<TabManager>();
    Ok(tab_manager.get_all_tabs())
}

/// Get the currently active tab label
#[command]
pub fn tab_get_active(app: AppHandle) -> Result<Option<String>, String> {
    let tab_manager = app.state::<TabManager>();
    Ok(tab_manager.get_active_tab())
}

/// Resize all content webviews to fit current window size.
/// Called when the window is resized.
#[command]
pub fn tab_resize_all(app: AppHandle) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let (width, content_height) = get_content_size(&app)?;

    for label in tab_manager.get_tab_labels() {
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.set_position(LogicalPosition::new(0.0, CHROME_HEIGHT));
            let _ = wv.set_size(LogicalSize::new(width, content_height));
        }
    }

    Ok(())
}

/// Internal command: receive title updates from content webviews via JS injection.
/// Not called directly by frontend UI — called from injected JS in content webviews.
#[command]
pub fn __tab_title_update(app: AppHandle, label: String, title: String) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    tab_manager.update_tab(&label, |tab| {
        tab.title = title.clone();
    });

    let _ = app.emit("tab_updated", serde_json::json!({
        "label": label,
        "title": title,
    }));

    Ok(())
}
