# Tasks — Aero Browser

Active task tracker. Update this file as tasks are started, completed, or blocked.

Legend: `[ ]` todo · `[x]` done · `[!]` blocked · `[~]` in progress

---

## Phase 0: Project Setup

- [x] Scaffold Tauri v2 + SvelteKit project with `npm create tauri-app@latest`
- [x] Select Svelte and JavaScript during scaffolding
- [x] Install dependencies: `tailwindcss`, `lucide-svelte`
- [x] Configure Tailwind CSS with `neutral` colour palette
- [x] Configure `adapter-static` in `svelte.config.js` (required for Tauri)
- [x] Set `decorations: false` in `tauri.conf.json` (custom title bar)
- [x] Add `unstable` feature flag to Tauri dependency in `Cargo.toml` (required for multi-webview)
- [x] Set up folder structure per `PROJECT.md` architecture
- [x] Add `rusqlite` dependency to `Cargo.toml`
- [x] Verify `npm run tauri dev` launches successfully
- [x] Create theme CSS variables in `src/lib/styles/themes.css`
- [x] Move project docs into `docs/` folder, keep `CLAUDE.md` in root

---

## Phase 1: Minimum Viable Browser

### 1.1 Window & Multi-Webview Foundation

- [x] Set up main window with custom title bar (no native decorations)
- [x] Create browser UI webview (SvelteKit) positioned at top of window
- [x] Create initial content webview positioned below UI chrome
- [x] Implement window resize handling (UI webview width, content webview fills remaining space)
- [x] Add custom window control buttons (minimise, maximise, close) in title bar
- [x] Implement window dragging via `startDragging()` (note: `data-tauri-drag-region` doesn't work on child webviews)
- [x] Handle double-click on title bar for maximise/restore

### 1.2 Tab System (Backend)

- [x] Create `src-tauri/src/commands/tabs.rs`
- [x] Implement `tab_create` command (async — creates new webview with unique label)
- [x] Implement `tab_close` command (async — destroys webview)
- [x] Implement `tab_set_active` command (show/hide webviews)
- [x] Implement `tab_get_all` command
- [x] Implement `tab_reorder` command
- [x] Implement `tab_duplicate` command
- [x] Set up tab ID counter (atomic u64)
- [x] Register all tab commands in `lib.rs`
- [x] Attach `on_page_load` event handler to content webviews
- [x] Title detection via JS injection + MutationObserver (on_document_title_changed not available on child webviews)
- [x] Emit `tab_updated` events to browser UI webview
- [x] Handle `on_new_window` (target="_blank" links → new tab)

### 1.3 Tab System (Frontend)

- [x] Create `src/lib/stores/tabs.js` — tab state store
- [x] Listen for `tab_updated`, `tab_created`, `tab_closed` events from backend
- [x] Create `TabBar.svelte` component
- [x] Create `Tab.svelte` component (shows favicon, title, close button, loading indicator)
- [x] Implement tab click to switch active tab
- [x] Implement new tab button (+)
- [x] Implement tab close button (×)
- [x] Implement tab drag-and-drop reordering
- [x] Show loading spinner on tabs while page loads
- [x] Display favicon on tabs
- [x] Truncate long tab titles with ellipsis
- [x] Ctrl+T → new tab, Ctrl+W → close tab keyboard shortcuts (via global-shortcut plugin)

### 1.4 Navigation

- [x] Create `src-tauri/src/commands/navigation.rs`
- [x] Implement `navigate_to` command (navigates content webview to URL)
- [x] Implement `navigate_back` command
- [x] Implement `navigate_forward` command
- [x] Implement `navigate_refresh` command
- [x] Implement `navigate_stop` command
- [x] Track navigation state per tab (can_go_back, can_go_forward)
- [x] Emit navigation state with `tab_updated` events

### 1.5 Address Bar

- [x] Create `AddressBar.svelte` component
- [x] Show current URL of active tab
- [x] Detect search queries vs URLs on Enter
- [x] Format search queries → `https://google.com/search?q=...`
- [x] Auto-prepend `https://` to bare domains
- [x] Select all text on focus (Ctrl+L)
- [x] Show security icon (padlock for HTTPS)
- [ ] URL suggestions/autocomplete (stretch — can be Phase 2)

### 1.6 Navigation Controls

- [x] Create `NavigationControls.svelte` component
- [x] Back button (disabled when can't go back)
- [x] Forward button (disabled when can't go forward)
- [x] Refresh/Stop button (toggles based on loading state)
- [x] Home button (navigates to Google)

### 1.7 Keyboard Shortcuts

- [x] Set up global shortcuts via `tauri-plugin-global-shortcut` (works even when content webview focused)
- [x] Implement core shortcuts:
  - [x] `Ctrl+T` — new tab
  - [x] `Ctrl+W` — close tab
  - [x] `Ctrl+Tab` — next tab
  - [x] `Ctrl+Shift+Tab` — previous tab
  - [x] `Ctrl+L` — focus address bar
  - [x] `Ctrl+R` / `F5` — refresh
  - [x] `Alt+Left` — back
  - [x] `Alt+Right` — forward
  - [x] `Ctrl+1-9` — switch to tab by index
  - [x] `Ctrl+F` — find in page
  - [x] `Ctrl+Shift+T` — reopen last closed tab (currently just opens new tab)

### 1.8 Context Menus

- [x] Create `ContextMenu.svelte` component (reusable, for future use)
- [x] Right-click on tab → close, close others, close to right, duplicate (native popup window approach — separate borderless always-on-top window, auto-closes on blur/move/resize)
- [ ] Link context menu handling (open in new tab, copy URL) — deferred to Phase 2

### 1.9 Status Bar

- [x] Create `StatusBar.svelte` component
- [x] Show link URL on hover (via JS injection in content webviews)
- [x] Show loading progress text

### 1.10 Error & Loading States

- [ ] Create custom error page for failed navigations (deferred — WebView2 shows built-in error pages)
- [x] Create loading state UI (loading spinner on tabs, "Loading..." in status bar)
- [ ] Handle webview crashes gracefully (deferred — Tauri v2 API limitations)

---

## Phase 2: Essential Features

Implementation order follows dependency chain. Each feature builds on the previous.

### 2.0 Database Foundation

- [x] Create `src-tauri/src/storage/database.rs` — `Database` struct wrapping `Mutex<Connection>`
- [x] Implement schema init with migrations via `PRAGMA user_version`
- [x] Create all tables: bookmarks, history, settings, permissions, autofill_profiles
- [x] Update `src-tauri/src/storage/mod.rs` — declare submodules
- [x] Update `src-tauri/src/lib.rs` — open DB at `{app_data_dir}/default/browser.db`, manage as state
- [x] Write unit tests for database init and migration

### 2.1 Settings

- [x] Create `src-tauri/src/storage/settings.rs` — get/set/get_all + seed defaults
- [x] Create `src-tauri/src/commands/settings.rs` — `settings_get`, `settings_set`, `settings_get_all`
- [x] Register settings commands in `lib.rs` and `commands/mod.rs`
- [x] Create `src/lib/stores/settings.js` — writable store with IPC
- [x] Create `src/routes/settings/+page.svelte` — settings UI (General, Search, Appearance, Privacy)
- [x] Wire up `aero://settings` navigation (detect in `navigate_to` → `WebviewUrl::App("/settings")`)
- [x] Update `src/lib/utils/url.js` — handle `aero://` URLs in `isValidUrl`, `resolveInput`, `displayUrl`
- [x] Update address bar to display `aero://settings` for internal pages
- [x] Write unit tests for settings storage

### 2.2 History

- [ ] Create `src-tauri/src/storage/history.rs` — `add_visit` (upsert), `search`, `get_recent`, `delete`, `clear(timeframe)`
- [ ] Create `src-tauri/src/commands/history.rs` — IPC commands
- [ ] Register history commands in `lib.rs` and `commands/mod.rs`
- [ ] Hook `add_visit()` into `on_page_load` Finished handler in `tabs.rs` (skip `aero://`, `about:blank`, incognito)
- [ ] Create `src/lib/stores/history.js` — search, getRecent, delete, clear
- [ ] Create `src/routes/history/+page.svelte` — search bar, date-grouped list, clear data button
- [ ] Wire up `aero://history` navigation
- [ ] Add `Ctrl+H` global shortcut to open history
- [ ] Address bar autocomplete — query history as user types, show dropdown suggestions
- [ ] Write unit tests for history storage

### 2.3 Bookmarks

- [ ] Create `src-tauri/src/storage/bookmarks.rs` — CRUD, tree ops, search, import/export HTML
- [ ] Create `src-tauri/src/commands/bookmarks.rs` — IPC commands + `bookmark_is_bookmarked(url)`
- [ ] Register bookmark commands in `lib.rs` and `commands/mod.rs`
- [ ] Seed root folders on first run: "Bookmarks Bar", "Other Bookmarks"
- [ ] Create `src/lib/stores/bookmarks.js` — bar items, add/remove/toggle
- [ ] Create `src/lib/components/BookmarkBar.svelte` — horizontal bar below toolbar
- [ ] Implement folder dropdowns using popup window pattern
- [ ] Add star icon to `AddressBar.svelte` — filled/outline to toggle bookmark
- [ ] Create `src/routes/bookmarks/+page.svelte` — full tree manager with search, import/export
- [ ] Wire up `aero://bookmarks` navigation
- [ ] Make CHROME_HEIGHT dynamic — store in Tauri state, push updates via events when bookmarks bar toggles
- [ ] Update `tab_resize_all` to read dynamic chrome height
- [ ] Add `Ctrl+D` shortcut to bookmark current page
- [ ] Add `Ctrl+Shift+B` shortcut to toggle bookmarks bar
- [ ] Write unit tests for bookmark storage

### 2.4 Multiple Windows

- [ ] Create `src-tauri/src/commands/windows.rs` — `window_create`, `window_close`
- [ ] Add `window_id` field to `TabInfo` in `tab_state.rs`
- [ ] Refactor `active_tab` to per-window `HashMap<String, Option<String>>`
- [ ] Extract window creation into reusable `create_browser_window(app, id)` in `lib.rs`
- [ ] Update tab commands to accept/use `window_id`
- [ ] Update `src/lib/stores/tabs.js` — filter tabs by window_id
- [ ] Pass window_id to each UI webview via query param
- [ ] Handle window close — close all child tab webviews, exit app when last window closes
- [ ] Add `Ctrl+N` global shortcut for new window
- [ ] Multi-window state sync: Rust emits global events for shared data changes

### 2.5 Certificate/Security Info

- [ ] Create `src-tauri/src/commands/security.rs` — `security_get_info(url)` fetches TLS cert info
- [ ] Add `reqwest` with `rustls-tls` feature to `Cargo.toml`
- [ ] Register security commands in `lib.rs` and `commands/mod.rs`
- [ ] Click padlock in address bar → popup window showing connection type, cert issuer, validity
- [ ] Enhance padlock icon states: secure, insecure, mixed content

### 2.6 Incognito/Private Mode (depends on 2.4)

- [ ] Extend `window_create` with `incognito: bool` parameter
- [ ] Incognito tab webviews use `.incognito(true)` on `WebviewBuilder`
- [ ] Add `is_incognito: bool` to `TabInfo`
- [ ] Skip history recording for incognito tabs in `on_page_load`
- [ ] Visual differentiation — darker tab bar, "Private" label in incognito windows
- [ ] Add `Ctrl+Shift+N` shortcut for new incognito window
- [ ] All data discarded on window close (WebView2 handles automatically)

### 2.7 Permissions

- [ ] Add `permissions` table to DB schema (origin, permission type, state)
- [ ] Create `src-tauri/src/storage/permissions.rs` — CRUD per origin+permission
- [ ] Create `src-tauri/src/commands/permissions.rs` — IPC commands
- [ ] Register permissions commands in `lib.rs` and `commands/mod.rs`
- [ ] Implement permission prompt popup when site requests camera/mic/location/notifications
- [ ] Remember decisions per-origin in DB
- [ ] Add permissions management section to settings page
- [ ] Write unit tests for permissions storage

### 2.8 Autofill

- [ ] Add `autofill_profiles` table to DB schema
- [ ] Create `src-tauri/src/storage/autofill.rs` — CRUD with encryption (aes-gcm or Windows DPAPI)
- [ ] Add `aes-gcm`, `rand` crates to `Cargo.toml`
- [ ] Create `src-tauri/src/commands/autofill.rs` — IPC commands + `autofill_get_suggestions(field_type)`
- [ ] Register autofill commands in `lib.rs` and `commands/mod.rs`
- [ ] Extend JS injection in `tabs.rs` to detect `autocomplete` inputs
- [ ] Show autofill dropdown on input focus (injected DOM in content webview)
- [ ] Add autofill management section to settings page
- [ ] Write unit tests for autofill storage

---

## Phase 3: Profiles & Google Account

_Tasks to be detailed when Phase 2 is complete._

---

## Phase 4: Ad Blocking

_Tasks to be detailed when Phase 3 is complete._

---

## Phase 5: Polish & Power User Features

_Tasks to be detailed when Phase 4 is complete._

---

## Notes & Blockers

_Record any issues, decisions, or blockers here as the project progresses._

| Date | Note |
|------|------|
| 2026-02-12 | Tab context menu uses a separate borderless popup window (not DOM overlay) — WebView2 always renders content webviews on top of UI webview, so the only way to float UI on top of everything is a separate OS window. Menu items use `onclick` + `window.location` navigation intercepted by `on_navigation` (no `__TAURI_INTERNALS__` needed on `about:blank`). Auto-closes on focus loss, main window move/resize. |
| 2026-02-12 | Escape key removed from global shortcuts — was hijacking Escape system-wide. Now uses local `keydown` listener in UI webview only. |
