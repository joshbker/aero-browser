# Code Style Guide — Aero Browser

## General Principles

- **Clarity over cleverness** — write code that's easy to read, not code that's impressive
- **Consistency** — follow existing patterns in the codebase, don't introduce new conventions without good reason
- **Small functions** — if a function is over 40 lines, it probably needs splitting
- **Meaningful names** — `tabCount` not `tc`, `isLoading` not `flag`, `handleTabClose` not `htc`
- **Comments for "why", not "what"** — the code says what it does, comments explain why

---

## JavaScript (Frontend)

### Formatting

- **No semicolons** — rely on ASI (automatic semicolon insertion)
- **Single quotes** for strings
- **Tabs** for indentation (matches Svelte defaults)
- **Trailing commas** in multi-line arrays/objects
- **Max line length**: soft limit 100 chars, hard limit 120

### Naming

| Thing | Convention | Example |
|-------|-----------|---------|
| Variables & functions | camelCase | `activeTab`, `handleClick` |
| Constants | UPPER_SNAKE_CASE | `MAX_TABS`, `DEFAULT_ZOOM` |
| Files (JS) | camelCase | `tabStore.js`, `urlUtils.js` |
| Files (Svelte) | PascalCase | `TabBar.svelte`, `AddressBar.svelte` |
| CSS classes | Tailwind utilities | `flex items-center gap-2` |
| Event handlers | `handle` prefix | `handleTabClose`, `handleNavigate` |
| Boolean variables | `is`/`has`/`can` prefix | `isLoading`, `hasBookmarks`, `canGoBack` |
| Store names | camelCase, plural for collections | `tabs`, `bookmarks`, `settings` |

### Functions

```javascript
// Prefer arrow functions for callbacks and short functions
const isValidUrl = (url) => {
    try {
        new URL(url)
        return true
    } catch {
        return false
    }
}

// Use regular functions for top-level named functions and store factories
function createTabStore() {
    // ...
}

// Destructure parameters when there are 3+ options
function createTab({ url = 'aero://newtab', active = true, index = -1 } = {}) {
    // ...
}
```

### Async/Await

```javascript
// Always use async/await over .then() chains
async function loadBookmarks() {
    try {
        const bookmarks = await invoke('bookmark_get_all')
        set(bookmarks)
    } catch (error) {
        console.error('Failed to load bookmarks:', error)
    }
}

// Never leave promises unhandled — always catch or handle errors
```

### Imports

```javascript
// Order: external packages → Tauri APIs → lib imports → relative imports
// Blank line between groups

import { writable } from 'svelte/store'
import { onMount } from 'svelte'

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { tabs } from '$lib/stores/tabs.js'
import { isValidUrl } from '$lib/utils/url.js'

import TabItem from './TabItem.svelte'
```

### Error Handling

```javascript
// Wrap all IPC calls in try/catch
// Show user-facing errors via a toast/notification system
// Log technical errors to console with context

try {
    await invoke('bookmark_add', { url, title })
} catch (error) {
    console.error(`Failed to add bookmark for ${url}:`, error)
    // Show user notification
}
```

---

## Svelte Components

### Structure

Follow this order within `.svelte` files:

```svelte
<script>
    // 1. Imports
    // 2. Props (using $props())
    // 3. Local state (using $state)
    // 4. Derived state (using $derived)
    // 5. Effects (using $effect)
    // 6. Functions/handlers
    // 7. Lifecycle (onMount, onDestroy)
</script>

<!-- Template -->

<style>
    /* Scoped styles — prefer Tailwind classes in template instead */
</style>
```

### Props & State (Svelte 5)

```svelte
<script>
    // Props with defaults
    let { url = '', isActive = false, onClose } = $props()

    // Local reactive state
    let isHovered = $state(false)
    let title = $state('New Tab')

    // Derived values
    let displayUrl = $derived(url.replace(/^https?:\/\//, ''))
</script>
```

### Event Handling

```svelte
<!-- Use native event attributes (Svelte 5 style) -->
<button onclick={handleClick}>Click</button>
<div onmouseenter={() => isHovered = true}>Hover me</div>

<!-- For custom component events, pass callback props -->
<Tab {url} onClose={() => handleTabClose(tab.id)} />
```

### Component Size

