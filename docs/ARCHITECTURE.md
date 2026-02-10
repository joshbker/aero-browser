# Architecture Decisions — Aero Browser

## ADR-001: Multi-Webview Tab Architecture

### Context

A browser needs to render multiple web pages (tabs) simultaneously, each with isolated sessions, cookies, and browsing contexts.

### Decision

Each browser tab is a separate Tauri webview instance. The browser chrome (tab bar, address bar, navigation controls, etc.) lives in the main/primary webview running the SvelteKit app. Content webviews are created and destroyed dynamically as tabs open and close.

### Reasoning

- **Isolation**: each webview has its own browsing context — cookies, localStorage, sessions don't leak between tabs
- **Performance**: inactive tabs can be suspended/hibernated without affecting the UI
- **Stability**: a crashed tab (webview) doesn't take down the whole browser
- **WebView2 native**: this is how WebView2 is designed to be used — multiple webview instances per window

### Consequences

- Need to manage webview lifecycle in Rust (create, destroy, show/hide, resize, z-order)
- IPC becomes more complex — main UI webview communicates with Rust, which manages content webviews
- Memory usage scales with tab count (each webview has baseline memory overhead)
- Need careful z-index/visibility management to show only the active tab's webview

### Implementation Notes

```
┌─────────────────────────────────────────────┐
│ Tauri Window                                │
│ ┌─────────────────────────────────────────┐ │
│ │ Main Webview (SvelteKit UI)             │ │
│ │ ┌─────────────────────────────────────┐ │ │
│ │ │ Tab Bar  │ + │                       │ │ │
│ │ ├─────────────────────────────────────┤ │ │
│ │ │ ← → ↻  │ URL Bar           │ ⋮ │   │ │ │
│ │ └─────────────────────────────────────┘ │ │
│ ├─────────────────────────────────────────┤ │
│ │ Content Webview (active tab)            │ │
│ │                                         │ │
│ │         (web page renders here)         │ │
│ │                                         │ │
│ │                                         │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

The main webview occupies the top portion (tab bar + toolbar). Content webviews are positioned below and fill the remaining space. Only the active tab's webview is visible — others are hidden or suspended.

### Chrome Height Constant

The browser chrome height (tab bar + toolbar) is defined as a shared constant:
- **Rust**: `const CHROME_HEIGHT: f64 = 76.0;` in `src-tauri/src/commands/tabs.rs`
- **Frontend**: `export const CHROME_HEIGHT = 76` in `src/lib/utils/constants.js`

If the UI layout changes (e.g. adding a bookmarks bar), update both values. The breakdown is:
- Tab bar: 36px
- Toolbar: 40px
- Total: 76px

### `target="_blank"` / `window.open()` Behaviour

Links with `target="_blank"` and `window.open()` calls from content webviews open a **new tab** (not the system browser). This matches Chrome/Firefox behaviour. Implemented via the `on_new_window` handler on each content webview.

### Component Library

No external component library (shadcn-svelte etc). All browser chrome components are custom-built with Tailwind CSS, taking design cues from shadcn's aesthetic (clean, minimal, neutral palette).

---

## ADR-002: State Management Strategy

### Context

The browser has state that lives in different places: Rust (authoritative state, persisted data), Svelte stores (UI state, reactive updates), and individual webviews (page-specific state).

### Decision

Three-tier state management:

1. **Rust (source of truth)**: all persisted data (bookmarks, history, settings, profiles), tab registry, download state
2. **Svelte stores (UI mirror)**: reactive copies of Rust state for the UI, plus UI-only state (which context menu is open, sidebar visibility, etc.)
3. **Webview state (page-level)**: managed by WebView2 internally — URL, scroll position, form data, etc.

### Data Flow

```
User action → Svelte component → Svelte store → invoke() → Rust command → Rust state/DB
                                                                    ↓
                                                              emit event
                                                                    ↓
Svelte store ← listen() ← Tauri event ← ← ← ← ← ← ← ← ← ← ← ←
     ↓
