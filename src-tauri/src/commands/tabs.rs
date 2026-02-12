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

                // On page finish: track navigation in nav_stack
                if !loading {
                    if tab.nav_traversing {
                        // This was a back/forward — nav_pos already updated, just clear flag
                        tab.nav_traversing = false;
                    } else {
                        // This is a new navigation (link click, redirect after initial load, etc.)
                        // Only push if URL differs from current stack position
                        let current = if tab.nav_pos >= 0 && (tab.nav_pos as usize) < tab.nav_stack.len() {
                            Some(tab.nav_stack[tab.nav_pos as usize].as_str())
                        } else {
                            None
                        };
                        if current != Some(&url_str) {
                            let new_pos = tab.nav_pos + 1;
                            tab.nav_stack.truncate(new_pos as usize);
                            tab.nav_stack.push(url_str.clone());
                            tab.nav_pos = new_pos;
                        }
                    }
                    tab.can_go_back = tab.nav_pos > 0;
                    tab.can_go_forward = (tab.nav_pos as usize) < tab.nav_stack.len() - 1;
                }
            });

            // Read nav state for the event
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

            // When page finishes loading, inject Aero helpers (title + hover)
            if !loading {
                let label_inject = label_clone.clone();
                let _ = webview.eval(&format!(
                    r#"
                    (function() {{
                        if (window.__aeroInjected) return;
                        window.__aeroInjected = true;
                        var label = '{}';

                        // --- Title detection ---
                        function sendTitle() {{
                            window.__TAURI_INTERNALS__?.invoke('__tab_title_update', {{
                                label: label,
                                title: document.title || ''
                            }}).catch(function(){{}});
                        }}
                        sendTitle();
                        var titleObs = new MutationObserver(sendTitle);
                        var titleEl = document.querySelector('title');
                        if (titleEl) {{
                            titleObs.observe(titleEl, {{ childList: true }});
                        }}

                        // --- Favicon detection ---
                        function sendFavicon() {{
                            var link = document.querySelector('link[rel*="icon"]');
                            var faviconUrl = link ? link.href : '';
                            if (!faviconUrl) {{
                                try {{
                                    faviconUrl = new URL('/favicon.ico', window.location.origin).href;
                                }} catch(e) {{}}
                            }}
                            window.__TAURI_INTERNALS__?.invoke('__tab_favicon_update', {{
                                label: label,
                                favicon: faviconUrl || ''
                            }}).catch(function(){{}});
                        }}
                        sendFavicon();
                        var faviconObs = new MutationObserver(sendFavicon);
                        var headEl = document.querySelector('head');
                        if (headEl) {{
                            faviconObs.observe(headEl, {{ childList: true, subtree: true }});
                        }}

                        // --- Link hover status bar ---
                        var statusEl = document.createElement('div');
                        statusEl.id = '__aero_status';
                        statusEl.style.cssText = 'position:fixed;bottom:0;left:0;max-width:50%;padding:2px 8px;background:rgba(38,38,38,0.95);border-top:1px solid rgba(64,64,64,0.8);border-right:1px solid rgba(64,64,64,0.8);border-top-right-radius:4px;color:rgba(163,163,163,1);font-size:12px;font-family:system-ui,sans-serif;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;z-index:2147483647;display:none;pointer-events:none;transition:opacity 0.15s;';
                        document.documentElement.appendChild(statusEl);

                        // Show loading text, then hide once page is fully loaded
                        statusEl.textContent = 'Loading...';
                        statusEl.style.display = 'block';
                        function hideLoading() {{
                            if (statusEl.textContent === 'Loading...') {{
                                statusEl.style.display = 'none';
                            }}
                        }}
                        if (document.readyState === 'complete') {{
                            hideLoading();
                        }} else {{
                            window.addEventListener('load', hideLoading);
                            // Fallback: hide after 2s even if load doesn't fire
                            setTimeout(hideLoading, 2000);
                        }}

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
                    label_inject
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
        nav_stack: Vec::new(),
        nav_pos: -1,
        nav_traversing: false,
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

/// Internal command: receive favicon updates from content webviews via JS injection.
#[command]
pub fn __tab_favicon_update(app: AppHandle, label: String, favicon: String) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();

    let favicon_opt = if favicon.is_empty() || !favicon.starts_with("http") {
        None
    } else {
        Some(favicon.clone())
    };

    tab_manager.update_tab(&label, |tab| {
        tab.favicon = favicon_opt.clone();
    });

    let _ = app.emit("tab_updated", serde_json::json!({
        "label": label,
        "favicon": favicon_opt,
    }));

    Ok(())
}

/// Reorder a tab to a new position in the tab list
#[command]
pub fn tab_reorder(
    app: AppHandle,
    label: String,
    new_index: usize,
) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();

    let mut tabs = tab_manager.tabs.lock().unwrap();

    let old_index = tabs.iter()
        .position(|t| t.label == label)
        .ok_or("Tab not found")?;

    if new_index >= tabs.len() {
        return Err("Invalid index".to_string());
    }

    let tab = tabs.remove(old_index);
    tabs.insert(new_index, tab);

    drop(tabs);

    let _ = app.emit("tab_reordered", serde_json::json!({
        "label": label,
        "old_index": old_index,
        "new_index": new_index,
    }));

    Ok(())
}

/// Show a native popup context menu as a separate borderless window on top of everything.
/// Uses anchor-based navigation for menu item clicks (no __TAURI_INTERNALS__ needed).
/// Auto-closes on focus loss, main window move, or Escape.
/// MUST be async to avoid WebView2 deadlock on Windows.
#[command]
pub async fn show_context_menu(
    app: AppHandle,
    x: f64,
    y: f64,
    tab_label: String,
    items: Vec<serde_json::Value>,
) -> Result<(), String> {
    // Close any existing context menu popup
    close_context_menu(app.clone())?;

    // Get the main window's screen position so we can place the popup relative to it
    let main_window = app.get_window("main").ok_or("Main window not found")?;
    let main_pos = main_window.outer_position().map_err(|e| e.to_string())?;
    let scale = main_window.scale_factor().map_err(|e| e.to_string())?;

    // outer_position returns physical pixels, convert to logical and add click offset
    let screen_x = (main_pos.x as f64 / scale) + x;
    let screen_y = (main_pos.y as f64 / scale) + y;

    // Build the menu HTML — menu items use onclick + window.location to trigger
    // on_navigation interception, avoiding WebView2's built-in link hover tooltip
    // The menu div has no border-radius — the window IS the menu, no gaps
    let mut menu_html = String::from(
        r#"<!DOCTYPE html><html><head><style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        html, body { background: #262626; overflow: hidden; margin: 0; height: 100%; }
        .menu { background: #262626; border: 1px solid #404040; border-radius: 6px;
                padding: 4px 0; font-family: system-ui, -apple-system, sans-serif;
                min-height: 100%; }
        .item { padding: 6px 12px; color: #d4d4d4; font-size: 13px;
                cursor: pointer; user-select: none; white-space: nowrap; }
        .item:hover { background: #404040; }
        .sep { margin: 4px 0; border-top: 1px solid #404040; }
        </style></head><body><div class="menu">"#,
    );

    for item in &items {
        if item.get("separator").and_then(|v| v.as_bool()).unwrap_or(false) {
            menu_html.push_str(r#"<div class="sep"></div>"#);
        } else {
            let label = item.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let action = item.get("action").and_then(|v| v.as_str()).unwrap_or("");
            menu_html.push_str(&format!(
                r#"<div class="item" onclick="window.location='aero://action/{}';">{}</div>"#,
                action, label
            ));
        }
    }

    menu_html.push_str("</div></body></html>");

    // Measure menu size:
    // Each item: 6px top pad + 13px font + 6px bottom pad = 25px
    // Each separator: 4px top margin + 1px border + 4px bottom margin = 9px
    // Menu: 1px border top + 4px padding top + items + 4px padding bottom + 1px border bottom = +10px
    let item_count = items.iter().filter(|i| !i.get("separator").and_then(|v| v.as_bool()).unwrap_or(false)).count();
    let sep_count = items.len() - item_count;
    let menu_height = (item_count as f64 * 25.0) + (sep_count as f64 * 9.0) + 24.0;
    let menu_width = 200.0;

    // Create a borderless, always-on-top popup window
    let app_for_focus = app.clone();
    let popup = tauri::window::WindowBuilder::new(&app, "ctx-menu")
        .title("")
        .inner_size(menu_width, menu_height)
        .position(screen_x, screen_y)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .focused(true)
        .transparent(true)
        .build()
        .map_err(|e| format!("Failed to create context menu window: {}", e))?;

    // Auto-close popup when it loses focus (click outside)
    popup.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if let Some(w) = app_for_focus.get_window("ctx-menu") {
                let _ = w.close();
            }
        }
    });

    // Build the webview with navigation interception for menu actions
    let webview_url = WebviewUrl::External(
        "about:blank".parse().map_err(|e| format!("Invalid URL: {}", e))?
    );

    let tab_label_clone = tab_label.clone();
    let app_for_nav = app.clone();
    let webview = tauri::webview::WebviewBuilder::new("ctx-menu-wv", webview_url)
        .transparent(true)
        .auto_resize()
        .on_navigation(move |url| {
            let url_str = url.to_string();
            // Intercept aero://action/* URLs — these are menu item clicks
            if url_str.starts_with("aero://action/") {
                let action = url_str.trim_start_matches("aero://action/").to_string();
                // Close the popup
                if let Some(w) = app_for_nav.get_window("ctx-menu") {
                    let _ = w.close();
                }
                // Emit the action event to the UI webview
                let _ = app_for_nav.emit("context_menu_action", serde_json::json!({
                    "tab_label": tab_label_clone,
                    "action": action,
                }));
                return false; // Block the navigation
            }
            // Allow about:blank
            url_str == "about:blank" || url_str == "about:blank/"
        });

    popup
        .add_child(
            webview,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(menu_width, menu_height),
        )
        .map_err(|e| format!("Failed to add context menu webview: {}", e))?;

    // Inject the menu HTML into the webview
    if let Some(wv) = app.get_webview("ctx-menu-wv") {
        let escaped = menu_html.replace('\\', "\\\\").replace('`', "\\`");
        let _ = wv.eval(&format!("document.open();document.write(`{}`);document.close();", escaped));
    }

    Ok(())
}

/// Close the context menu popup window
#[command]
pub fn close_context_menu(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("ctx-menu") {
        let _ = window.close();
    }
    Ok(())
}

/// Focus the browser-ui webview (used when global shortcuts need to interact with UI)
#[command]
pub fn ui_focus(app: AppHandle) -> Result<(), String> {
    let webview = app
        .get_webview("browser-ui")
        .ok_or("browser-ui webview not found")?;
    webview.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// Resize the browser-ui webview height (e.g. to show context menus below the 76px chrome)
#[command]
pub fn ui_set_height(app: AppHandle, height: f64) -> Result<(), String> {
    let webview = app
        .get_webview("browser-ui")
        .ok_or("browser-ui webview not found")?;

    let window = app.get_window("main").ok_or("Main window not found")?;
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let width = size.width as f64 / scale;

    webview
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    Ok(())
}
