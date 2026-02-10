# Aero Browser

A minimal, privacy-focused browser built with Tauri v2 + SvelteKit + WebView2. No bloat, just browsing.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs) (latest stable)
- [Node.js](https://nodejs.org) (v18+)
- [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Windows requirement for Rust)
- Windows 10/11 (WebView2 runtime is pre-installed)

### Setup

```bash
# Clone the repo
git clone <repo-url>
cd aero-browser

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Project Structure

```
aero-browser/
├── src-tauri/       # Rust backend (Tauri commands, state, storage)
├── src/             # SvelteKit frontend (browser UI shell)
├── docs/            # Documentation
│   ├── PROJECT.md       # Full spec, roadmap, IPC contract, DB schema
│   ├── CLAUDE.md        # AI agent instructions
│   ├── STYLE.md         # Code style guide
│   └── ARCHITECTURE.md  # Architecture decision records
└── README.md        # You are here
```

### Key Commands

```bash
npm run tauri dev      # Dev mode with hot reload
npm run tauri build    # Production build
cd src-tauri && cargo test   # Run Rust tests
cargo fmt              # Format Rust code
cargo clippy           # Lint Rust code
```

## Documentation

| Doc | What it covers |
|-----|---------------|
| [PROJECT.md](docs/PROJECT.md) | Vision, feature roadmap, architecture overview, IPC contract, DB schema, tech stack |
| [CLAUDE.md](CLAUDE.md) | Instructions for AI coding agents working on this project |
| [STYLE.md](docs/STYLE.md) | Code formatting, naming conventions, patterns for JS/Svelte/Rust/CSS |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Key architectural decisions and reasoning |
| [MULTIWEBVIEW.md](docs/MULTIWEBVIEW.md) | Concrete code examples for the multi-webview tab architecture |
| [CONFIG.md](docs/CONFIG.md) | Expected configuration for all config files |
| [TASKS.md](docs/TASKS.md) | Active task tracker with phase breakdowns |

## License

TBD
