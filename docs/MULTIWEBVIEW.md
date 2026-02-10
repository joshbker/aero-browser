# Multi-Webview Reference — Aero Browser

This document contains concrete code examples and patterns for Tauri v2's multi-webview API. This is the most architecturally complex part of the project — read this thoroughly before working on tab management.

## Critical: Unstable Feature Flag

Multi-webview support in Tauri v2 requires the `unstable` feature flag. Without it, key APIs like `Window::add_child` and `WebviewBuilder` are private.

```toml
# In src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2", features = ["unstable"] }
```

## Key Concepts

Tauri v2 separates `Window` and `Webview`:
- **Window** — the native OS window (title bar, frame, resizing)
- **Webview** — a web content renderer that lives inside a window
- **WebviewWindow** — convenience type that creates a window hosting a single webview (DON'T use this for our tab architecture)

For Aero, we have ONE Window containing MULTIPLE Webviews:
- One webview for the browser UI (SvelteKit app — tab bar, address bar, etc.)
- One webview per tab (content webviews showing actual web pages)

## Window + Webview Setup

### Initial Setup in main.rs / lib.rs

```rust
use tauri::{
    LogicalPosition, LogicalSize,
    webview::WebviewBuilder,
    window::WindowBuilder,
    WebviewUrl,
};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Create the main window with NO decorations (we render our own title bar)
            let window = WindowBuilder::new(app, "main")
                .title("Aero")
                .inner_size(1280.0, 800.0)
                .decorations(false)
                .build()?;

            let size = window.inner_size()?;
            let width = size.width as f64;
            let height = size.height as f64;

            // UI chrome height (tab bar + toolbar)
            let chrome_height = 76.0; // Adjust as needed

            // Browser UI webview (SvelteKit app) — fixed at top
            let ui_webview = WebviewBuilder::new(
                "browser-ui",
                WebviewUrl::App("index.html".into()),
            );
            window.add_child(
                ui_webview,
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(width, chrome_height),
            )?;

            // Initial tab webview — fills remaining space below UI
            let tab_webview = WebviewBuilder::new(
                "tab-0",
                WebviewUrl::External("https://google.com".parse().unwrap()),
            )
            .auto_resize(); // Auto-resize with window

            window.add_child(
                tab_webview,
                LogicalPosition::new(0.0, chrome_height),
                LogicalSize::new(width, height - chrome_height),
            )?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Register commands here
        ])
        .run(tauri::generate_context!())
        .expect("error while running Aero");
}
```

## Creating Tab Webviews Dynamically

### CRITICAL: Use async commands

On Windows, creating webviews in synchronous commands causes a **deadlock** due to a WebView2 limitation. Always use `async fn` for commands that create or destroy webviews.

```rust
use tauri::{command, AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use std::sync::atomic::{AtomicU64, Ordering};

// Global tab ID counter
static TAB_COUNTER: AtomicU64 = AtomicU64::new(1);

#[command]
pub async fn tab_create(
    app: AppHandle,
    url: Option<String>,
) -> Result<String, String> {
    let tab_id = TAB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let label = format!("tab-{}", tab_id);

    let url = url.unwrap_or_else(|| "aero://newtab".to_string());

    let webview_url = if url.starts_with("http://") || url.starts_with("https://") {
        WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?)
    } else {
        WebviewUrl::App("index.html".into()) // Internal pages
    };

    let window = app
        .get_window("main")
        .ok_or("Main window not found")?;

    let size = window.inner_size().map_err(|e| e.to_string())?;
    let chrome_height = 76.0;

    let webview = tauri::webview::WebviewBuilder::new(&label, webview_url)
        .auto_resize();

    window
        .add_child(
            webview,
            LogicalPosition::new(0.0, chrome_height),
            LogicalSize::new(size.width as f64, size.height as f64 - chrome_height),
        )
        .map_err(|e| format!("Failed to create tab webview: {}", e))?;

    Ok(label)
}
```

## Switching Active Tab

Only one content webview should be visible at a time. Hide inactive tabs, show the active one.

```rust
#[command]
pub async fn tab_set_active(
    app: AppHandle,
    tab_label: String,
    all_tab_labels: Vec<String>,
) -> Result<(), String> {
    for label in &all_tab_labels {
        if let Some(webview) = app.get_webview(label) {
            if *label == tab_label {
                webview.show().map_err(|e| e.to_string())?;
                // Bring to front / ensure correct z-order
            } else {
                webview.hide().map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
```

## Closing a Tab

```rust
#[command]
pub async fn tab_close(app: AppHandle, tab_label: String) -> Result<(), String> {
    if let Some(webview) = app.get_webview(&tab_label) {
        webview.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

## Navigating a Tab

```rust
#[command]
pub async fn navigate_to(
    app: AppHandle,
    tab_label: String,
    url: String,
) -> Result<(), String> {
    let webview = app
        .get_webview(&tab_label)
        .ok_or("Tab not found")?;

    let parsed_url: url::Url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
    webview
        .navigate(parsed_url)
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

## Listening to Webview Events

Track page load events, title changes, URL changes from content webviews and relay them to the UI webview.

```rust
// During webview creation, attach event handlers:
let webview = WebviewBuilder::new(&label, webview_url)
    .on_page_load(move |webview, payload| {
        match payload.event() {
            tauri::webview::PageLoadEvent::Started => {
                // Emit loading started event to UI
                let _ = webview.emit("tab_loading", serde_json::json!({
                    "tab_label": label_clone,
                    "loading": true,
                    "url": payload.url().to_string(),
                }));
            }
            tauri::webview::PageLoadEvent::Finished => {
                // Emit loading finished event to UI
                let _ = webview.emit("tab_loading", serde_json::json!({
                    "tab_label": label_clone,
                    "loading": false,
                    "url": payload.url().to_string(),
                }));
            }
        }
    })
    .on_document_title_changed(move |webview, title| {
        let _ = webview.emit("tab_title_changed", serde_json::json!({
            "tab_label": label_clone2,
            "title": title,
        }));
    })
    .on_new_window(move |url, features| {
        // Handle window.open() / target="_blank" links
        // Create a new tab instead of a new window
        // Return NewWindowResponse appropriately
        todo!("Implement new window -> new tab logic")
    });
```

## Frontend: Listening to Tab Events

```javascript
// In src/lib/stores/tabs.js
import { listen } from '@tauri-apps/api/event'
import { writable } from 'svelte/store'

function createTabStore() {
    const { subscribe, set, update } = writable({
        tabs: [],
        activeTabLabel: null,
    })

    // Listen for backend events
    listen('tab_loading', (event) => {
        const { tab_label, loading, url } = event.payload
        update(state => ({
            ...state,
            tabs: state.tabs.map(tab =>
                tab.label === tab_label
                    ? { ...tab, loading, url }
                    : tab
            ),
        }))
    })

    listen('tab_title_changed', (event) => {
        const { tab_label, title } = event.payload
        update(state => ({
            ...state,
            tabs: state.tabs.map(tab =>
                tab.label === tab_label
                    ? { ...tab, title }
                    : tab
            ),
        }))
    })

    return {
        subscribe,
        // ... store methods
    }
}
```

## Resizing on Window Resize

When the window resizes, content webviews need to be repositioned. Use `auto_resize()` on the WebviewBuilder where possible, or listen for window resize events:

```rust
// If auto_resize() isn't sufficient, handle manually:
window.on_window_event(move |event| {
    if let tauri::WindowEvent::Resized(size) = event {
        // Resize active content webview
        // UI webview width should match window width
        // Content webview should fill remaining height below chrome
    }
});
```

## Important Gotchas

1. **Deadlock on Windows**: NEVER create/destroy webviews in synchronous commands or event handlers. Always use `async fn`.

2. **Unique labels**: Every webview MUST have a unique label. Use the atomic counter pattern shown above.

3. **auto_resize()**: Call this on the WebviewBuilder to have webviews automatically resize with the window. Very useful but may not handle the chrome height offset correctly — test thoroughly.

4. **IPC scope**: By default, Tauri IPC commands are only accessible from webviews that are created with `WebviewUrl::App(...)`. External URL webviews (content tabs) cannot call `invoke()`. This is actually what we want — content pages should NOT have access to our IPC commands. Only the browser UI webview should.

5. **Event routing**: Events emitted with `webview.emit()` are scoped to that webview. Use `app.emit()` for global events that the UI webview needs to receive. Alternatively, emit to a specific webview label with `app.emit_to("browser-ui", ...)`.

6. **z-ordering**: When showing/hiding webviews, be mindful of z-order. The UI webview should always be on top (rendered above content webviews at its position), and only the active tab's webview should be visible.

7. **Memory**: Each webview has baseline memory overhead (~50-100MB). For many tabs, consider implementing tab suspension (destroying the webview but keeping the URL/state so it can be recreated when activated).
