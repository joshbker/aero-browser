use tauri::{command, AppHandle, Emitter, Manager};

use crate::state::tab_state::TabManager;

/// Helper: update can_go_back/forward from nav_stack/nav_pos, then emit event.
fn emit_nav_state(app: &AppHandle, label: &str) {
    let tab_manager = app.state::<TabManager>();

    let (can_go_back, can_go_forward) = {
        let tabs = tab_manager.tabs.lock().unwrap();
        if let Some(tab) = tabs.iter().find(|t| t.label == label) {
            let back = tab.nav_pos > 0;
            let fwd = tab.nav_pos < (tab.nav_stack.len() as i32 - 1);
            (back, fwd)
        } else {
            (false, false)
        }
    };

    tab_manager.update_tab(label, |tab| {
        tab.can_go_back = can_go_back;
        tab.can_go_forward = can_go_forward;
    });

    let _ = app.emit("tab_updated", serde_json::json!({
        "label": label,
        "can_go_back": can_go_back,
        "can_go_forward": can_go_forward,
    }));
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

    // Push onto nav stack (truncate any forward history)
    tab_manager.update_tab(&target_label, |tab| {
        tab.url = url.clone();
        tab.is_loading = true;
        let new_pos = tab.nav_pos + 1;
        tab.nav_stack.truncate(new_pos as usize);
        tab.nav_stack.push(url.clone());
        tab.nav_pos = new_pos;
    });

    emit_nav_state(&app, &target_label);

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

    // Only go back if we can
    let can_back = {
        let tabs = tab_manager.tabs.lock().unwrap();
        tabs.iter().find(|t| t.label == label).map(|t| t.nav_pos > 0).unwrap_or(false)
    };

    if !can_back {
        return Ok(());
    }

    tab_manager.update_tab(&label, |tab| {
        tab.nav_traversing = true;
        tab.nav_pos -= 1;
    });

    webview
        .eval("window.history.back()")
        .map_err(|e| e.to_string())?;

    emit_nav_state(&app, &label);

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

    // Only go forward if we can
    let can_fwd = {
        let tabs = tab_manager.tabs.lock().unwrap();
        tabs.iter().find(|t| t.label == label)
            .map(|t| t.nav_pos < (t.nav_stack.len() as i32 - 1))
            .unwrap_or(false)
    };

    if !can_fwd {
        return Ok(());
    }

    tab_manager.update_tab(&label, |tab| {
        tab.nav_traversing = true;
        tab.nav_pos += 1;
    });

    webview
        .eval("window.history.forward()")
        .map_err(|e| e.to_string())?;

    emit_nav_state(&app, &label);

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
