use tauri::{command, AppHandle, Emitter, Manager, WebviewUrl};
use tauri::webview::NewWindowResponse;
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

    let app_for_new_window = app.clone();
    let webview = tauri::webview::WebviewBuilder::new(&label, webview_url)
        .on_new_window(move |url, _features| {
            let _ = app_for_new_window.emit("open_in_new_tab", url.to_string());
            NewWindowResponse::Deny
        })
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

                // Track navigation history on page finish
                if !loading {
                    let current_url = if tab.nav_index >= 0 && (tab.nav_index as usize) < tab.nav_history.len() {
                        Some(tab.nav_history[tab.nav_index as usize].clone())
                    } else {
                        None
                    };
                    // Only push if this is a genuinely new navigation (not back/forward)
                    if current_url.as_deref() != Some(&url_str) {
                        // Truncate forward history
                        let idx = (tab.nav_index + 1) as usize;
                        tab.nav_history.truncate(idx);
                        tab.nav_history.push(url_str.clone());
                        tab.nav_index = tab.nav_history.len() as i32 - 1;
                    }
                    tab.can_go_back = tab.nav_index > 0;
                    tab.can_go_forward = (tab.nav_index as usize) < tab.nav_history.len() - 1;
                }
            });

            // Read updated state for the event
            let (can_go_back, can_go_forward) = tab_manager
                .get_tab(&label_clone)
                .map(|t| (t.can_go_back, t.can_go_forward))
                .unwrap_or((false, false));

            let _ = app_for_load.emit("tab_updated", serde_json::json!({
                "label": label_clone,
                "loading": loading,
                "url": url_str,
                "can_go_back": can_go_back,
                "can_go_forward": can_go_forward,
            }));

            // When page finishes loading, grab the title
            if !loading {
                let app_clone = app_for_load.clone();
                let label_clone2 = label_clone.clone();
                // Inject JS to get the document title and track link hovers
                let _ = webview.eval(&format!(
                    r#"
                    (function() {{
                        if (window.__aeroInjected) return;
                        window.__aeroInjected = true;

                        // Title detection
                        function sendTitle() {{
                            window.__TAURI_INTERNALS__?.invoke('__tab_title_update', {{
                                label: '{}',
                                title: document.title || ''
                            }}).catch(function(){{}});
                        }}
                        sendTitle();
                        var titleObs = new MutationObserver(sendTitle);
                        var titleEl = document.querySelector('title');
                        if (titleEl) {{
                            titleObs.observe(titleEl, {{ childList: true }});
                        }}

                        // Link hover status bar (rendered in content webview, bottom-left)
                        var statusEl = document.createElement('div');
                        statusEl.id = '__aero_status';
                        statusEl.style.cssText = 'position:fixed;bottom:0;left:0;max-width:50%;padding:2px 8px;background:rgba(38,38,38,0.95);border-top:1px solid rgba(64,64,64,0.8);border-right:1px solid rgba(64,64,64,0.8);border-top-right-radius:4px;color:rgba(163,163,163,1);font-size:12px;font-family:system-ui,sans-serif;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;z-index:2147483647;display:none;pointer-events:none;transition:opacity 0.15s;';
                        document.documentElement.appendChild(statusEl);

                        var lastHref = '';
                        document.addEventListener('mouseover', function(e) {{
                            var a = e.target.closest('a[href]');
                            var href = a ? a.href : '';
                            if (href !== lastHref) {{
                                lastHref = href;
                                if (href) {{
                                    statusEl.textContent = href;
                                    statusEl.style.display = 'block';
                                }} else {{
                                    statusEl.style.display = 'none';
                                }}
                            }}
                        }}, true);
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
        can_go_back: false,
        can_go_forward: false,
        nav_history: vec![url.clone()],
        nav_index: 0,
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

/// Duplicate a tab — creates a new tab with the same URL.
#[command]
pub async fn tab_duplicate(app: AppHandle, label: String) -> Result<TabInfo, String> {
    let tab_manager = app.state::<TabManager>();
    let tab = tab_manager.get_tab(&label).ok_or("Tab not found")?;
    tab_create(app, Some(tab.url)).await
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