- Keep components focused — one responsibility per component
- If a component file exceeds ~150 lines, consider splitting it
- Extract repeated patterns into shared components

### Accessibility

- All interactive elements must be keyboard accessible
- Use semantic HTML (`<button>`, `<nav>`, `<main>`, etc.)
- Include `aria-label` for icon-only buttons
- Maintain visible focus indicators

---

## Rust (Backend)

### Formatting

- Use `rustfmt` defaults — run `cargo fmt` before committing
- Use `clippy` — run `cargo clippy` and address warnings

### Naming

| Thing | Convention | Example |
|-------|-----------|---------|
| Functions & methods | snake_case | `create_tab`, `get_bookmarks` |
| Types & structs | PascalCase | `TabInfo`, `BookmarkEntry` |
| Constants | UPPER_SNAKE_CASE | `MAX_HISTORY_ENTRIES` |
| Modules | snake_case | `tab_state`, `bookmark_storage` |
| Tauri commands | snake_case matching IPC contract | `tab_create`, `bookmark_add` |

### Structs & Serialisation

```rust
use serde::{Deserialize, Serialize};

// All structs that cross the IPC boundary must derive Serialize (and Deserialize if received)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub favicon: Option<String>,
}
```

### Error Handling

```rust
// For Tauri commands, return Result<T, String> for simplicity
// Map internal errors to user-friendly strings

#[command]
pub fn bookmark_add(url: String, title: String) -> Result<BookmarkEntry, String> {
    let bookmark = BookmarkEntry::new(&url, &title)
        .map_err(|e| format!("Failed to create bookmark: {}", e))?;

    storage::save_bookmark(&bookmark)
        .map_err(|e| format!("Failed to save bookmark: {}", e))?;

    Ok(bookmark)
}
```

### Module Organisation

```rust
// In mod.rs files, keep pub use statements for clean imports
// src-tauri/src/commands/mod.rs

pub mod bookmarks;
pub mod downloads;
pub mod history;
pub mod navigation;
pub mod settings;
pub mod tabs;

// Re-export command functions for registration in lib.rs
pub use bookmarks::*;
pub use tabs::*;
// etc.
```

---

## Tailwind CSS

### Guidelines

- Use Tailwind utilities directly in templates — avoid custom CSS unless absolutely necessary
- Use `@apply` sparingly and only in global styles for truly repeated patterns
- Follow a consistent ordering of utilities (layout → spacing → sizing → typography → colour → effects):

```svelte
<div class="flex items-center gap-2 px-3 py-1.5 h-9 text-sm text-neutral-300 bg-neutral-800 rounded-md hover:bg-neutral-700 transition-colors">
```

### Theme & Design Tokens

Define design tokens as CSS custom properties in `src/lib/styles/themes.css`:

```css
:root {
    --color-bg-primary: theme('colors.neutral.900');
    --color-bg-secondary: theme('colors.neutral.800');
    --color-bg-hover: theme('colors.neutral.700');
    --color-text-primary: theme('colors.neutral.100');
    --color-text-secondary: theme('colors.neutral.400');
    --color-accent: theme('colors.blue.500');
    --color-border: theme('colors.neutral.700');

    --tab-height: 36px;
    --toolbar-height: 40px;
    --sidebar-width: 240px;
}
```

### Colour Palette

Use `neutral` for greys (not `gray`, `slate`, or `zinc` — pick one and stick with it). Use `blue` for primary accent. Keep it minimal.

---

## File & Folder Naming

| Type | Convention | Example |
|------|-----------|---------|
| Svelte components | PascalCase | `TabBar.svelte` |
| JS modules | camelCase | `tabStore.js` |
| Rust files | snake_case | `tab_state.rs` |
| CSS files | kebab-case or camelCase | `themes.css`, `app.css` |
| Documentation | UPPER_CASE | `CLAUDE.md`, `STYLE.md` |
| Config files | as per convention | `tailwind.config.js`, `Cargo.toml` |

---

## Git Conventions

### Commit Messages

Use conventional commits:

```
feat: add tab drag-and-drop reordering
fix: prevent crash when closing last tab
refactor: extract URL validation to util
docs: update IPC contract for bookmark commands
style: format rust code with rustfmt
chore: update tauri to v2.1.0
```

### Branch Naming

```
feature/tab-system
fix/bookmark-save-crash
refactor/store-cleanup
```
