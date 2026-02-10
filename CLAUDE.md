# CLAUDE.md — Agent Instructions for Aero Browser

## Project Overview

Aero is a minimal, privacy-focused web browser built with Tauri v2 (Rust backend) + SvelteKit (frontend UI) + WebView2 (rendering engine). The goal is a zero-bloat daily driver browser.

**Read these docs before starting any work:**

- `docs/PROJECT.md` — feature roadmap, architecture overview, IPC contract, database schema
- `docs/STYLE.md` — code formatting, naming conventions, patterns
- `docs/ARCHITECTURE.md` — key architectural decisions and reasoning
- `docs/MULTIWEBVIEW.md` — **CRITICAL** — concrete code examples for the multi-webview tab system (read this before touching anything tab-related)
- `docs/CONFIG.md` — expected configuration for tauri.conf.json, Cargo.toml, svelte.config.js, vite.config.js, tailwind.config.js
- `docs/TASKS.md` — current task tracker, update as you complete work

---

## Critical Context

### This is a Tauri v2 project, NOT Tauri v1

Tauri v2 has significant API differences from v1. Key things to remember:

- Import from `@tauri-apps/api` v2 — the API surface has changed
- Use `@tauri-apps/plugin-*` packages for functionality that moved to plugins (shell, dialog, fs, etc.)
- Window management uses `@tauri-apps/api/window` with the new v2 API
- IPC uses `invoke` from `@tauri-apps/api/core`
- Events use `listen`/`emit` from `@tauri-apps/api/event`
- Check https://v2.tauri.app/start/ for v2 docs — do NOT reference v1 docs
- `tauri.conf.json` schema is different in v2

### WebView2 is handled by Tauri

We don't interact with WebView2 directly. Tauri's `WebviewWindow` abstraction handles the webview. We may need to use Tauri's webview API for multi-webview setups (one webview per tab).

### Multi-Webview Architecture

Each browser tab is a separate Tauri webview. The browser UI (tab bar, address bar, etc.) is rendered in the main webview. Content webviews are created/destroyed as tabs are opened/closed. This is a critical architectural pattern — do not try to render web content inside the SvelteKit app via iframes.

---

## Tech Stack Quick Reference

| What | Technology |
|------|-----------|
| App framework | Tauri v2 |
| Backend | Rust (2021 edition) |
| Frontend | SvelteKit (with Svelte 5 runes) |
| Styling | Tailwind CSS |
| Components | shadcn-svelte |
| Database | SQLite via rusqlite |
| Icons | Lucide (lucide-svelte) |
| Package manager | npm |
| Frontend language | JavaScript (NOT TypeScript) |

---

## Working With This Codebase

### Before Making Changes

1. Read the relevant section of `PROJECT.md` for context on what you're building
2. Check `STYLE.md` for formatting and naming conventions
3. Look at existing code patterns before introducing new ones
4. Check the IPC contract in `PROJECT.md` if adding new commands

### File Structure Conventions

- **Rust commands**: One file per domain in `src-tauri/src/commands/` (e.g., `tabs.rs`, `bookmarks.rs`)
- **Svelte components**: One component per file in `src/lib/components/`
- **Stores**: One store per domain in `src/lib/stores/`
- **Utils**: Pure functions grouped by purpose in `src/lib/utils/`

### Adding a New Feature

1. Define the IPC commands needed (if any) — add to `PROJECT.md` IPC contract
2. Implement Rust commands in appropriate `src-tauri/src/commands/` file
3. Register commands in `lib.rs`
4. Create/update Svelte stores to manage frontend state
5. Build UI components
6. Wire up IPC calls in stores or components via `src/lib/utils/ipc.js`

### Adding a New Tauri Command

```rust
// In src-tauri/src/commands/example.rs
use tauri::command;

#[command]
pub fn example_action(param: String) -> Result<String, String> {
    // Implementation
    Ok("result".to_string())
}
```

```javascript
// In Svelte, calling it:
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('example_action', { param: 'value' });
```

### Adding a New Store

```javascript
// src/lib/stores/example.js
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

function createExampleStore() {
    const { subscribe, set, update } = writable([]);

    return {
        subscribe,
        async load() {
            const data = await invoke('example_get_all');
            set(data);
        },
        async add(item) {
            const result = await invoke('example_add', { item });
            update(items => [...items, result]);
        }
    };
}

export const examples = createExampleStore();
```

---

## Common Gotchas

### Tauri v2 Specific

- `tauri::command` functions must return `Result<T, String>` or a type that implements `Serialize` — use `Result<T, String>` for simplicity, mapping errors with `.map_err(|e| e.to_string())`
- State must be managed via `app.manage(state)` in the builder — access with `tauri::State<'_, AppState>` in commands
- Async commands need `#[command(async)]` or just use `async fn`
- Window events and webview events have different APIs in v2

### SvelteKit + Tauri

- SvelteKit runs in SSG mode for Tauri — make sure `adapter-static` is configured
- All Tauri API calls (`invoke`, `listen`) only work in the browser, not during SSR — guard with `browser` check from `$app/environment` or use `onMount`
- Don't use SvelteKit's server-side features (load functions with server fetches, form actions, etc.) — there's no server in a Tauri app

### Svelte 5

- Use runes (`$state`, `$derived`, `$effect`) for component-level reactivity
- Stores still work and are preferred for shared state across components
- Props use `let { propName } = $props()` syntax
- Event handlers use `onclick` not `on:click`

### General

- All user data paths should use Tauri's `appDataDir` — never hardcode paths
- Test on Windows — this is a Windows-first project (WebView2 is Windows-only, though Tauri uses WebKit on macOS/Linux)
- Keep the main thread free — heavy operations (DB queries, file I/O, filter list parsing) should be async in Rust

---

## Testing Approach

- **Rust**: Unit tests for storage, adblock engine, URL parsing — run with `cargo test`
- **Frontend**: Manual testing primarily — the browser UI is heavily tied to Tauri's runtime so unit testing components in isolation has limited value
- **Integration**: Test IPC commands work correctly end-to-end by running the app with `npm run tauri dev`

---

## Build & Run

```bash
# Development
npm run tauri dev

# Production build
npm run tauri build

# Run Rust tests
cd src-tauri && cargo test

# Run frontend only (no Tauri — limited use)
npm run dev
```

---

## Decision Log

Track important architectural decisions here as the project evolves.

| Date | Decision | Reasoning |
|------|----------|-----------|
| TBD | Multi-webview for tabs | Each tab needs isolated browsing context, cookies, sessions |
| TBD | SQLite for storage | Lightweight, embedded, no external dependencies, good Rust support |
| TBD | Google as default search | Privacy-first default, user can change |
| TBD | JavaScript over TypeScript | Developer preference, faster iteration, less boilerplate |
| 2026-02-10 | target="_blank" opens new tab | Match Chrome/Firefox behaviour, not system browser |
| 2026-02-10 | Chrome height as shared constant | 76px (36 tab + 40 toolbar), defined in both Rust and JS |
| 2026-02-10 | No shadcn-svelte, custom components | Browser chrome is all custom, take design cues from shadcn aesthetic |
