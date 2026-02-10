use tauri::{command, AppHandle, Manager};

use crate::state::tab_state::TabManager;

/// JS snippet that queries nav state and sends it back to Rust.
/// Uses setTimeout to let the history navigation settle first.
fn nav_state_check_js(label: &str) -> String {
    format!(
        r#"
        (function() {{
            function check() {{
                var canBack = window.navigation ? window.navigation.canGoBack : (window.history.length > 1);
                var canFwd = window.navigation ? window.navigation.canGoForward : false;
                window.__TAURI_INTERNALS__?.invoke('__tab_nav_state_update', {{
                    label: '{}',
                    can_go_back: canBack,
                    can_go_forward: canFwd
                }}).catch(function(){{}});
            }}
            setTimeout(check, 50);
            setTimeout(check, 200);
            setTimeout(check, 600);
        }})();
        "#,
        label
    )
}

/// Navigate the active tab (or a specific tab) to a URL
#[command]
pub async fn navigate_to(
    app: AppHandle,
    url: String,
    label: Option<String>,
) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let target_label = label
        .or_else(|| tab_manager.get_active_tab())
        .ok_or("No active tab")?;

    let webview = app
        .get_webview(&target_label)
        .ok_or("Tab webview not found")?;

    let parsed_url: url::Url = url
        .parse()
        .map_err(|e| format!("Invalid URL: {}", e))?;

    webview
        .navigate(parsed_url)
        .map_err(|e| e.to_string())?;

    // Update state
    tab_manager.update_tab(&target_label, |tab| {
        tab.url = url.clone();
        tab.is_loading = true;
    });

    Ok(())
}

/// Navigate back in the active tab
#[command]
pub async fn navigate_back(app: AppHandle) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let label = tab_manager.get_active_tab().ok_or("No active tab")?;

    let webview = app
        .get_webview(&label)
        .ok_or("Tab webview not found")?;

    webview
        .eval("window.history.back()")
        .map_err(|e| e.to_string())?;

    // Query nav state after the history navigation settles
    let _ = webview.eval(&nav_state_check_js(&label));

    Ok(())
}

/// Navigate forward in the active tab
#[command]
pub async fn navigate_forward(app: AppHandle) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let label = tab_manager.get_active_tab().ok_or("No active tab")?;

    let webview = app
        .get_webview(&label)
        .ok_or("Tab webview not found")?;

    webview
        .eval("window.history.forward()")
        .map_err(|e| e.to_string())?;

    // Query nav state after the history navigation settles
    let _ = webview.eval(&nav_state_check_js(&label));

    Ok(())
}

/// Refresh the active tab
#[command]
pub async fn navigate_refresh(app: AppHandle) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let label = tab_manager.get_active_tab().ok_or("No active tab")?;

    let webview = app
        .get_webview(&label)
        .ok_or("Tab webview not found")?;

    webview
        .eval("window.location.reload()")
        .map_err(|e| e.to_string())?;

    tab_manager.update_tab(&label, |tab| {
        tab.is_loading = true;
    });

    Ok(())
}

/// Stop loading the active tab
#[command]
pub async fn navigate_stop(app: AppHandle) -> Result<(), String> {
    let tab_manager = app.state::<TabManager>();
    let label = tab_manager.get_active_tab().ok_or("No active tab")?;

    let webview = app
        .get_webview(&label)
        .ok_or("Tab webview not found")?;

    webview
        .eval("window.stop()")
        .map_err(|e| e.to_string())?;

    tab_manager.update_tab(&label, |tab| {
        tab.is_loading = false;
    });

    Ok(())
}

/// Get the current URL of the active tab by querying the webview
#[command]
pub async fn navigate_get_url(app: AppHandle) -> Result<String, String> {
    let tab_manager = app.state::<TabManager>();
    let label = tab_manager.get_active_tab().ok_or("No active tab")?;

    let tab = tab_manager
        .get_tab(&label)
        .ok_or("Tab not found")?;

    Ok(tab.url)
}
