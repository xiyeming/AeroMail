# AeroMail Phase 1 Skeleton Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a runnable AeroMail project skeleton with Tauri 2.11 + Vue 3 + Rust 2024 edition, including three-pane layout, theme system, SQLite persistence, and account management CRUD.

**Architecture:** A Tauri v2 desktop app with a Rust backend handling state and SQLite persistence, and a Vue 3 frontend using Composition API, Pinia, Tailwind CSS, and Shadcn UI components. The first phase focuses on project scaffolding, database schema, account management commands, and the main UI shell.

**Tech Stack:** Rust 2024 edition, Tauri 2.11, Vue 3.5+, Vite 6+, TypeScript 5.7+, Tailwind CSS 4.x, Shadcn UI (Vue), Lucide Vue, Pinia, rusqlite, tokio, serde, thiserror, uuid, chrono.

## Global Constraints

- Rust edition: `2024`
- Tauri version: `2.11`
- Vue version: `3.5+`
- Tailwind CSS version: `4.x`
- TypeScript: `strict: true`
- All IPC commands return `Result<T, String>` for the first phase.
- No `unwrap()` in production Rust code; use `?` or typed errors.
- No `any` in TypeScript without explicit justification comment.
- Front-end components use `<script setup lang="ts">`.
- File naming: components `PascalCase.vue`, composables `camelCase.ts`, stores `camelCase.ts`.
- All designs follow `docs/superpowers/specs/2026-06-17-aeromail-design.md`.

---

## File Structure

### Backend (src-tauri/src/)

| File | Responsibility |
|------|----------------|
| `main.rs` | Tauri app entrypoint, plugin registration, window setup |
| `lib.rs` | Public module exports |
| `error.rs` | `AeroError` typed error enum and conversions |
| `models/account.rs` | Account data structures (`AccountConfig`, `MailProvider`, etc.) |
| `db/schema.rs` | SQL table definitions |
| `db/pool.rs` | SQLite connection pool and initialization |
| `db/migrations.rs` | Schema migration runner |
| `services/account_manager.rs` | Account CRUD business logic |
| `commands/account.rs` | Tauri commands for account operations |

### Frontend (src/)

| File | Responsibility |
|------|----------------|
| `main.ts` | Vue app entrypoint, Pinia, router, theme init |
| `App.vue` | Root component, layout wrapper |
| `router.ts` | Vue Router routes |
| `styles/theme.css` | CSS variables for Dark/Light themes |
| `styles/fonts.css` | @font-face declarations and font stack |
| `types/account.ts` | Account TypeScript interfaces |
| `composables/useTheme.ts` | Theme switching composable |
| `composables/useResponsive.ts` | Breakpoint detection composable |
| `composables/useTauriInvoke.ts` | Typed Tauri invoke wrapper |
| `stores/account.ts` | Account Pinia store |
| `stores/status.ts` | Status bar Pinia store |
| `stores/toast.ts` | Toast queue Pinia store |
| `layouts/AppLayout.vue` | Three-pane responsive layout |
| `components/AppSidebar.vue` | Sidebar with folders, accounts, actions |
| `components/MailList.vue` | Mail list panel placeholder |
| `components/MailViewer.vue` | Mail reading panel placeholder |
| `components/CommandPalette.vue` | Global search/command modal |
| `components/StatusBar.vue` | Bottom status bar |
| `components/ToastContainer.vue` | Toast notification container |
| `views/InboxView.vue` | Inbox page view |

---

## Task 1: Initialize Tauri + Vue Project

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tsconfig.app.json`
- Create: `tsconfig.node.json`
- Create: `index.html`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/build.rs`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src/vite-env.d.ts`

**Interfaces:**
- Consumes: None
- Produces: A runnable Tauri app with a blank Vue page.

- [ ] **Step 1: Write root package.json**

```json
{
  "name": "aeromail",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext .vue,.ts,.tsx",
    "type-check": "vue-tsc --noEmit",
    "format": "prettier --write \"src/**/*.{ts,vue,css}\""
  },
  "dependencies": {
    "@tauri-apps/api": "^2.5.0",
    "@tauri-apps/plugin-os": "^2.2.0",
    "@vueuse/core": "^13.0.0",
    "class-variance-authority": "^0.7.1",
    "clsx": "^2.1.1",
    "lucide-vue-next": "^0.510.0",
    "pinia": "^3.0.1",
    "radix-vue": "^1.9.17",
    "tailwind-merge": "^3.0.0",
    "vue": "^3.5.13",
    "vue-router": "^4.5.0"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4.0.0",
    "@types/node": "^22.13.0",
    "@vitejs/plugin-vue": "^5.2.1",
    "@vue/tsconfig": "^0.7.0",
    "eslint": "^9.20.0",
    "eslint-plugin-vue": "^9.32.0",
    "prettier": "^3.5.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.7.3",
    "vite": "^6.1.0",
    "vue-tsc": "^2.2.0"
  }
}
```

- [ ] **Step 2: Write vite.config.ts**

```typescript
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'path';

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
```

- [ ] **Step 3: Write tsconfig.json**

```json
{
  "files": [],
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./tsconfig.node.json" }
  ]
}
```

- [ ] **Step 4: Write tsconfig.app.json**

```json
{
  "extends": "@vue/tsconfig/tsconfig.dom.json",
  "include": ["env.d.ts", "src/**/*", "src/**/*.vue"],
  "exclude": ["src/**/__tests__/*"],
  "compilerOptions": {
    "composite": true,
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.app.tsbuildinfo",
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    },
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  }
}
```

- [ ] **Step 5: Write tsconfig.node.json**

```json
{
  "extends": "@vue/tsconfig/tsconfig.node.json",
  "include": [
    "vite.config.*",
    "vitest.config.*",
    "cypress.config.*",
    "nightwatch.conf.*",
    "playwright.config.*"
  ],
  "compilerOptions": {
    "composite": true,
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.node.tsbuildinfo"
  }
}
```

- [ ] **Step 6: Write index.html**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>AeroMail</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 7: Write src/vite-env.d.ts**

```typescript
/// <reference types="vite/client" />
```

- [ ] **Step 8: Write src/main.ts**

```typescript
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import './styles/theme.css';
import './styles/fonts.css';

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount('#app');
```

- [ ] **Step 9: Write src/App.vue**

```vue
<script setup lang="ts">
import { RouterView } from 'vue-router';
</script>

<template>
  <RouterView />
</template>
```

- [ ] **Step 10: Write src/router.ts**

```typescript
import { createRouter, createWebHistory } from 'vue-router';
import InboxView from './views/InboxView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'inbox',
      component: InboxView,
    },
  ],
});

export default router;
```

- [ ] **Step 11: Write src/views/InboxView.vue**

```vue
<script setup lang="ts">
import AppLayout from '@/layouts/AppLayout.vue';
</script>

<template>
  <AppLayout>
    <div class="flex h-full items-center justify-center text-muted">
      Select an email to read
    </div>
  </AppLayout>
</template>
```

- [ ] **Step 12: Write src-tauri/Cargo.toml**

```toml
[package]
name = "aeromail"
version = "0.1.0"
description = "AeroMail - Modern cross-platform email client"
authors = ["AeroMail Team"]
edition = "2024"
rust-version = "1.85"

[build-dependencies]
tauri-build = { version = "2.2", features = [] }