UI updates reactively
```

### Rules

- **Never** store persisted data only in Svelte — it must go through Rust to SQLite
- Svelte stores can have optimistic updates (update UI immediately, sync to Rust async) but must handle failures
- Rust emits events for state changes that other parts of the UI might care about
- Avoid duplicating state — if Rust owns it, Svelte should mirror it, not maintain a separate copy

---

## ADR-003: SQLite for Local Storage

### Context

Need persistent storage for bookmarks, history, settings, downloads, profiles.

### Decision

Use SQLite via the `rusqlite` crate. One database file per profile, stored in Tauri's app data directory.

### Reasoning

- Embedded — no external database server
- Fast for the volume of data a browser generates
- ACID transactions for data integrity
- Excellent Rust support via `rusqlite`
- Easy to backup (it's just a file)
- Schema migrations are straightforward

### File Structure

```
{appDataDir}/
├── profiles/
│   ├── default/
│   │   ├── browser.db      # Main database (bookmarks, history, etc.)
│   │   └── favicons.db     # Favicon cache (separate for easy clearing)
│   └── work/
│       ├── browser.db
│       └── favicons.db
└── global.db               # Profile registry, global settings
```

---

## ADR-004: Ad Blocking Strategy

### Context

Built-in ad blocking is a core feature. Need to intercept and block network requests before they reach the page.

### Decision

Use the `adblock` Rust crate (from Brave's adblock-rust) for filter list parsing and URL matching. Intercept requests at the Tauri/WebView2 level.

### How It Works

1. On startup, load filter lists (EasyList, EasyPrivacy, etc.) into the adblock engine
2. WebView2 supports `add_WebResourceRequested` event — intercept all requests
3. For each request, check URL against the adblock engine
4. Block matched requests by returning an empty response
5. For cosmetic filtering, inject CSS rules into pages to hide ad elements

### Filter List Management

- Ship with default filter lists bundled in the binary
- Check for updates periodically (daily)
- Store downloaded lists in app data directory
- Allow users to add custom filter list URLs

### Performance Considerations

- The adblock engine builds a highly optimised data structure (bloom filters + hash maps) — lookups are O(1) to O(log n)
- Engine initialisation takes ~100-200ms with full filter lists — do this on startup in background
- Memory usage is ~50-100MB for full filter lists — acceptable for a desktop app

---

## ADR-005: Internal Pages (aero:// protocol)

### Context

Need custom pages for new tab, settings, history, bookmarks, etc.

### Decision

Register a custom `aero://` protocol handler in Tauri. Internal pages are served from bundled HTML/JS/CSS assets.

### Pages

| URL | Purpose |
|-----|---------|
| `aero://newtab` | New tab page |
| `aero://settings` | Settings page |
| `aero://history` | History page |
| `aero://bookmarks` | Bookmark manager |
| `aero://downloads` | Downloads page |
| `aero://about` | About/version info |

### Implementation

Internal pages can either be:
1. **Part of the SvelteKit app** — rendered in the main webview, hiding the content webview
2. **Separate HTML files** — loaded in the content webview via the custom protocol

Option 1 is simpler and allows internal pages to use the same component library and stores. The content webview area simply shows a SvelteKit route instead of a web page.

---

## ADR-006: Keyboard Shortcut System

### Context

Browsers have dozens of keyboard shortcuts. Need a centralised, extensible system.

### Decision

Define all shortcuts in a single configuration file (`src/lib/utils/keybindings.js`). Use a global keyboard event listener that maps key combos to actions.

### Structure

```javascript
// Shortcuts defined as { keys, action, context }
// context: 'global' (works everywhere), 'addressbar' (when focused), etc.

export const keybindings = [
    { keys: 'ctrl+t', action: 'tab:create', context: 'global' },
    { keys: 'ctrl+w', action: 'tab:close', context: 'global' },
    { keys: 'ctrl+tab', action: 'tab:next', context: 'global' },
    { keys: 'ctrl+shift+tab', action: 'tab:previous', context: 'global' },
    { keys: 'ctrl+l', action: 'addressbar:focus', context: 'global' },
    { keys: 'ctrl+f', action: 'find:open', context: 'global' },
    { keys: 'escape', action: 'find:close', context: 'find' },
    { keys: 'ctrl+shift+delete', action: 'history:clear', context: 'global' },
    // ...
]
```

Actions are dispatched through a central action handler that routes to the appropriate store method or command.

---

## ADR-007: Why Not TypeScript

### Context

TypeScript is the default for many modern web projects.

### Decision

Use plain JavaScript for the SvelteKit frontend.

### Reasoning

- Developer preference — faster iteration, less boilerplate
- SvelteKit works great with JS — Svelte's reactivity model + JSDoc type hints provide enough safety
- Tauri's IPC is stringly-typed anyway — TypeScript types for invoke calls would be manually maintained and potentially misleading
- One less build step, simpler toolchain
- JSDoc comments can provide type hints where useful without requiring TypeScript compilation

### Trade-offs

- No compile-time type checking — rely on good naming, small functions, and testing
- IDE autocomplete is slightly less precise — mitigated by JSDoc where it matters
- Contributors familiar with TypeScript may need to adjust

---

## ADR-008: Window Chrome & Custom Title Bar

### Context

Need a custom title bar to integrate the tab bar and browser controls seamlessly.

### Decision

Use Tauri's `decorations: false` to remove the native title bar, then implement a custom title bar in SvelteKit with drag regions and window control buttons (minimise, maximise, close).

### Implementation

- Set `decorations: false` in `tauri.conf.json`
- Add `data-tauri-drag-region` attribute to the title bar area for window dragging
- Implement custom window control buttons that call Tauri's window API
- Handle double-click on drag region for maximise/restore
- Ensure the tab bar is part of the title bar area to maximise vertical space (Chrome-style)

### Layout

```
┌──────────────────────────────────────────────────────┐
│ [Tab1] [Tab2] [Tab3] [+]              [—] [□] [✕]  │  ← Custom title bar + tabs
│ [←] [→] [↻]  [🔒 https://example.com       ]  [⋮] │  ← Toolbar
├──────────────────────────────────────────────────────┤
│                                                      │
│                  Web Content                         │
│                                                      │
└──────────────────────────────────────────────────────┘
```
