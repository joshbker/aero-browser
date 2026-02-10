# Expected Configuration — Aero Browser

This file documents the expected configuration for key config files in the project. Use these as reference when setting up or modifying configs.

---

## tauri.conf.json

Key settings that differ from defaults:

```json
{
    "productName": "Aero",
    "version": "0.1.0",
    "identifier": "com.aero.browser",
    "build": {
        "frontendDist": "../build",
        "devUrl": "http://localhost:5173",
        "beforeDevCommand": "npm run dev",
        "beforeBuildCommand": "npm run build"
    },
    "app": {
        "withGlobalTauri": true,
        "windows": [
            {
                "label": "main",
                "title": "Aero",
                "width": 1280,
                "height": 800,
                "minWidth": 400,
                "minHeight": 300,
                "decorations": false,
                "transparent": false,
                "resizable": true,
                "fullscreen": false,
                "center": true
            }
        ]
    },
    "bundle": {
        "active": true,
        "targets": "all",
        "icon": [
            "icons/32x32.png",
            "icons/128x128.png",
            "icons/128x128@2x.png",
            "icons/icon.icns",
            "icons/icon.ico"
        ]
    }
}
```

### Important Notes

- `decorations: false` — we render our own title bar
- `withGlobalTauri: true` — injects `window.__TAURI__` so we can use Tauri APIs
- The window config here is for the MAIN window only. Content webviews (tabs) are created programmatically, not in config
- `create: false` can be set on the window if you want to create it manually in `setup()`

---

## Cargo.toml (src-tauri)

Key dependencies:

```toml
[dependencies]
tauri = { version = "2", features = ["unstable"] }
tauri-build = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
url = "2"
uuid = { version = "1", features = ["v4"] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

### Important Notes

- `features = ["unstable"]` on tauri — REQUIRED for multi-webview `add_child` API
- `rusqlite` with `bundled` feature — bundles SQLite so no system dependency needed
- `url` crate — for URL parsing and validation
- `uuid` — for generating unique IDs for bookmarks, history entries, etc.

---

## svelte.config.js

```javascript
import adapter from '@sveltejs/adapter-static'

/** @type {import('@sveltejs/kit').Config} */
const config = {
    kit: {
        adapter: adapter({
            fallback: 'index.html',
        }),
    },
}

export default config
```

### Important Notes

- MUST use `adapter-static` — there is no server in a Tauri app
- `fallback: 'index.html'` — ensures client-side routing works
- Do NOT use `adapter-auto` or `adapter-node`

---

## vite.config.js

```javascript
import { sveltekit } from '@sveltejs/kit/vite'
import { defineConfig } from 'vite'

export default defineConfig({
    plugins: [sveltekit()],
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
        target: ['es2021', 'chrome97'],
        minify: !process.env.TAURI_DEBUG && 'esbuild',
        sourcemap: !!process.env.TAURI_DEBUG,
    },
    server: {
        port: 5173,
        strictPort: true,
    },
})
```

### Important Notes

- `envPrefix: ['VITE_', 'TAURI_']` — exposes Tauri env vars to frontend
- `target: chrome97` — WebView2 supports this target
- Fixed port `5173` to match `devUrl` in `tauri.conf.json`

---

## tailwind.config.js

```javascript
/** @type {import('tailwindcss').Config} */
export default {
    content: ['./src/**/*.{html,js,svelte}'],
    theme: {
        extend: {
            colors: {
                // Use neutral palette as our grey scale
                // Extend only if custom brand colours are needed
            },
            fontSize: {
                'xxs': '0.625rem', // 10px — useful for tab text
            },
        },
    },
    plugins: [],
}
```

### Important Notes

- Use `neutral` for all greys — do not mix with `gray`, `slate`, `zinc`
- Keep the config minimal — avoid adding unused utilities
- `content` path covers all Svelte and JS files in `src/`

---

## package.json scripts

```json
{
    "scripts": {
        "dev": "vite dev",
        "build": "vite build",
        "preview": "vite preview",
        "tauri": "tauri"
    }
}
```

### Key Dependencies

```json
{
    "dependencies": {
        "@tauri-apps/api": "^2",
        "@tauri-apps/plugin-shell": "^2"
    },
    "devDependencies": {
        "@sveltejs/adapter-static": "^3",
        "@sveltejs/kit": "^2",
        "svelte": "^5",
        "tailwindcss": "^3",
        "autoprefixer": "^10",
        "postcss": "^8",
        "vite": "^5",
        "@tauri-apps/cli": "^2",
        "lucide-svelte": "latest"
    }
}
```

### Important Notes

- `@tauri-apps/api` v2, NOT v1
- `@sveltejs/adapter-static` — required for Tauri
- `svelte` v5 — uses runes syntax
- shadcn-svelte components are copied into the project, not installed as a dep — follow their CLI setup
