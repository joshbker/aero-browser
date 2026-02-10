# Aero Browser

A minimal, privacy-focused web browser built with Tauri v2 + SvelteKit + WebView2. Zero bloat, just browsing.

## Vision

A daily-driver browser that does what a browser should do — render web pages fast, block ads, and stay out of your way. No crypto wallets, no AI assistants, no news feeds, no rewards programs. Just tabs, a URL bar, and the web.

Built on WebView2 (via Tauri) so the rendering engine stays updated automatically through Edge, with a custom SvelteKit UI shell for everything else.

## Core Principles

- **No bloat** — if it's not essential for browsing, it doesn't ship
- **Privacy-first** — sensible defaults, no telemetry, no tracking
- **Fast** — instant startup, minimal memory overhead beyond the webview
- **Familiar** — standard browser UX, no learning curve

---

## Feature Roadmap

### Phase 1: Minimum Viable Browser

The absolute basics to make it usable as a browser.

- [ ] **Window management** — single window with proper title bar
- [ ] **URL/search bar** — type URLs or search queries (configurable search engine, default Google)
- [ ] **Navigation controls** — back, forward, refresh, stop, home
- [ ] **Tab system** — create, close, switch, reorder, duplicate tabs
- [ ] **Page rendering** — WebView2 handles this, but we need proper loading states, error pages, favicon fetching
- [ ] **Context menus** — right-click menus for links (open in new tab, copy URL), text (copy, paste, search), page (view source, inspect)
- [ ] **Keyboard shortcuts** — standard browser shortcuts (Ctrl+T, Ctrl+W, Ctrl+L, Ctrl+Tab, etc.)
- [ ] **Find in page** — Ctrl+F search overlay
- [ ] **Zoom** — Ctrl+/- and Ctrl+scroll zoom per tab
- [ ] **Download manager** — basic download handling with progress, open file, open folder
- [ ] **Status bar** — show link preview on hover, loading status

### Phase 2: Essential Features

Things you'd miss within a day of daily driving.

- [ ] **Bookmarks** — add, remove, folders, bookmarks bar, import/export
- [ ] **History** — browsing history with search, clear history options
- [ ] **Settings page** — internal `aero://settings` page for configuration
- [ ] **Multiple windows** — Ctrl+N for new windows
- [ ] **Incognito/private mode** — separate session, no history, no cookies persisted
- [ ] **Print** — Ctrl+P print dialog
- [ ] **Permissions** — camera, microphone, location, notifications per-site management
- [ ] **Certificate/security info** — padlock icon, HTTPS status, cert details
- [ ] **Autofill** — basic form autofill for addresses, credit cards (stored locally, encrypted)

### Phase 3: Google Account & Profile Support

Multi-profile support tied to Google accounts.

- [ ] **Browser profiles** — separate profiles with own bookmarks, history, cookies, settings
- [ ] **Profile switcher** — quick switch UI in the title bar
- [ ] **Google account sign-in** — sign into Google services within a profile
- [ ] **Profile data isolation** — each profile is fully sandboxed
- [ ] **Profile customisation** — name, colour/avatar per profile
- [ ] **Import from Chrome/Brave** — import bookmarks, history, passwords from other browsers

### Phase 4: Ad Blocking & Content Filtering

Built-in content blocking without needing extensions.

