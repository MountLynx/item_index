<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Index" width="80" />
</p>

<h1 align="center">Index</h1>

<p align="center">
  <strong>Local-first · Plugin-extensible · Object Manager</strong>
  <br />
  <em>Think "personal wiki meets Zotero" — built with Tauri 2</em>
</p>

<p align="center">
  <a href="README.zh-CN.md">中文</a>
</p>

---

## Overview

**Index** is a local-first desktop application for managing structured data — items with dynamic types, multi-level groups, flat tags, and file attachments. It provides a customizable framework that can become a calendar, a library tracker, a knowledge base, or anything in between through plugins and workspace configurations.

Each repository is a plain folder on disk: items live as hash-named subfolders with attachments, and all metadata is stored in a local SQLite database (`.index/index.db`). No cloud, no lock-in — your data is yours.

### Core Concepts

| Concept | Description |
|---------|-------------|
| **Item** | The fundamental unit — could be a note, a task, a book, a contact, anything. Each item gets a 12-char hex hash folder. |
| **Item Type** | User-defined schemas. Add custom fields (text, checkbox, etc.) to shape items for different use cases. |
| **Group** | Multi-level tree hierarchy for organizing items. Drag and drop items between groups. |
| **Tag** | Flat labels for cross-cutting categorization. An item can have multiple tags. |
| **Attachment** | Any file dropped into an item's hash folder. Browse the file tree in the right panel. |

### Screenshot

> *Coming soon*

## Features

- **Dynamic Type System** — create item types with custom fields, icons, and labels
- **Multi-Level Groups** — tree-based organization with drag-and-drop
- **Flat Tags** — cross-cutting labels for filtering
- **File Attachments** — per-item hash folders with full directory tree
- **Three-Panel Layout** — sidebar (groups + tags) · center list · right detail panel
- **Frameless Window** — custom titlebar with native window controls
- **Theme System** — light/dark mode, custom CSS variable overrides, preset management
- **i18n** — English and Simplified Chinese
- **Repository Dashboard** — manage multiple local repos from a grid-card landing screen

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | [Tauri 2](https://v2.tauri.app/) |
| Frontend | [Vue 3.5](https://vuejs.org/) + TypeScript (strict) |
| State | [Pinia](https://pinia.vuejs.org/) (Composition API) |
| Icons | [Tabler Icons](https://tabler.io/icons) |
| Backend | Rust, [sqlx](https://github.com/launchbadge/sqlx) (SQLite), serde, tokio, chrono |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) ≥ 18
- [pnpm](https://pnpm.io/) ≥ 9
- [Rust](https://rustup.rs/) (stable toolchain)
- Platform-specific Tauri 2 [prerequisites](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
# Install dependencies
pnpm install

# Start dev server + Tauri window
pnpm dev

# Type-check only
pnpm build
```

The frontend dev server runs on **port 1420**. Debug tools open automatically in dev mode.

### Build

```bash
pnpm tauri build
```

Outputs to `src-tauri/target/release/`.

## Project Structure

```
├── src/                      # Vue 3 frontend
│   ├── components/           # Vue components (Titlebar, Sidebar, RepoDashboard, …)
│   ├── stores/               # Pinia stores (repo, types, items, groups, tags, dashboard)
│   ├── locales/              # i18n (zh-CN.ts, en.ts)
│   ├── types/                # TypeScript bindings (mirrors Rust models)
│   └── assets/               # theme.css, fonts
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── commands/         # IPC commands (repo, types, items, groups, tags, files, dashboard)
│   │   ├── models.rs         # Shared data structures
│   │   ├── db.rs             # SQLite pool + migrations
│   │   └── safe_path.rs      # Path traversal prevention
│   ├── migrations/           # sqlx migrations
│   └── Cargo.toml
├── docs/
│   ├── design.md             # Original vision & FRS (Chinese)
│   └── superpowers/specs/    # Design specs for recent features
└── CLAUDE.md                 # Developer guide for AI-assisted coding
```

### Repository Layout (on disk)

```
~/MyIndex/                    # A repo (regular folder)
├── .index/
│   ├── index.db              # SQLite database
│   └── state.json            # App state (theme, etc.)
├── a3f2c1b8e9d4/             # Item folder (12-char hex)
│   ├── My Note.md            # Auto-generated Markdown
│   └── image.png             # Arbitrary attachments
└── b7e1d5c3f2a8/
    └── ...
```

## Roadmap

See [`docs/design.md`](docs/design.md) for the full 6-phase vision:

1. ✅ Core Engine — dynamic types, groups, tags, file system
2. ✅ UI Layout — three-panel shell, frameless window, theme system
3. 🚧 File System — enhanced attachment management
4. 📋 Plugin System — extension API for custom views and panels
5. 🤖 AI Integration — LLM-powered operations and skills
6. ⚙️ Workspace Configs — preset workspaces for specific domains

## Contributing

This is a personal project in active early development. Feedback and ideas are welcome — feel free to open an issue.

## License

MIT
