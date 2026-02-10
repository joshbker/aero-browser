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
- [ ] Implement `tab_reorder` command
- [ ] Implement `tab_duplicate` command
- [x] Set up tab ID counter (atomic u64)
- [x] Register all tab commands in `lib.rs`
- [x] Attach `on_page_load` event handler to content webviews
- [x] Title detection via JS injection + MutationObserver (on_document_title_changed not available on child webviews)
- [x] Emit `tab_updated` events to browser UI webview
- [ ] Handle `on_new_window` (target="_blank" links → new tab)

### 1.3 Tab System (Frontend)

- [x] Create `src/lib/stores/tabs.js` — tab state store
- [x] Listen for `tab_updated`, `tab_created`, `tab_closed` events from backend
- [x] Create `TabBar.svelte` component
- [x] Create `Tab.svelte` component (shows favicon, title, close button, loading indicator)
- [x] Implement tab click to switch active tab
- [x] Implement new tab button (+)
- [x] Implement tab close button (×)
- [ ] Implement tab drag-and-drop reordering
- [x] Show loading spinner on tabs while page loads
- [ ] Display favicon on tabs
- [x] Truncate long tab titles with ellipsis
- [x] Ctrl+T → new tab, Ctrl+W → close tab keyboard shortcuts (via global-shortcut plugin)

### 1.4 Navigation

- [x] Create `src-tauri/src/commands/navigation.rs`
- [x] Implement `navigate_to` command (navigates content webview to URL)
- [x] Implement `navigate_back` command
- [x] Implement `navigate_forward` command
- [x] Implement `navigate_refresh` command
- [x] Implement `navigate_stop` command
- [ ] Track navigation state per tab (can_go_back, can_go_forward)
- [ ] Emit `navigation_state_changed` events

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
- [x] Back button
- [x] Forward button
- [x] Refresh/Stop button (toggles based on loading state)
- [ ] Home button (navigates to homepage setting)

### 1.7 Keyboard Shortcuts

- [x] Set up global shortcuts via `tauri-plugin-global-shortcut` (works even when content webview focused)
- [x] Implement core shortcuts:
  - [x] `Ctrl+T` — new tab
  - [x] `Ctrl+W` — close tab
  - [ ] `Ctrl+Tab` — next tab
  - [ ] `Ctrl+Shift+Tab` — previous tab
  - [x] `Ctrl+L` — focus address bar
  - [x] `Ctrl+R` / `F5` — refresh
  - [ ] `Alt+Left` — back
  - [ ] `Alt+Right` — forward
  - [ ] `Ctrl+1-9` — switch to tab by index
  - [ ] `Ctrl+F` — find in page (placeholder for now)
  - [ ] `Ctrl+Shift+T` — reopen last closed tab (stretch)

### 1.8 Context Menus

- [ ] Create `ContextMenu.svelte` component
- [ ] Right-click on tab → close, close others, close to right, duplicate, pin (placeholder)
- [ ] Link context menu handling (open in new tab, copy URL) — may need webview-level interception

### 1.9 Status Bar

- [ ] Create `StatusBar.svelte` component
- [ ] Show link URL on hover (requires webview hover event or JS injection)
- [ ] Show loading progress text

### 1.10 Error & Loading States

- [ ] Create custom error page for failed navigations
- [ ] Create loading state UI
- [ ] Handle webview crashes gracefully

---

## Phase 2: Essential Features

_Tasks to be detailed when Phase 1 is complete._

- [ ] Bookmarks system
- [ ] Browsing history with search
- [ ] Settings page (`aero://settings`)
- [ ] Multiple windows (Ctrl+N)
- [ ] Private/incognito mode
- [ ] Print support
- [ ] Site permissions management
- [ ] Certificate/security info display
- [ ] Basic autofill

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
| | |
