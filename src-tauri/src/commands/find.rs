use tauri::{command, AppHandle, Emitter, Manager};

use crate::state::tab_state::TabManager;

/// Find text in the active tab's page.
/// On new_search: counts all matches, scrolls to first, highlights it.
/// On next/prev: moves to next/previous match with wrap-around.
#[command]
pub async fn find_in_page(
    app: AppHandle,
    query: String,
    forward: bool,
    new_search: bool,
) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let label = tab_manager.get_active_tab().ok_or("No active tab")?;

    let webview = app
        .get_webview(&label)
        .ok_or("Tab webview not found")?;

    let escaped = query
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n");

    if new_search {
        // New search: clear selection, move to top, find first match, count total
        let js = format!(
            r#"
            (function() {{
                var q = '{}';
                window.getSelection()?.removeAllRanges();

                // Count total matches (case-insensitive)
                var text = document.body.innerText || '';
                var escaped = q.replace(/[.*+?^${{}}()|[\]\\]/g, '\\$&');
                var re = new RegExp(escaped, 'gi');
                var matches = text.match(re);
                var total = matches ? matches.length : 0;

                // Find the first match (forward from start)
                var found = false;
                if (total > 0) {{
                    found = window.find(q, false, false, true, false, true, false);
                }}

                window.__TAURI_INTERNALS__?.invoke('__find_result', {{
                    total: total,
                    current: found ? 1 : 0
                }}).catch(function(){{}});
            }})();
            "#,
            escaped
        );
        webview.eval(&js).map_err(|e| e.to_string())?;
    } else {
        // Continue searching (next/prev) with wrap-around
        let backward = !forward;
        let js = format!(
            r#"
            (function() {{
                var found = window.find('{}', false, {}, true, false, true, false);
                if (!found) {{
                    window.getSelection()?.removeAllRanges();
                    found = window.find('{}', false, {}, true, false, true, false);
                }}
            }})();
            "#,
            escaped, backward,
            escaped, backward
        );
        webview.eval(&js).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Internal command: receive find match count from content webview JS
#[command]
pub fn __find_result(app: AppHandle, total: i32, current: i32) -> Result<(), String> {
    let _ = app.emit("find_result", serde_json::json!({
        "total": total,
        "current": current,
    }));
    Ok(())
}

/// Clear find highlighting in the active tab
#[command]
pub async fn find_clear(app: AppHandle) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let label = tab_manager.get_active_tab().ok_or("No active tab")?;

    let webview = app
        .get_webview(&label)
        .ok_or("Tab webview not found")?;

    webview
        .eval("window.getSelection()?.removeAllRanges()")
        .map_err(|e| e.to_string())?;

    let _ = app.emit("find_result", serde_json::json!({
        "total": 0,
        "current": 0,
    }));

    Ok(())
}
