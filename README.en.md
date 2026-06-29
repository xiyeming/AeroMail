<p align="center">
  <img src="assets/logo.svg" alt="AeroMail Logo" width="120">
</p>

<h1 align="center">AeroMail</h1>

<p align="center">
  A modern cross-platform desktop email client built with Rust + Tauri v2 + Vue 3
</p>

<p align="center">
  English · <a href="README.md">中文</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri" alt="Tauri v2">
  <img src="https://img.shields.io/badge/Vue-3-4FC08D?logo=vue.js" alt="Vue 3">
  <img src="https://img.shields.io/badge/Rust-2024-000000?logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT">
</p>

---

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Requirements](#requirements)
- [Quick Start](#quick-start)
  - [Install Dependencies](#install-dependencies)
  - [Start Development](#start-development)
  - [Build Production Package](#build-production-package)
- [Project Structure](#project-structure)
- [Architecture](#architecture)
- [Development Commands](#development-commands)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## Introduction

**AeroMail** is a modern desktop email client designed for end users. It follows a “heavy backend, lightweight frontend” philosophy:
all IO-intensive (database, network) and CPU-intensive (email parsing, search tokenization) tasks are handled by the Rust backend,
while the frontend focuses solely on reactive rendering, ensuring a smooth user experience and low system resource usage.

AeroMail supports IMAP receive/sync, SMTP send, local SQLite caching, Tantivy full-text search, AI assistant,
translation, system tray, minimize-to-tray, multi-language support, and more. It runs on Linux (native Wayland), macOS, and Windows.

## Features

- 📧 **Email send & receive**: IMAP sync for inbox and custom folders; SMTP sending.
- 🔍 **Full-text search**: Tantivy-powered indexing with Chinese tokenization via `tantivy-jieba`.
- 🤖 **AI assistant**: Built-in chat sessions, email summarization, todo extraction, writing assistance, and custom AI providers.
- 🌐 **Translation**: Traditional translation APIs and AI translation with result caching; selected text translation in email body.
- 🔒 **Secure HTML rendering**: Email HTML is rendered in a sandboxed iframe with CSP nonce and event forwarding,
  physically isolating script execution and XSS attacks.
- 🖼️ **Remote content control**: Automatically detects remote image/resource domains in emails; supports “allow once” and “always trust”.
- 🌙 **Dark theme**: Dark mode by default; email content is locked to `color-scheme: light` to prevent layout issues.
- 🌐 **Multi-language**: Simplified Chinese and English built-in; language preference is persisted.
- 🔔 **System tray**: Minimize to tray with show/quit menu.
- ⌨️ **Keyboard navigation**: J/K to navigate emails, Enter to open, Esc to close, and more.

## Tech Stack

| Layer | Technology |
|------|------------|
| Desktop framework | Tauri v2 |
| Frontend framework | Vue 3 Composition API |
| Build tool | Vite |
| Styling | Tailwind CSS v4 |
| State management | Pinia |
| Internationalization | vue-i18n v10 |
| Backend language | Rust 2024 edition |
| Async runtime | Tokio |
| Database | SQLite (rusqlite bundled) |
| Search index | Tantivy + tantivy-jieba |
| Email protocols | IMAP (`async-imap`), SMTP (`lettre`) |
| Email parsing | mail-parser |
| TLS | native-tls / tokio-native-tls |

## Requirements

- **Rust**: 1.85+ (this project uses `edition = "2024"`)
- **Node.js**: 20+
- **Package manager**: pnpm
- **Linux development dependencies**: WebKitGTK development packages, e.g.:
  - `webkit2gtk-4.1-devel`
  - `libgtk-3-devel`

> Package names may vary by distribution. Install the WebKitGTK 4.1 development package matching your environment.

## Quick Start

### Install Dependencies

```bash
pnpm install
```

### Start Development

```bash
cargo tauri dev
```

This command starts the Vite frontend dev server (port `1420`) and the Rust backend automatically.

#### Running on Linux Wayland

```bash
export GDK_BACKEND=wayland
export WEBKIT_DISABLE_COMPOSITING_MODE=0
cargo tauri dev
```

### Build Production Package

```bash
# Frontend production build
pnpm build

# Full application package (outputs deb/appimage/dmg/msi/nsis per platform)
cargo tauri build
```

## Project Structure

```text
AeroMail/
├── src/                    # Frontend source (Vue 3 + TypeScript)
│   ├── components/         # Reusable components
│   ├── composables/        # Composable functions
│   ├── layouts/            # Layout components
│   ├── router.ts           # Vue Router
│   ├── stores/             # Pinia state management
│   ├── i18n/               # Internationalization messages
│   ├── styles/             # Theme and global styles
│   └── main.ts             # Frontend entry
├── src-tauri/              # Rust + Tauri backend
│   ├── src/
│   │   ├── commands/       # Tauri IPC command handlers
│   │   ├── services/       # Business service implementations
│   │   ├── models/         # Shared data models
│   │   ├── db/             # SQLite schema, pool, migrations
│   │   └── lib.rs          # App state and Tauri runtime
│   └── Cargo.toml
├── docs/                   # Documentation
├── scripts/                # Scripts (e.g. i18n check)
├── assets/                 # Static assets (logo, etc.)
├── package.json
├── LICENSE
└── README.md
```

## Architecture

### Processes & Runtime

- **Webview process**: Each window runs in its own process with the Vue 3 frontend.
- **Rust main process**: Single process, multi-threaded, responsible for IPC routing, state management, and plugin scheduling.
- **Tokio runtime**: Multi-threaded thread pool scheduling IMAP sync, SMTP sending, AI/translation API calls, and other async tasks.

### Backend State

Backend state is centralized in `AppState` in `src-tauri/src/lib.rs`.
All services are attached to the Tauri app state as `Arc<RwLock<T>>`:

```rust
pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub ai_service: Arc<RwLock<AiService>>,
    pub translation_service: TranslationService,
    pub sync_service: Arc<RwLock<SyncService>>,
    pub search_service: Arc<RwLock<SearchService>>,
    pub db: Arc<Database>,
}
```

Tauri Commands access these services via `tauri::State<'_, AppState>`.

### Sync Engine

`SyncService` maintains a dedicated Tokio Task per account:

1. UIDVALIDITY validation
2. Fetch the local maximum UID
3. IMAP FETCH new emails
4. MIME parsing with `mail-parser`
5. Write to SQLite
6. Write to Tantivy index

Failures are retried with exponential backoff; sync resumes automatically when the network recovers.

### HTML Email Rendering Security

Email HTML is rendered in an `<iframe sandbox="allow-same-origin allow-scripts">`
with a CSP nonce that allows only a trusted inline script. This script forwards `<a>` link clicks to the parent window via `postMessage`.
External images are blocked by default to protect privacy and prevent XSS.

## Development Commands

```bash
# Frontend type check
pnpm type-check

# ESLint check
pnpm lint

# Frontend formatting
pnpm format

# i18n key consistency check
pnpm i18n:check

# Rust formatting
cargo fmt

# Rust static analysis
cargo clippy

# Rust tests
cargo test
```

## Contributing

Issues and Pull Requests are welcome!

1. Fork this repository and create a feature branch: `git checkout -b feat/your-feature`
2. Make sure your code passes `pnpm type-check`, `pnpm lint`, `pnpm i18n:check`, and `cargo clippy`
3. Write clear commit messages following [Conventional Commits](https://www.conventionalcommits.org/)
4. Open a Pull Request describing the motivation and impact of your changes

## License

This project is open source under the [MIT License](LICENSE).

Copyright (c) 2026 [xiyeming](mailto:xiyeming@163.com)

## Acknowledgements

- [Tauri](https://tauri.app/)
- [Vue.js](https://vuejs.org/)
- [Vite](https://vitejs.dev/)
- [Tailwind CSS](https://tailwindcss.com/)
- [Tokio](https://tokio.rs/)
- [Tantivy](https://tantivy-py.github.io/)
- [mail-parser](https://github.com/stalwartlabs/mail-parser)
- [lettre](https://lettre.rs/)
- [async-imap](https://github.com/async-email/async-imap)