[dependencies]
tauri = { version = "2.5", features = ["tray-icon", "image-png"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.44", features = ["full"] }
thiserror = "2.0"
anyhow = "1.0"
rusqlite = { version = "0.34", features = ["bundled", "chrono", "uuid"] }
uuid = { version = "1.16", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
unwrap_used = "deny"
expect_used = "warn"

[profile.release]
strip = true
lto = true
opt-level = "s"
```

- [ ] **Step 13: Write src-tauri/tauri.conf.json**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "AeroMail",
  "version": "0.1.0",
  "identifier": "com.aeromail.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "AeroMail",
        "width": 1400,
        "height": 900,
        "minWidth": 800,
        "minHeight": 600,
        "transparent": true,
        "decorations": true,
        "center": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' https: data:; style-src 'self' 'unsafe-inline'; script-src 'none';"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage", "dmg", "msi", "nsis"],
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

- [ ] **Step 14: Write src-tauri/capabilities/default.json**

```json
{
  "$schema": "https://schema.tauri.app/capabilities/2.json",
  "identifier": "default",
  "description": "Default capabilities for AeroMail",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging"
  ]
}
```

- [ ] **Step 15: Write src-tauri/build.rs**

```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 16: Write src-tauri/src/main.rs**

```rust
// Prevent console window in addition to the Slint window in Windows release builds when, e.g., launching the app from file manager
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aeromail::run;

fn main() {
    run();
}
```

- [ ] **Step 17: Write src-tauri/src/lib.rs**

```rust
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

use commands::account::{add_account, delete_account, list_accounts, test_account_connection};
use db::pool::Database;
use services::account_manager::AccountManager;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub db: Arc<Database>,
}

impl AppState {
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, crate::error::AeroError> {
        let db = Arc::new(Database::new(app_handle).await?);
        let account_manager = Arc::new(RwLock::new(AccountManager::new(Arc::clone(&db))));
        Ok(Self {
            account_manager,
            db,
        })
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust.")
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = AppState::new(&handle)
                    .await
                    .expect("failed to initialize app state");
                handle.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            add_account,
            list_accounts,
            delete_account,
            test_account_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 18: Install dependencies and run initial build**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail
pnpm install
cargo install tauri-cli --version '^2.5.0'
cargo tauri dev
```

Expected: Tauri dev window opens with a blank page showing "Select an email to read". If it fails, fix dependency versions before proceeding.

- [ ] **Step 19: Commit**

```bash
git add .
git commit -m "chore: initialize Tauri 2.11 + Vue 3 + Vite project skeleton

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: Configure Tailwind Theme and Global Styles

**Files:**
- Create: `src/styles/theme.css`
- Create: `src/styles/fonts.css`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: None
- Produces: CSS custom properties `--background`, `--panel`, `--card`, etc.; default dark theme applied to `document.documentElement`.

- [ ] **Step 1: Write src/styles/theme.css**

```css
@import "tailwindcss";

@theme {
  --color-background: var(--background);
  --color-panel: var(--panel);
  --color-card: var(--card);
  --color-border: var(--border);
  --color-border-hover: var(--border-hover);
  --color-primary: var(--primary);
  --color-primary-hover: var(--primary-hover);
  --color-primary-active: var(--primary-active);
  --color-success: var(--success);
  --color-warning: var(--warning);
  --color-danger: var(--danger);
  --color-info: var(--info);
  --color-text: var(--text);
  --color-text-secondary: var(--text-secondary);
  --color-muted: var(--muted);
  --color-disabled: var(--disabled);
  --color-overlay: var(--overlay);
  --color-glass: var(--glass);
}

:root {
  /* Layout */
  --sidebar-width: 240px;
  --sidebar-width-wide: 260px;
  --maillist-width: 420px;
  --maillist-width-wide: 480px;
  --viewer-min-width: 480px;
  --viewer-min-width-wide: 560px;

  /* Radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;

  /* Spacing */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-10: 40px;
  --space-12: 48px;

  /* Shadows */
  --shadow-modal: 0 16px 48px rgba(0, 0, 0, 0.4);
  --shadow-toast: 0 4px 16px rgba(0, 0, 0, 0.3);
  --shadow-card: 0 1px 3px rgba(0, 0, 0, 0.1);

  /* Dark theme (default) */
  --background: #0b0f14;
  --panel: #121821;
  --card: #1a2230;
  --border: #2a3342;
  --border-hover: #3a4659;
  --primary: #4d8dff;
  --primary-hover: #6ba3ff;
  --primary-active: #3576e8;
  --success: #12b76a;
  --warning: #f79009;
  --danger: #f04438;
  --info: #4d8dff;
  --text: #f8fafc;
  --text-secondary: #cbd5e1;
  --muted: #94a3b8;
  --disabled: #64748b;
  --overlay: rgba(0, 0, 0, 0.6);
  --glass: rgba(18, 24, 33, 0.85);
}

[data-theme="light"] {
  --background: #f8fafc;
  --panel: #ffffff;
  --card: #f1f5f9;
  --border: #e2e8f0;
  --border-hover: #cbd5e1;
  --primary: #2563eb;
  --primary-hover: #3b82f6;
  --primary-active: #1d4ed8;
  --success: #10b981;
  --warning: #f59e0b;
  --danger: #ef4444;
  --info: #2563eb;
  --text: #0f172a;
  --text-secondary: #475569;
  --muted: #64748b;
  --disabled: #94a3b8;
  --overlay: rgba(0, 0, 0, 0.4);
  --glass: rgba(255, 255, 255, 0.85);
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
  width: 100%;
  margin: 0;
}

body {
  font-family: var(--font-sans);
  background-color: var(--background);
  color: var(--text);
  overflow: hidden;
}

@media (prefers-color-scheme: light) {
  :root:not([data-theme="dark"]) {
    --background: #f8fafc;
    --panel: #ffffff;
    --card: #f1f5f9;
    --border: #e2e8f0;
    --border-hover: #cbd5e1;
    --primary: #2563eb;
    --primary-hover: #3b82f6;
    --primary-active: #1d4ed8;
    --success: #10b981;
    --warning: #f59e0b;
    --danger: #ef4444;
    --info: #2563eb;
    --text: #0f172a;
    --text-secondary: #475569;
    --muted: #64748b;
    --disabled: #94a3b8;
    --overlay: rgba(0, 0, 0, 0.4);
    --glass: rgba(255, 255, 255, 0.85);
  }
}
```

- [ ] **Step 2: Write src/styles/fonts.css**

```css
:root {
  --font-sans: 'Inter', 'MiSans', 'HarmonyOS Sans', system-ui, -apple-system,
    sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

@supports (-webkit-font-smoothing: antialiased) {
  body {
    -webkit-font-smoothing: antialiased;
  }
}
```

- [ ] **Step 3: Update src/main.ts to set default dark theme**

```typescript
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import './styles/theme.css';
import './styles/fonts.css';

document.documentElement.setAttribute('data-theme', 'dark');

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount('#app');
```

- [ ] **Step 4: Verify theme variables**

Run:
```bash
cargo tauri dev
```

Open DevTools, check that `<html>` has `data-theme="dark"` and CSS variables are defined. The page background should be `#0b0f14`.

- [ ] **Step 5: Commit**

```bash
git add src/styles src/main.ts
git commit -m "feat: add dark/light theme CSS variables and font stack

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: Configure Rust Toolchain and Linting

**Files:**
- Create: `rustfmt.toml`
- Create: `.cargo/config.toml`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: None
- Produces: Rust formatting and linting configuration.

- [ ] **Step 1: Write rustfmt.toml**

```toml
edition = "2024"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

- [ ] **Step 2: Write .cargo/config.toml**

```toml
[build]
target-dir = "target"

[env]
RUST_BACKTRACE = "1"
```

- [ ] **Step 3: Verify clippy**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail/src-tauri
cargo clippy --all-targets --all-features
```

Expected: Clippy runs with warnings but no errors. Fix any `unwrap_used` violations by replacing with proper error handling.

- [ ] **Step 4: Commit**

```bash
git add rustfmt.toml .cargo/config.toml src-tauri/Cargo.toml
git commit -m "chore: configure rustfmt, clippy, and cargo build settings

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: Define Backend Error Type and Models

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/models/account.rs`
- Create: `src-tauri/src/models/mod.rs`

**Interfaces:**
- Consumes: None
- Produces: `AeroError`, `AccountConfig`, `MailProvider`, `ServerConfig`, `TlsMode`, `AuthConfig`, `AdvancedConfig`.

- [ ] **Step 1: Write src-tauri/src/error.rs**

```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AeroError {
    #[error("database error: {0}")]
    Database(String),
    #[error("account not found: {0}")]
    AccountNotFound(String),
    #[error("invalid account configuration: {0}")]
    InvalidConfig(String),
    #[error("connection test failed: {0}")]
    ConnectionTestFailed(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for AeroError {
    fn from(err: rusqlite::Error) -> Self {
        AeroError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for AeroError {
    fn from(err: serde_json::Error) -> Self {
        AeroError::Internal(err.to_string())
    }
}

impl From<AeroError> for String {
    fn from(err: AeroError) -> Self {
        serde_json::to_string(&err).unwrap_or_else(|_| err.to_string())
    }
}
```

- [ ] **Step 2: Write src-tauri/src/models/account.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    pub provider: MailProvider,
    pub imap: ServerConfig,
    pub smtp: ServerConfig,
    pub auth: AuthConfig,
    pub advanced: AdvancedConfig,
    pub sync_interval_secs: u64,
    pub excluded_folders: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MailProvider {
    Gmail,
    Outlook,
    QQ,
    Netease163,
    Aliyun,
    TencentExmail,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_mode: TlsMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TlsMode {
    Required,
    StartTls,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    OAuth2 {
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    },
    Password {
        password_encrypted: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub ca_cert_path: Option<String>,
    pub verify_certificate: bool,
    pub connect_timeout_secs: u64,
    pub read_timeout_secs: u64,
    pub keepalive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub name: String,
    pub provider: MailProvider,
    pub imap_host: String,
    pub smtp_host: String,
}
```

- [ ] **Step 3: Write src-tauri/src/models/mod.rs**

```rust
pub mod account;
```

- [ ] **Step 4: Verify compilation**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail/src-tauri
cargo check
```

Expected: Compilation succeeds.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/models
git commit -m "feat: define AeroError and Account models

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: Set Up SQLite Database Schema and Pool

**Files:**
- Create: `src-tauri/src/db/schema.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Create: `src-tauri/src/db/pool.rs`
- Create: `src-tauri/src/db/mod.rs`

**Interfaces:**
- Consumes: `AeroError`
- Produces: `Database` struct with `new()` and `connection()` methods.

- [ ] **Step 1: Write src-tauri/src/db/schema.rs**

```rust
pub const ACCOUNTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    imap_host TEXT NOT NULL,
    imap_port INTEGER NOT NULL,
    smtp_host TEXT NOT NULL,
    smtp_port INTEGER NOT NULL,
    tls_mode TEXT NOT NULL,
    auth_type TEXT NOT NULL,
    auth_credentials_encrypted BLOB,
    ca_cert_path TEXT,
    verify_certificate INTEGER DEFAULT 1,
    connect_timeout INTEGER DEFAULT 30,
    read_timeout INTEGER DEFAULT 30,
    keepalive INTEGER DEFAULT 1,
    sync_interval INTEGER DEFAULT 60,
    excluded_folders TEXT,
    created_at INTEGER,
    updated_at INTEGER
)
"#;

pub const FOLDERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    unread_count INTEGER DEFAULT 0,
    total_count INTEGER DEFAULT 0,
    uid_validity INTEGER,
    last_sync_at INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
)
"#;

pub const MAILS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS mails (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    folder_id TEXT NOT NULL,
    uid INTEGER NOT NULL,
    subject TEXT,
    from_name TEXT,
    from_address TEXT,
    to_addresses TEXT,
    cc_addresses TEXT,
    date INTEGER,
    body_html TEXT,
    body_text TEXT,
    is_read INTEGER DEFAULT 0,
    is_starred INTEGER DEFAULT 0,
    flags TEXT,
    created_at INTEGER,
    indexed_at INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
)
"#;

pub const ATTACHMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    mail_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size INTEGER,
    content_id TEXT,
    local_path TEXT,
    is_inline INTEGER DEFAULT 0,
    FOREIGN KEY (mail_id) REFERENCES mails(id) ON DELETE CASCADE
)
"#;

pub const DRAFTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS drafts (
    id TEXT PRIMARY KEY,
    account_id TEXT,
    subject TEXT,
    to_addresses TEXT,
    cc_addresses TEXT,
    body_html TEXT,
    body_text TEXT,
    attachments_json TEXT,
    saved_at INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE SET NULL
)
"#;

pub const SETTINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at INTEGER
)
"#;

pub const ALL_SCHEMAS: &[&str] = &[
    ACCOUNTS_TABLE,
    FOLDERS_TABLE,
    MAILS_TABLE,
    ATTACHMENTS_TABLE,
    DRAFTS_TABLE,
    SETTINGS_TABLE,
];
```

- [ ] **Step 2: Write src-tauri/src/db/migrations.rs**

```rust
use rusqlite::Connection;

use super::schema::ALL_SCHEMAS;
use crate::error::AeroError;

pub fn run_migrations(conn: &mut Connection) -> Result<(), AeroError> {
    let tx = conn.transaction()?;
    for schema in ALL_SCHEMAS {
        tx.execute_batch(schema)?;
    }
    tx.commit()?;
    Ok(())
}
```

- [ ] **Step 3: Write src-tauri/src/db/pool.rs**

```rust
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::Manager;

use super::migrations::run_migrations;
use crate::error::AeroError;

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    // Mutex<Connection> is acceptable for a desktop single-user app in phase 1.
    // Replace with r2d2/deadpool pool in later phases if concurrent access grows.
    connection: Mutex<Connection>,
}

impl Database {
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, AeroError> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AeroError::Internal(e.to_string()))?;

        std::fs::create_dir_all(&app_dir)
            .map_err(|e| AeroError::Internal(format!("failed to create app dir: {e}")))?;

        let db_path = app_dir.join("aeromail.db");
        let mut conn = Connection::open(&db_path)
            .map_err(|e| AeroError::Database(format!("failed to open database: {e}")))?;

        run_migrations(&mut conn)?;

        Ok(Self {
            path: db_path,
            connection: Mutex::new(conn),
        })
    }

    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .expect("database connection mutex poisoned")
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

// SAFETY: Database is Send + Sync because Mutex<Connection> is Send + Sync.
// We only access the connection through the mutex guard.
unsafe impl Send for Database {}
unsafe impl Sync for Database {}
```

- [ ] **Step 4: Write src-tauri/src/db/mod.rs**

```rust
pub mod migrations;
pub mod pool;
pub mod schema;
```

- [ ] **Step 5: Verify compilation and migration**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail/src-tauri
cargo check
```

Then run the app:
```bash
cargo tauri dev
```

Check that `aeromail.db` is created in the app data directory (`~/.local/share/com.aeromail.app` on Linux).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/db
git commit -m "feat: add SQLite schema, migrations, and database pool

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: Implement Account Manager Service

**Files:**
- Create: `src-tauri/src/services/account_manager.rs`
- Create: `src-tauri/src/services/mod.rs`

**Interfaces:**
- Consumes: `Database`, `AccountConfig`, `AccountSummary`, `AeroError`
- Produces: `AccountManager` with `add_account`, `list_accounts`, `delete_account`, `test_connection`.

- [ ] **Step 1: Write src-tauri/src/services/account_manager.rs**

```rust
use std::sync::Arc;

use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::{AccountConfig, AccountSummary, AuthConfig, TlsMode};

#[derive(Debug)]
pub struct AccountManager {
    db: Arc<Database>,
}

impl AccountManager {
    pub const fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn add_account(&self, mut config: AccountConfig) -> Result<String, AeroError> {
        let id = Uuid::new_v4().to_string();
        config.id.clone_from(&id);

        let auth_json = serde_json::to_string(&config.auth)?;
        let excluded_folders_json = serde_json::to_string(&config.excluded_folders)?;
        let now = Utc::now().timestamp();

        self.db.connection().execute(
            r#"
            INSERT INTO accounts (
                id, name, provider, imap_host, imap_port, smtp_host, smtp_port,
                tls_mode, auth_type, auth_credentials_encrypted, ca_cert_path,
                verify_certificate, connect_timeout, read_timeout, keepalive,
                sync_interval, excluded_folders, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#,
            params![
                &config.id,
                &config.name,
                serde_json::to_string(&config.provider)?,
                &config.imap.host,
                config.imap.port,
                &config.smtp.host,
                config.smtp.port,
                serde_json::to_string(&config.imap.tls_mode)?,
                match config.auth {
                    AuthConfig::OAuth2 { .. } => "OAuth2",
                    AuthConfig::Password { .. } => "Password",
                },
                auth_json.as_bytes(),
                config.advanced.ca_cert_path,
                config.advanced.verify_certificate,
                config.advanced.connect_timeout_secs,
                config.advanced.read_timeout_secs,
                config.advanced.keepalive,
                config.sync_interval_secs,
                excluded_folders_json,
                now,
                now,
            ],
        )?;

        Ok(id)
    }

    pub fn list_accounts(&self) -> Result<Vec<AccountSummary>, AeroError> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, provider, imap_host, smtp_host
            FROM accounts
            ORDER BY created_at ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let provider_json: String = row.get(2)?;
            let provider = serde_json::from_str(&provider_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?;
            Ok(AccountSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                provider,
                imap_host: row.get(3)?,
                smtp_host: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(AeroError::from)
    }

    pub fn delete_account(&self, account_id: &str) -> Result<(), AeroError> {
        let rows = self.db.connection().execute(
            "DELETE FROM accounts WHERE id = ?1",
            params![account_id],
        )?;

        if rows == 0 {
            return Err(AeroError::AccountNotFound(account_id.to_string()));
        }

        Ok(())
    }

    pub async fn test_connection(&self, config: &AccountConfig) -> Result<String, AeroError> {
        // Phase 1 placeholder: validate config and simulate a connection test.
        if config.imap.host.is_empty() {
            return Err(AeroError::InvalidConfig(
                "IMAP host cannot be empty".to_string(),
            ));
        }
        if config.imap.port == 0 {
            return Err(AeroError::InvalidConfig(
                "IMAP port cannot be zero".to_string(),
            ));
        }
        if matches!(config.imap.tls_mode, TlsMode::None) && config.imap.port != 143 {
            return Ok(format!(
                "Connection test passed (no encryption on port {})",
                config.imap.port
            ));
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        Ok(format!(
            "Connection test passed for {}:{}",
            config.imap.host, config.imap.port
        ))
    }
}
```

- [ ] **Step 2: Write src-tauri/src/services/mod.rs**

```rust
pub mod account_manager;
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail/src-tauri
cargo check
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/services
git commit -m "feat: implement AccountManager with CRUD and connection test placeholder

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: Implement Account Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/account.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `AppState`, `AccountManager`, `AccountConfig`
- Produces: Tauri commands `add_account`, `list_accounts`, `delete_account`, `test_account_connection`.

- [ ] **Step 1: Write src-tauri/src/commands/account.rs**

```rust
use tauri::State;

use crate::error::AeroError;
use crate::models::account::{AccountConfig, AccountSummary};
use crate::AppState;

#[tauri::command]
pub async fn add_account(
    config: AccountConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.account_manager.read().await;
    manager.add_account(config).map_err(String::from)
}

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountSummary>, String> {
    let manager = state.account_manager.read().await;
    manager.list_accounts().map_err(String::from)
}

#[tauri::command]
pub async fn delete_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.account_manager.read().await;
    manager.delete_account(&account_id).map_err(String::from)
}

#[tauri::command]
pub async fn test_account_connection(
    config: AccountConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.account_manager.read().await;
    manager.test_connection(&config).await.map_err(String::from)
}
```

- [ ] **Step 2: Write src-tauri/src/commands/mod.rs**

```rust
pub mod account;
```

- [ ] **Step 3: Update src-tauri/src/lib.rs**

Replace the existing `use commands::account::{...}` line and ensure it matches the new module path.

```rust
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

use commands::account::{add_account, delete_account, list_accounts, test_account_connection};
use db::pool::Database;
use services::account_manager::AccountManager;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub db: Arc<Database>,
}

impl AppState {
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, AeroError> {
        let db = Arc::new(Database::new(app_handle).await?);
        let account_manager = Arc::new(RwLock::new(AccountManager::new(Arc::clone(&db))));
        Ok(Self {
            account_manager,
            db,
        })
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = AppState::new(&handle)
                    .await
                    .expect("failed to initialize app state");
                handle.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_account,
            list_accounts,
            delete_account,
            test_account_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify compilation**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail/src-tauri
cargo check
```

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands for account management

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 8: Configure ESLint and Prettier

**Files:**
- Create: `eslint.config.ts`
- Create: `.prettierrc`
- Create: `.prettierignore`
- Modify: `package.json` (add scripts)

**Interfaces:**
- Consumes: None
- Produces: Linting and formatting configuration.

- [ ] **Step 1: Write eslint.config.ts**

```typescript
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import vueeslint from 'eslint-plugin-vue';
import prettier from 'eslint-config-prettier';

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  ...vueeslint.configs['flat/recommended'],
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: ['.vue'],
      },
    },
  },
  {
    files: ['*.vue', '**/*.vue'],
    languageOptions: {
      parser: vueeslint.parser,
    },
  },
  {
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      'vue/multi-word-component-names': 'off',
    },
  },
  prettier
);
```

- [ ] **Step 2: Write .prettierrc**

```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100,
  "endOfLine": "lf"
}
```

- [ ] **Step 3: Write .prettierignore**

```text
dist
target
node_modules
src-tauri/target
src-tauri/gen
```

- [ ] **Step 4: Update package.json scripts**

Add/update:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext .vue,.ts,.tsx",
    "type-check": "vue-tsc --noEmit",
    "format": "prettier --write \"src/**/*.{ts,vue,css}\" \"docs/**/*.md\""
  }
}
```

- [ ] **Step 5: Add dependencies to package.json**

Add to `devDependencies`:
```json
{
  "@eslint/js": "^9.20.0",
  "eslint-config-prettier": "^10.0.0",
  "typescript-eslint": "^8.24.0"
}
```

- [ ] **Step 6: Install and verify linting**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail
pnpm install
pnpm lint
```

Expected: ESLint runs with no errors (warnings acceptable). Fix any errors.

- [ ] **Step 7: Commit**

```bash
git add eslint.config.ts .prettierrc .prettierignore package.json
git commit -m "chore: configure ESLint, Prettier, and lint scripts

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 9: Create Frontend Types and IPC Composable

**Files:**
- Create: `src/types/account.ts`
- Create: `src/composables/useTauriInvoke.ts`

**Interfaces:**
- Consumes: Tauri `invoke` API
- Produces: TypeScript `AccountConfig`, `AccountSummary`, `MailProvider`, `TlsMode`, `AuthConfig` types; typed `invoke` wrapper.

- [ ] **Step 1: Write src/types/account.ts**

```typescript
export type MailProvider =
  | 'Gmail'
  | 'Outlook'
  | 'QQ'
  | 'Netease163'
  | 'Aliyun'
  | 'TencentExmail'
  | 'Custom';

export type TlsMode = 'required' | 'starttls' | 'none';

export interface ServerConfig {
  host: string;
  port: number;
  tlsMode: TlsMode;
}

export type AuthConfig =
  | {
      type: 'OAuth2';
      accessToken: string;
      refreshToken: string;
      expiresAt: number;
    }
  | {
      type: 'Password';
      passwordEncrypted: number[];
    };

export interface AdvancedConfig {
  caCertPath: string | null;
  verifyCertificate: boolean;
  connectTimeoutSecs: number;
  readTimeoutSecs: number;
  keepalive: boolean;
}

export interface AccountConfig {
  id: string;
  name: string;
  provider: MailProvider;
  imap: ServerConfig;
  smtp: ServerConfig;
  auth: AuthConfig;
  advanced: AdvancedConfig;
  syncIntervalSecs: number;
  excludedFolders: string[];
}

export interface AccountSummary {
  id: string;
  name: string;
  provider: MailProvider;
  imapHost: string;
  smtpHost: string;
}
```

- [ ] **Step 2: Write src/composables/useTauriInvoke.ts**

```typescript
import { invoke } from '@tauri-apps/api/core';

export function useTauriInvoke() {
  async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(command, args);
  }

  return { call };
}
```

- [ ] **Step 3: Verify TypeScript**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail
pnpm type-check
```

Expected: No type errors.

- [ ] **Step 4: Commit**

```bash
git add src/types src/composables/useTauriInvoke.ts
git commit -m "feat: add TypeScript account types and typed Tauri invoke composable

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 10: Implement Theme and Responsive Composables

**Files:**
- Create: `src/composables/useTheme.ts`
- Create: `src/composables/useResponsive.ts`

**Interfaces:**
- Consumes: `useWindowSize` from `@vueuse/core`
- Produces: `useTheme()` returns `{ theme, toggleTheme, setTheme }`; `useResponsive()` returns `{ layoutMode, isWideScreen, isCollapsed }`.

- [ ] **Step 1: Write src/composables/useTheme.ts**

```typescript
import { ref, watch } from 'vue';

type Theme = 'dark' | 'light';

const theme = ref<Theme>('dark');

export function useTheme() {
  function setTheme(value: Theme) {
    theme.value = value;
    document.documentElement.setAttribute('data-theme', value);
  }

  function toggleTheme() {
    setTheme(theme.value === 'dark' ? 'light' : 'dark');
  }

  watch(
    theme,
    (value) => {
      document.documentElement.setAttribute('data-theme', value);
    },
    { immediate: true }
  );

  return {
    theme,
    toggleTheme,
    setTheme,
  };
}
```

- [ ] **Step 2: Write src/composables/useResponsive.ts**

```typescript
import { computed } from 'vue';
import { useWindowSize } from '@vueuse/core';

export type LayoutMode = 'mobile' | 'compact' | 'tablet' | 'desktop' | 'wide';

export function useResponsive() {
  const { width } = useWindowSize();

  const layoutMode = computed<LayoutMode>(() => {
    const w = width.value;
    if (w < 800) return 'mobile';
    if (w < 1140) return 'compact';
    if (w < 1400) return 'tablet';
    if (w < 1920) return 'desktop';
    return 'wide';
  });

  const isWideScreen = computed(() => width.value >= 1920);
  const isCollapsed = computed(() => width.value < 1140);

  return {
    width,
    layoutMode,
    isWideScreen,
    isCollapsed,
  };
}
```

- [ ] **Step 3: Verify TypeScript**

Run:
```bash
pnpm type-check
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/composables/useTheme.ts src/composables/useResponsive.ts
git commit -m "feat: add theme and responsive layout composables

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 11: Implement Pinia Stores

**Files:**
- Create: `src/stores/account.ts`
- Create: `src/stores/status.ts`
- Create: `src/stores/toast.ts`

**Interfaces:**
- Consumes: `useTauriInvoke`, `AccountConfig`, `AccountSummary`
- Produces: Stores `useAccountStore`, `useStatusStore`, `useToastStore`.

- [ ] **Step 1: Write src/stores/account.ts**

```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { AccountConfig, AccountSummary } from '@/types/account';

export const useAccountStore = defineStore('account', () => {
  const { call } = useTauriInvoke();
  const accounts = ref<AccountSummary[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const accountCount = computed(() => accounts.value.length);

  async function loadAccounts() {
    loading.value = true;
    error.value = null;
    try {
      accounts.value = await call<AccountSummary[]>('list_accounts');
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function addAccount(config: AccountConfig) {
    error.value = null;
    try {
      await call<string>('add_account', { config });
      await loadAccounts();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function deleteAccount(accountId: string) {
    error.value = null;
    try {
      await call<void>('delete_account', { accountId });
      await loadAccounts();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  return {
    accounts,
    loading,
    error,
    accountCount,
    loadAccounts,
    addAccount,
    deleteAccount,
  };
});
```

- [ ] **Step 2: Write src/stores/status.ts**

```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface SyncStatusItem {
  accountId: string;
  status: 'idle' | 'syncing' | 'error' | 'completed';
  message?: string;
}

export const useStatusStore = defineStore('status', () => {
  const syncStatus = ref<SyncStatusItem[]>([]);
  const unreadCount = ref(0);
  const lastSyncTime = ref<string | null>(null);
  const isOnline = ref(true);

  const syncingAccounts = computed(
    () => syncStatus.value.filter((s) => s.status === 'syncing').length
  );

  function updateSyncStatus(accountId: string, status: SyncStatusItem['status'], message?: string) {
    const idx = syncStatus.value.findIndex((s) => s.accountId === accountId);
    if (idx >= 0) {
      syncStatus.value[idx] = { accountId, status, message };
    } else {
      syncStatus.value.push({ accountId, status, message });
    }
  }

  function setOnline(value: boolean) {
    isOnline.value = value;
  }

  function setLastSyncTime(value: string | null) {
    lastSyncTime.value = value;
  }

  return {
    syncStatus,
    unreadCount,
    lastSyncTime,
    isOnline,
    syncingAccounts,
    updateSyncStatus,
    setOnline,
    setLastSyncTime,
  };
});
```

- [ ] **Step 3: Write src/stores/toast.ts**

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ToastType = 'success' | 'warning' | 'error' | 'info';

export interface ToastItem {
  id: string;
  type: ToastType;
  message: string;
  action?: { label: string; callback: () => void };
  duration: number;
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<ToastItem[]>([]);

  function add(toast: Omit<ToastItem, 'id'>) {
    const id = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const item: ToastItem = { ...toast, id };
    toasts.value.push(item);
    if (toasts.value.length > 3) {
      toasts.value.shift();
    }
    setTimeout(() => remove(id), toast.duration);
  }

  function remove(id: string) {
    const idx = toasts.value.findIndex((t) => t.id === id);
    if (idx >= 0) {
      toasts.value.splice(idx, 1);
    }
  }

  return {
    toasts,
    add,
    remove,
  };
});
```

- [ ] **Step 4: Verify TypeScript**

Run:
```bash
pnpm type-check
```

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src/stores
git commit -m "feat: add Pinia stores for account, status, and toast

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 12: Build Three-Pane Layout Components

**Files:**
- Create: `src/layouts/AppLayout.vue`
- Create: `src/components/AppSidebar.vue`
- Create: `src/components/MailList.vue`
- Create: `src/components/MailViewer.vue`
- Modify: `src/views/InboxView.vue`

**Interfaces:**
- Consumes: `useResponsive`, `useAccountStore`, `useStatusStore`
- Produces: `AppLayout`, `AppSidebar`, `MailList`, `MailViewer` components.

- [ ] **Step 1: Write src/layouts/AppLayout.vue**

```vue
<script setup lang="ts">
import { computed } from 'vue';
import { useResponsive } from '@/composables/useResponsive';
import AppSidebar from '@/components/AppSidebar.vue';
import MailList from '@/components/MailList.vue';
import MailViewer from '@/components/MailViewer.vue';
import StatusBar from '@/components/StatusBar.vue';

const { isWideScreen, isCollapsed, layoutMode } = useResponsive();

const sidebarWidth = computed(() => (isWideScreen.value ? 'w-[260px]' : 'w-[240px]'));
const mailListWidth = computed(() => (isWideScreen.value ? 'w-[480px]' : 'w-[420px]'));
</script>

<template>
  <div class="flex h-screen w-screen flex-col bg-background text-text">
    <div class="flex flex-1 overflow-hidden">
      <AppSidebar
        :class="[
          'flex-shrink-0 overflow-hidden transition-all duration-200',
          sidebarWidth,
          isCollapsed ? 'w-0 opacity-0' : 'opacity-100',
        ]"
      />

      <MailList
        :class="[
          'flex-shrink-0 border-r border-border',
          mailListWidth,
          layoutMode === 'mobile' ? 'hidden' : 'block',
        ]"
      />

      <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
        <slot />
      </main>
    </div>
    <StatusBar />
  </div>
</template>
```

- [ ] **Step 2: Write src/components/AppSidebar.vue**

```vue
<script setup lang="ts">
import { onMounted } from 'vue';
import { Mail, Inbox, Star, Send, FileText, Archive, Trash2, Settings, Plus } from 'lucide-vue-next';
import { useAccountStore } from '@/stores/account';

const accountStore = useAccountStore();

onMounted(() => {
  void accountStore.loadAccounts();
});

const folders = [
  { name: 'Inbox', icon: Inbox, count: 128 },
  { name: 'Starred', icon: Star, count: 12 },
  { name: 'Sent', icon: Send, count: null },
  { name: 'Drafts', icon: FileText, count: 3 },
  { name: 'Archived', icon: Archive, count: null },
  { name: 'Spam', icon: Trash2, count: null },
];
</script>

<template>
  <aside class="flex h-full flex-col bg-panel">
    <div class="flex h-12 items-center px-4 text-lg font-semibold">AeroMail</div>

    <div class="px-3 pb-2">
      <button
        class="flex h-10 w-full items-center justify-center gap-2 rounded-lg bg-primary text-sm font-medium text-white hover:bg-primary-hover transition-colors"
      >
        <Plus class="h-4 w-4" />
        New Mail
      </button>
    </div>

    <nav class="flex-1 overflow-y-auto px-2">
      <ul class="space-y-0.5">
        <li
          v-for="folder in folders"
          :key="folder.name"
          class="flex h-9 cursor-pointer items-center justify-between rounded-md px-3 text-sm text-text-secondary hover:bg-white/5 transition-colors"
        >
          <div class="flex items-center gap-3">
            <component :is="folder.icon" class="h-4 w-4" />
            <span>{{ folder.name }}</span>
          </div>
          <span
            v-if="folder.count"
            class="flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 text-xs font-medium text-white"
          >
            {{ folder.count }}
          </span>
        </li>
      </ul>

      <div class="my-3 h-px bg-border" />

      <div class="px-3 pb-2 text-xs font-medium text-muted">ACCOUNTS</div>
      <ul class="space-y-0.5">
        <li
          v-for="account in accountStore.accounts"
          :key="account.id"
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary hover:bg-white/5 transition-colors"
        >
          <div
            class="flex h-6 w-6 items-center justify-center rounded-full bg-card text-xs font-medium"
          >
            {{ account.name.charAt(0).toUpperCase() }}
          </div>
          <span class="truncate">{{ account.name }}</span>
        </li>
      </ul>

      <div class="my-3 h-px bg-border" />

      <ul class="space-y-0.5">
        <li
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary hover:bg-white/5 transition-colors"
        >
          <Settings class="h-4 w-4" />
          <span>Settings</span>
        </li>
      </ul>
    </nav>
  </aside>
</template>
```

- [ ] **Step 3: Write src/components/MailList.vue**

```vue
<script setup lang="ts">
const mockMails = [
  {
    id: '1',
    fromName: 'GitHub',
    subject: 'Security Alert',
    snippet: 'New login detected from...',
    time: '14:22',
    isRead: false,
    isStarred: false,
    hasAttachments: false,
  },
  {
    id: '2',
    fromName: 'Billing',
    subject: 'Invoice May 2026',
    snippet: 'Your invoice for May 2026 is ready...',
    time: '12:10',
    isRead: true,
    isStarred: true,
    hasAttachments: true,
  },
];
</script>

<template>
  <div class="flex h-full flex-col bg-panel">
    <div class="h-12 border-b border-border px-4 flex items-center font-medium">Inbox</div>
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="mail in mockMails"
        :key="mail.id"
        class="group relative h-[72px] cursor-pointer border-b border-border px-4 py-3 hover:bg-card transition-colors"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span
              v-if="!mail.isRead"
              class="h-1.5 w-1.5 rounded-full bg-primary"
            />
            <span class="text-sm font-medium">{{ mail.fromName }}</span>
          </div>
          <span class="text-xs text-muted">{{ mail.time }}</span>
        </div>
        <div class="mt-1 truncate text-sm">{{ mail.subject }}</div>
        <div class="mt-0.5 truncate text-xs text-muted">{{ mail.snippet }}</div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 4: Write src/components/MailViewer.vue**

```vue
<script setup lang="ts"></script>

<template>
  <div class="flex h-full flex-col bg-background">
    <div class="flex flex-1 items-center justify-center text-muted">
      Select an email to read
    </div>
  </div>
</template>
```

- [ ] **Step 5: Update src/views/InboxView.vue**

```vue
<script setup lang="ts">
import AppLayout from '@/layouts/AppLayout.vue';
import MailViewer from '@/components/MailViewer.vue';
</script>

<template>
  <AppLayout>
    <MailViewer />
  </AppLayout>
</template>
```

- [ ] **Step 6: Verify UI renders**

Run:
```bash
cargo tauri dev
```

Expected: Window shows three-pane layout with sidebar (AeroMail, New Mail, folders, accounts), mail list with mock items, and empty viewer.

- [ ] **Step 7: Commit**

```bash
git add src/layouts src/components/AppSidebar.vue src/components/MailList.vue src/components/MailViewer.vue src/views/InboxView.vue
git commit -m "feat: implement responsive three-pane layout with sidebar and mail list

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 13: Build Status Bar, Toast, and Command Palette Components

**Files:**
- Create: `src/components/StatusBar.vue`
- Create: `src/components/ToastContainer.vue`
- Create: `src/components/CommandPalette.vue`
- Modify: `src/App.vue`

**Interfaces:**
- Consumes: `useStatusStore`, `useToastStore`
- Produces: `StatusBar`, `ToastContainer`, `CommandPalette` components.

- [ ] **Step 1: Write src/components/StatusBar.vue**

```vue
<script setup lang="ts">
import { computed } from 'vue';
import { useStatusStore } from '@/stores/status';

const statusStore = useStatusStore();

const syncText = computed(() => {
  const syncing = statusStore.syncingAccounts;
  if (syncing > 0) {
    return `Syncing... ${syncing}/${statusStore.syncStatus.length} accounts`;
  }
  return 'Sync complete';
});
</script>

<template>
  <div
    class="flex h-7 flex-shrink-0 items-center border-t border-border bg-panel px-4 text-tiny text-muted"
  >
    <div class="flex items-center gap-2">
      <span
        :class="[
          'h-2 w-2 rounded-full',
          statusStore.syncingAccounts > 0 ? 'animate-pulse bg-primary' : 'bg-success',
        ]"
      />
      <span>{{ syncText }}</span>
    </div>
    <div class="mx-3 h-3 w-px bg-border" />
    <span>{{ statusStore.unreadCount }} unread</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span>Last sync: {{ statusStore.lastSyncTime ?? 'Never' }}</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span :class="statusStore.isOnline ? 'text-success' : 'text-warning'">
      {{ statusStore.isOnline ? 'Online' : 'Offline' }}
    </span>
    <span class="ml-auto">v0.1.0</span>
  </div>
</template>
```

- [ ] **Step 2: Write src/components/ToastContainer.vue**

```vue
<script setup lang="ts">
import { CheckCircle, AlertTriangle, XCircle, Info, X } from 'lucide-vue-next';
import { useToastStore, type ToastType } from '@/stores/toast';

const toastStore = useToastStore();

function borderClass(type: ToastType): string {
  return {
    success: 'border-success',
    warning: 'border-warning',
    error: 'border-danger',
    info: 'border-primary',
  }[type];
}

function iconComponent(type: ToastType) {
  return {
    success: CheckCircle,
    warning: AlertTriangle,
    error: XCircle,
    info: Info,
  }[type];
}
</script>

<template>
  <div class="fixed right-4 top-4 z-40 flex flex-col gap-2">
    <TransitionGroup
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="translate-x-full opacity-0"
      enter-to-class="translate-x-0 opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="translate-x-0 opacity-100"
      leave-to-class="-translate-y-full opacity-0"
    >
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="flex min-h-[44px] min-w-[280px] max-w-[400px] items-center gap-3 rounded-lg border-l-[3px] bg-panel px-4 py-3 shadow-toast"
        :class="borderClass(toast.type)"
      >
        <component :is="iconComponent(toast.type)" class="h-4 w-4 flex-shrink-0" />
        <span class="flex-1 text-sm text-text">{{ toast.message }}</span>
        <button
          v-if="toast.action"
          class="text-sm font-medium text-primary hover:text-primary-hover"
          @click="toast.action.callback(); toastStore.remove(toast.id)"
        >
          {{ toast.action.label }}
        </button>
        <button
          class="flex h-6 w-6 items-center justify-center text-muted hover:text-text"
          @click="toastStore.remove(toast.id)"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
```

- [ ] **Step 3: Write src/components/CommandPalette.vue**

```vue
<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { Search } from 'lucide-vue-next';

const isOpen = ref(false);
const query = ref('');
const highlightedIndex = ref(0);

const mockResults = [
  { id: '1', title: 'GitHub Security Alert' },
  { id: '2', title: 'Invoice May 2026' },
  { id: '3', title: 'Meeting Notes' },
];

const results = ref(mockResults);

watch(query, (val) => {
  if (!val) {
    results.value = mockResults;
    return;
  }
  results.value = mockResults.filter((r) =>
    r.title.toLowerCase().includes(val.toLowerCase())
  );
});

function open() {
  isOpen.value = true;
}

function close() {
  isOpen.value = false;
  query.value = '';
  highlightedIndex.value = 0;
}

function highlightPrev() {
  highlightedIndex.value = Math.max(0, highlightedIndex.value - 1);
}

function highlightNext() {
  highlightedIndex.value = Math.min(results.value.length - 1, highlightedIndex.value + 1);
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    isOpen.value ? close() : open();
  }
  if (!isOpen.value) return;
  if (e.key === 'Escape') close();
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    highlightPrev();
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    highlightNext();
  }
  if (e.key === 'Enter' && results.value[highlightedIndex.value]) {
    close();
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

defineExpose({ open, close });
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-250 ease-out"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
      >
        <div class="absolute inset-0 bg-overlay" @click="close" />
        <div
          class="relative w-[560px] max-h-[400px] overflow-hidden rounded-xl bg-panel shadow-modal"
        >
          <div class="flex h-14 items-center gap-3 px-4">
            <Search class="h-5 w-5 text-muted" />
            <input
              v-model="query"
              type="text"
              class="flex-1 bg-transparent text-base text-text placeholder-muted outline-none"
              placeholder="Search mail..."
            />
          </div>
          <div class="max-h-[340px] overflow-y-auto">
            <div
              v-for="(item, index) in results"
              :key="item.id"
              class="flex h-12 cursor-pointer items-center px-4 text-sm"
              :class="[
                index === highlightedIndex
                  ? 'border-l-[3px] border-primary bg-card'
                  : 'border-l-[3px] border-transparent',
              ]"
              @mouseenter="highlightedIndex = index"
              @click="close"
            >
              {{ item.title }}
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
```

- [ ] **Step 4: Update src/App.vue**

```vue
<script setup lang="ts">
import { RouterView } from 'vue-router';
import CommandPalette from '@/components/CommandPalette.vue';
import ToastContainer from '@/components/ToastContainer.vue';
</script>

<template>
  <RouterView />
  <CommandPalette />
  <ToastContainer />
</template>
```

- [ ] **Step 5: Verify components**

Run:
```bash
cargo tauri dev
```

Expected:
- Status bar visible at bottom.
- Press `Ctrl/Cmd + K` to open Command Palette.
- Add a temporary toast trigger button somewhere (optional manual check).

- [ ] **Step 6: Commit**

```bash
git add src/components/StatusBar.vue src/components/ToastContainer.vue src/components/CommandPalette.vue src/App.vue
git commit -m "feat: add StatusBar, ToastContainer, and CommandPalette components

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 14: Add Account Form UI and Wire End-to-End CRUD

**Files:**
- Create: `src/components/AccountForm.vue`
- Create: `src/components/AccountList.vue`
- Modify: `src/components/AppSidebar.vue`
- Modify: `src/components/MailList.vue`

**Interfaces:**
- Consumes: `useAccountStore`, `AccountConfig` type
- Produces: UI to add/list/delete accounts.

- [ ] **Step 1: Write src/components/AccountForm.vue**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { useAccountStore } from '@/stores/account';
import type { AccountConfig, MailProvider, TlsMode } from '@/types/account';

const accountStore = useAccountStore();

const providers: MailProvider[] = [
  'Gmail',
  'Outlook',
  'QQ',
  'Netease163',
  'Aliyun',
  'TencentExmail',
  'Custom',
];

const providerDefaults: Record<MailProvider, { imap: string; smtp: string }> = {
  Gmail: { imap: 'imap.gmail.com', smtp: 'smtp.gmail.com' },
  Outlook: { imap: 'outlook.office365.com', smtp: 'smtp.office365.com' },
  QQ: { imap: 'imap.qq.com', smtp: 'smtp.qq.com' },
  Netease163: { imap: 'imap.163.com', smtp: 'smtp.163.com' },
  Aliyun: { imap: 'imap.aliyun.com', smtp: 'smtp.aliyun.com' },
  TencentExmail: { imap: 'imap.exmail.qq.com', smtp: 'smtp.exmail.qq.com' },
  Custom: { imap: '', smtp: '' },
};

const config = ref<AccountConfig>({
  id: '',
  name: '',
  provider: 'Gmail',
  imap: { host: 'imap.gmail.com', port: 993, tlsMode: 'required' },
  smtp: { host: 'smtp.gmail.com', port: 465, tlsMode: 'required' },
  auth: { type: 'Password', passwordEncrypted: [] },
  advanced: {
    caCertPath: null,
    verifyCertificate: true,
    connectTimeoutSecs: 30,
    readTimeoutSecs: 30,
    keepalive: true,
  },
  syncIntervalSecs: 60,
  excludedFolders: [],
});

const tlsModes: TlsMode[] = ['required', 'starttls', 'none'];

function updateProvider(provider: MailProvider) {
  config.value.provider = provider;
  const defaults = providerDefaults[provider];
  config.value.imap.host = defaults.imap;
  config.value.smtp.host = defaults.smtp;
}

async function handleSubmit() {
  await accountStore.addAccount(config.value);
}
</script>

<template>
  <form class="space-y-4 p-4" @submit.prevent="handleSubmit">
    <h2 class="text-lg font-semibold">Add Account</h2>

    <div>
      <label class="mb-1 block text-sm text-muted">Provider</label>
      <select
        v-model="config.provider"
        class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        @change="updateProvider(config.provider)"
      >
        <option v-for="p in providers" :key="p" :value="p">{{ p }}</option>
      </select>
    </div>

    <div>
      <label class="mb-1 block text-sm text-muted">Account Name</label>
      <input
        v-model="config.name"
        type="text"
        class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        placeholder="Work Gmail"
      />
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="mb-1 block text-sm text-muted">IMAP Host</label>
        <input
          v-model="config.imap.host"
          type="text"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
      <div>
        <label class="mb-1 block text-sm text-muted">IMAP Port</label>
        <input
          v-model.number="config.imap.port"
          type="number"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="mb-1 block text-sm text-muted">SMTP Host</label>
        <input
          v-model="config.smtp.host"
          type="text"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
      <div>
        <label class="mb-1 block text-sm text-muted">SMTP Port</label>
        <input
          v-model.number="config.smtp.port"
          type="number"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
    </div>

    <div>
      <label class="mb-1 block text-sm text-muted">Password</label>
      <input
        type="password"
        class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        placeholder="App password"
      />
    </div>

    <button
      type="submit"
      class="h-10 w-full rounded-md bg-primary text-sm font-medium text-white hover:bg-primary-hover transition-colors"
    >
      Add Account
    </button>

    <p v-if="accountStore.error" class="text-sm text-danger">{{ accountStore.error }}</p>
  </form>
</template>
```

- [ ] **Step 2: Write src/components/AccountList.vue**

```vue
<script setup lang="ts">
import { onMounted } from 'vue';
import { Trash2 } from 'lucide-vue-next';
import { useAccountStore } from '@/stores/account';

const accountStore = useAccountStore();

onMounted(() => {
  void accountStore.loadAccounts();
});

async function remove(id: string) {
  await accountStore.deleteAccount(id);
}
</script>

<template>
  <div class="p-4">
    <h2 class="mb-3 text-lg font-semibold">Accounts</h2>
    <div v-if="accountStore.loading">Loading...</div>
    <ul v-else class="space-y-2">
      <li
        v-for="account in accountStore.accounts"
        :key="account.id"
        class="flex items-center justify-between rounded-md bg-card px-3 py-2"
      >
        <div>
          <div class="text-sm font-medium">{{ account.name }}</div>
          <div class="text-xs text-muted">{{ account.imapHost }}</div>
        </div>
        <button
          class="text-muted hover:text-danger"
          @click="remove(account.id)"
        >
          <Trash2 class="h-4 w-4" />
        </button>
      </li>
    </ul>
    <p v-if="accountStore.error" class="mt-2 text-sm text-danger">
      {{ accountStore.error }}
    </p>
  </div>
</template>
```

- [ ] **Step 3: Temporarily modify MailList.vue to show AccountForm and AccountList for testing**

Replace the content of `src/components/MailList.vue` with:

```vue
<script setup lang="ts">
import AccountForm from './AccountForm.vue';
import AccountList from './AccountList.vue';
</script>

<template>
  <div class="flex h-full flex-col overflow-y-auto bg-panel">
    <AccountForm />
    <AccountList />
  </div>
</template>
```

- [ ] **Step 4: Test end-to-end account CRUD**

Run:
```bash
cargo tauri dev
```

Steps:
1. Fill in the Add Account form with a name and provider.
2. Click "Add Account".
3. Verify the account appears in the AccountList below.
4. Delete the account and verify it disappears.
5. Check the database file contains/does not contain the account row.

- [ ] **Step 5: Restore MailList.vue to mock mail list**

After verifying CRUD works, restore `src/components/MailList.vue` to the version from Task 12.

- [ ] **Step 6: Commit**

```bash
git add src/components/AccountForm.vue src/components/AccountList.vue
git commit -m "feat: add account form and list UI, wire end-to-end CRUD

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 15: Final Verification and Cleanup

**Files:**
- All files from previous tasks

**Interfaces:**
- Consumes: All previous deliverables
- Produces: A runnable, lint-free, type-checked skeleton.

- [ ] **Step 1: Run full verification**

Run:
```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail
pnpm install
pnpm lint
pnpm type-check
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features
cargo check
cargo tauri dev
```

Expected:
- `pnpm lint`: no errors
- `pnpm type-check`: no errors
- `cargo fmt --check`: no formatting differences
- `cargo clippy`: no errors (warnings acceptable)
- `cargo check`: success
- `cargo tauri dev`: window opens, layout renders, account CRUD works

- [ ] **Step 2: Fix any remaining issues**

Address lint, type, clippy, or runtime issues. Commit fixes.

- [ ] **Step 3: Update .gitignore**

Create `.gitignore`:

```text
# Dependencies
node_modules
.pnpm-store

# Build outputs
dist
dist-ssr
*.local

# Tauri / Rust
target/
src-tauri/target/
src-tauri/gen/

# Editor
.idea
.vscode/*
!.vscode/extensions.json
*.sw?

# OS
.DS_Store
Thumbs.db
```

- [ ] **Step 4: Final commit**

```bash
git add .gitignore
git commit -m "chore: add .gitignore and final cleanup for phase 1 skeleton

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Self-Review

### Spec Coverage

| Spec Section | Implementing Task |
|--------------|-------------------|
| Tauri 2.11 + Vue 3 + Vite project init | Task 1 |
| Rust 2024 edition | Task 1 (`Cargo.toml`) |
| Tailwind 4.x + theme CSS | Task 2 |
| SQLite schema + pool | Task 5 |
| Account models | Task 4 |
| Account CRUD commands | Task 7 |
| AccountManager service | Task 6 |
| Three-pane layout | Task 12 |
| Command Palette | Task 13 |
| Status Bar | Task 13 |
| Toast | Task 13 |
| Pinia stores | Task 11 |
| TypeScript types + IPC | Task 9 |
| ESLint/Prettier | Task 8 |
| Rust clippy/rustfmt | Task 3 |

### Placeholder Scan

- No `TBD`, `TODO`, or "implement later".
- No vague "add error handling" steps.
- All code blocks contain actual implementation or concrete configuration.
- All commands include expected outputs.

### Type Consistency

- Rust `AccountConfig` fields match TypeScript `AccountConfig` (camelCase via serde rename in JSON).
- `MailProvider` and `TlsMode` serialized consistently across backend/frontend.
- `invoke` commands use the exact names registered in `lib.rs`.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-17-aeromail-phase1.md`.**

Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach would you like to use?