- [ ] **Ad blocker engine** — integrate a content blocking engine (look into [adblock-rust](https://github.com/nicbarker/nicbarker/nicbarker) or similar Rust-native blocker)
- [ ] **Filter lists** — support EasyList, EasyPrivacy, uBlock Origin filter lists
- [ ] **Per-site toggle** — easily disable blocking for specific sites
- [ ] **Custom rules** — user-defined block/allow rules
- [ ] **Cosmetic filtering** — hide ad elements from the DOM, not just block requests
- [ ] **Block counter** — show number of blocked items per page in URL bar
- [ ] **HTTPS Everywhere** — auto-upgrade HTTP to HTTPS where possible
- [ ] **Tracker blocking** — block known tracking scripts and pixels

### Phase 5: Polish & Power User Features

Nice-to-haves that make it feel complete.

- [ ] **Extension support** — investigate WebView2 extension API support (limited but possible)
- [ ] **Custom themes** — light/dark mode, accent colours
- [ ] **Tab pinning** — pin tabs to the left
- [ ] **Tab grouping** — colour-coded tab groups
- [ ] **Reader mode** — strip page to just content
- [ ] **Picture-in-picture** — floating video player
- [ ] **DevTools** — WebView2 supports opening Edge DevTools
- [ ] **Startup options** — continue where you left off, specific pages, new tab
- [ ] **Custom new tab page** — minimal, fast, customisable
- [ ] **Command palette** — Ctrl+Shift+P quick actions (like VS Code)
- [ ] **Split view** — two webviews side by side
- [ ] **Vertical tabs** — optional sidebar tab layout

---

## Architecture Overview

```
aero-browser/
├── src-tauri/          # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs            # Entry point, window setup
│   │   ├── lib.rs             # Tauri command exports
│   │   ├── commands/          # Tauri IPC commands
│   │   │   ├── mod.rs
│   │   │   ├── navigation.rs  # URL loading, back/forward/refresh
│   │   │   ├── tabs.rs        # Tab lifecycle management
│   │   │   ├── bookmarks.rs   # Bookmark CRUD
│   │   │   ├── history.rs     # History storage/search
│   │   │   ├── downloads.rs   # Download management
│   │   │   ├── profiles.rs    # Profile management
│   │   │   ├── settings.rs    # Settings read/write
│   │   │   └── adblock.rs     # Content blocking
│   │   ├── state/             # App state management
│   │   │   ├── mod.rs
│   │   │   ├── app_state.rs   # Global app state
│   │   │   └── tab_state.rs   # Per-tab state
│   │   ├── storage/           # Persistent storage
│   │   │   ├── mod.rs
│   │   │   ├── database.rs    # SQLite via rusqlite
│   │   │   ├── bookmarks.rs   # Bookmark storage
│   │   │   ├── history.rs     # History storage
│   │   │   └── settings.rs    # Settings storage
│   │   └── adblock/           # Ad blocking engine
│   │       ├── mod.rs
│   │       ├── engine.rs      # Filter list parsing/matching
│   │       └── lists.rs       # Filter list management
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                # SvelteKit frontend (browser UI)
│   ├── lib/
│   │   ├── components/        # Svelte components
│   │   │   ├── TabBar.svelte         # Tab strip
│   │   │   ├── Tab.svelte            # Individual tab
│   │   │   ├── AddressBar.svelte     # URL/search input
│   │   │   ├── NavigationControls.svelte  # Back/fwd/refresh buttons
│   │   │   ├── BookmarkBar.svelte    # Bookmarks toolbar
│   │   │   ├── ContextMenu.svelte    # Right-click menus
│   │   │   ├── FindBar.svelte        # Ctrl+F search
│   │   │   ├── DownloadPanel.svelte  # Download progress/list
│   │   │   ├── ProfileSwitcher.svelte # Profile selector
│   │   │   ├── SettingsPage.svelte   # Settings UI
│   │   │   └── StatusBar.svelte      # Bottom status bar
│   │   ├── stores/            # Svelte stores
│   │   │   ├── tabs.js        # Tab state store
│   │   │   ├── navigation.js  # Navigation state
│   │   │   ├── bookmarks.js   # Bookmarks store
│   │   │   ├── settings.js    # Settings store
│   │   │   ├── downloads.js   # Downloads store
│   │   │   └── profiles.js    # Profiles store
│   │   ├── actions/           # Svelte actions (DOM behaviours)
│   │   │   ├── clickOutside.js
│   │   │   ├── draggable.js   # Tab drag-reorder
│   │   │   └── shortcut.js    # Keyboard shortcut handler
│   │   ├── utils/             # Utility functions
│   │   │   ├── url.js         # URL parsing, validation, search query detection
│   │   │   ├── favicon.js     # Favicon fetching/caching
│   │   │   ├── keybindings.js # Keyboard shortcut definitions
│   │   │   └── ipc.js         # Tauri IPC wrapper helpers
│   │   └── styles/            # Global styles
│   │       ├── app.css        # Global CSS, Tailwind imports
│   │       └── themes.css     # Theme variables
│   ├── routes/
│   │   ├── +layout.svelte     # Root layout (tab bar + chrome)
│   │   └── +page.svelte       # Main browser view
│   └── app.html
├── static/
│   └── icons/                 # App icons, UI icons
├── docs/                      # Project documentation
│   ├── PROJECT.md             # This file
│   ├── CLAUDE.md              # AI agent instructions
│   ├── STYLE.md               # Code style guide
│   └── ARCHITECTURE.md        # Detailed architecture decisions
├── package.json
├── svelte.config.js
├── tailwind.config.js
├── vite.config.js
└── README.md
```

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Rendering engine | WebView2 (via Edge) | Page rendering, JS execution |
| App framework | Tauri v2 | Native window, IPC, system access |
| Backend language | Rust | Performance, safety, Tauri requirement |
| Frontend framework | SvelteKit | Browser UI shell |
| Styling | Tailwind CSS | Utility-first CSS |
| Component library | shadcn-svelte | UI components where needed |
| Local storage | SQLite (rusqlite) | Bookmarks, history, settings |
| Ad blocking | adblock-rust or custom | Content filtering engine |
| Icons | Lucide | Consistent icon set |

---

## Data Storage

All data stored locally using SQLite. One database per profile.

### Database Schema (initial)

```sql
-- Bookmarks
CREATE TABLE bookmarks (
    id TEXT PRIMARY KEY,
    parent_id TEXT REFERENCES bookmarks(id),
    title TEXT NOT NULL,
    url TEXT,                    -- NULL for folders
    is_folder BOOLEAN DEFAULT FALSE,
    position INTEGER NOT NULL,  -- Sort order within parent
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- History
CREATE TABLE history (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    title TEXT,
    visit_count INTEGER DEFAULT 1,
    last_visited DATETIME DEFAULT CURRENT_TIMESTAMP,
    first_visited DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_history_url ON history(url);
CREATE INDEX idx_history_last_visited ON history(last_visited DESC);

-- Downloads
CREATE TABLE downloads (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL,
    filesize INTEGER,
    state TEXT DEFAULT 'in_progress', -- in_progress, completed, cancelled, failed
    progress REAL DEFAULT 0,
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

-- Settings (key-value)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Profiles
CREATE TABLE profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    avatar TEXT,           -- emoji or colour code
    colour TEXT,           -- accent colour hex
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_default BOOLEAN DEFAULT FALSE
);
```

---

## IPC Contract

Communication between the Svelte frontend and Rust backend uses Tauri's invoke system.

### Command Naming Convention

Commands follow `entity_action` pattern: `tab_create`, `bookmark_add`, `history_search`, etc.

### Core Commands (Phase 1)

```
// Navigation
navigate_to(tab_id, url) -> Result<(), Error>
navigate_back(tab_id) -> Result<(), Error>
navigate_forward(tab_id) -> Result<(), Error>
navigate_refresh(tab_id) -> Result<(), Error>
navigate_stop(tab_id) -> Result<(), Error>

// Tabs
tab_create(url?) -> Result<TabInfo, Error>
tab_close(tab_id) -> Result<(), Error>
tab_get_all() -> Result<Vec<TabInfo>, Error>
tab_set_active(tab_id) -> Result<(), Error>
tab_reorder(tab_id, new_index) -> Result<(), Error>
tab_duplicate(tab_id) -> Result<TabInfo, Error>

// Downloads
download_get_all() -> Result<Vec<DownloadInfo>, Error>
download_cancel(download_id) -> Result<(), Error>
download_open_file(download_id) -> Result<(), Error>
download_open_folder(download_id) -> Result<(), Error>

// Settings
settings_get(key) -> Result<String, Error>
settings_set(key, value) -> Result<(), Error>
settings_get_all() -> Result<HashMap<String, String>, Error>
```

### Events (Backend -> Frontend)

```
tab_updated { tab_id, title?, url?, loading?, favicon? }
tab_created { tab_id, url }
tab_closed { tab_id }
download_progress { download_id, progress, state }
download_complete { download_id, filepath }
navigation_state_changed { tab_id, can_go_back, can_go_forward }
```

---

## Default Settings

```json
{
    "search_engine": "https://google.com/search?q=",
    "homepage": "aero://newtab",
    "new_tab_page": "aero://newtab",
    "restore_on_startup": "new_tab",
    "theme": "system",
    "show_bookmarks_bar": true,
    "show_status_bar": true,
    "adblock_enabled": true,
    "https_upgrade": true,
    "default_zoom": 100,
    "download_path": "~/Downloads",
    "ask_download_location": false
}
```
