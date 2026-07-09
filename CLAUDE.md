# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

AeroMail 是基于 Rust + Tauri v2 + Vue 3 + TypeScript 构建的跨平台桌面邮件客户端，采用"重后端、轻前端"架构：所有 IO 密集型（数据库、网络）和 CPU 密集型（邮件解析、搜索分词）任务由 Rust 后端处理，前端仅负责响应式渲染。

- 技术栈：Tauri v2、Vue 3 Composition API、Vite、Tailwind CSS v4、Pinia、vue-i18n、Rust 2024 edition、Tokio、rusqlite、Tantivy、imap、mail-parser、native-tls。
- 目标平台：Linux（原生 Wayland）、macOS、Windows。
- 数据存储：SQLite 本地缓存 + Tantivy 全文索引。
- 邮件协议：IMAP（收信/同步）、SMTP（发信）。

## 常用开发命令

### 环境要求

- Rust 1.85+（项目使用 `edition = "2024"`）
- Node.js 20+ + pnpm
- Linux 开发需安装 WebKitGTK 开发包（如 `webkit2gtk-4.1-devel`、`libgtk-3-devel`）

### 启动开发

```bash
# 安装前端依赖
pnpm install

# 启动 Tauri 开发模式（自动拉起 Vite 前端）
cargo tauri dev
```

### 构建

```bash
# 前端生产构建
pnpm build

# 完整应用打包（按平台输出 deb/appimage/dmg/msi/nsis）
cargo tauri build
```

### 代码质量

```bash
# 前端类型检查
pnpm type-check

# ESLint 检查
pnpm lint

# 前端代码格式化
pnpm format

# i18n key 一致性检查
pnpm i18n:check

# Rust 格式化（项目使用 rustfmt.toml 配置）
cargo fmt

# Rust 静态检查（Cargo.toml 中启用了 clippy 的 pedantic/nursery/unwrap_used/expect_used）
cargo clippy

# Rust 测试（当前仓库暂无测试文件）
cargo test

# 运行单个 Rust 测试
cargo test --package aeromail -- <test_name>
```

### Linux Wayland 运行

```bash
export GDK_BACKEND=wayland
export WEBKIT_DISABLE_COMPOSITING_MODE=0
cargo tauri dev
```

## 架构说明

### 进程与运行时

- **Webview 进程**：每个窗口独立进程，运行 Vue 3 前端。
- **Rust 主进程**：单进程多线程，负责 IPC 路由、状态管理、插件调度。
- **Tokio Runtime**：多线程线程池，调度 IMAP 同步、SMTP 发信、AI/翻译 API 调用等异步任务。

### 后端状态管理

后端状态通过 `src-tauri/src/lib.rs` 中的 `AppState` 集中管理，所有服务以 `Arc<RwLock<T>>` 形式挂载到 Tauri 应用状态：

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

Tauri Command 通过 `tauri::State<'_, AppState>` 访问这些服务。初始化在 `tauri::async_runtime::spawn` 中异步完成，失败会调用 `handle.exit(1)`。

### 后端目录结构

- `src-tauri/src/commands/`：Tauri IPC 命令入口，按领域分为 account、mail、sync、search、settings、ai、translation 等模块。
- `src-tauri/src/services/`：业务服务实现，每个领域一个模块。
  - `account_manager.rs`：账户 CRUD 与连接测试。
  - `sync/`：IMAP 同步引擎，每账户独立 Tokio Task。
  - `search/`：Tantivy 索引与搜索。
  - `ai/`：AI 助手服务（内置厂商预设、聊天上下文、API 调用）。
  - `translation/`：传统翻译 API 与 AI 翻译，含 SQLite 缓存。
- `src-tauri/src/models/`：前后端共享的 Rust 数据模型。
- `src-tauri/src/db/`：SQLite schema、连接池、迁移。

### 错误处理约定

后端统一使用 `src-tauri/src/error.rs` 中的 `AeroError`，并通过 `to_payload()` 转换为 `ErrorPayload { code, args }` 返回前端。前端负责根据 `code` 从 `src/i18n/locales/*.json` 映射为本地化文案。后端命令返回类型通常是 `Result<T, ErrorPayload>`。

新增错误类型时，需要同步：

1. 在 `AeroError` 中添加变体。
2. 在 `to_payload()` 中映射为错误码和参数。
3. 在前端 `src/i18n/locales/en.json` 和 `src/i18n/locales/zh-CN.json` 中添加对应 key，并执行 `pnpm i18n:check` 验证一致性。

### 数据库

- 使用 `rusqlite` + `bundled` feature，schema 定义在 `src-tauri/src/db/schema.rs`。
- 核心表：`accounts`、`folders`、`mails`、`attachments`、`drafts`、`settings`、`ai_providers`、`ai_chat_sessions`、`ai_chat_messages`、`translation_providers`、`translations`。
- 数据库文件位于 Tauri 应用数据目录，通过 `src-tauri/src/db/pool.rs` 中的 `Database` 封装访问。

### 搜索索引

- 使用 Tantivy 0.26，`tantivy-jieba` 提供中文分词。
- 索引目录：`app_data_dir/tantivy_index`。
- 索引字段包括邮件标题、正文纯文本、发件人名称/地址、日期、已读/星标状态。
- `SearchService` 提供 `search_mails()` 与 `index_pending_mails()` 能力。

### 同步引擎

- `SyncService` 为每个账户维护独立 Tokio Task。
- 流程：UIDVALIDITY 校验 → 获取本地最大 UID → IMAP FETCH 新增邮件 → `mail-parser` 解析 MIME → 写入 SQLite → 写入 Tantivy 索引。
- 失败时指数退避重试，网络恢复后自动续传。

### 前端架构

- `src/main.ts`：Vue 应用入口，默认设置 `data-theme="dark"`。
- `src/App.vue`：根组件，挂载 `AppLayout`。
- `src/router.ts`：Vue Router，当前路由：`/`（收件箱）、`/folder/:folderId`、`/accounts`、`/settings`。
- `src/layouts/AppLayout.vue`：三栏布局（Sidebar / MailList / RouterView）。
- `src/stores/`：Pinia 全局状态（account、ai、mail、settings、status、toast）。
- `src/composables/`：可复用逻辑（主题、响应式、搜索、AI 聊天、翻译、快捷键等）。
- `src/i18n/`：vue-i18n 实例 + 英文/简体中文 locale 文件。
- `src/types/`：前端 TypeScript 类型定义。

### HTML 邮件渲染安全

- 邮件 HTML 通过 `<iframe sandbox="allow-same-origin">` 渲染，**不启用 `allow-scripts`**，物理隔离 XSS。
- `srcdoc` 注入 Rust 处理后的 HTML，默认拦截外部 `<img>` 请求（替换为占位图），锁定 `color-scheme: light` 防止暗黑模式下邮件排版错乱。
- `tauri.conf.json` 中 CSP 已限制为：`default-src 'self'; img-src 'self' https: data:; style-src 'self' 'unsafe-inline'; script-src 'none';`。

### 国际化

- 前端使用 `vue-i18n v10`，支持 `en` 和 `zh-CN`。
- 语言偏好通过 `settings` 表持久化（`app.locale`）。
- 后端不返回翻译后文案，统一返回 `ErrorPayload { code, args }`，由前端映射。
- `pnpm i18n:check` 会校验两个 locale 文件的 key 集合是否一致。

### AI 与翻译

- AI 助手与翻译模块共用 `AiProvider` 概念（配置在 `ai_providers` 表）。
- AI 服务通过 OpenAI-compatible Chat Completions 协议调用各厂商 API。
- 翻译支持传统 API 与 AI 翻译两种引擎，结果按 `source_hash + target_lang + provider_id` 缓存在 `translations` 表。

## 重要工程约定

- `Cargo.toml` 中启用了严格的 clippy lint：`unsafe_code = "forbid"`、`unwrap_used = "deny"`、`expect_used = "warn"`，以及 `pedantic`/`nursery` warnings。提交前需保证 `cargo clippy` 无新增错误。
- `rustfmt.toml` 配置 Rust 代码格式，提交前运行 `cargo fmt`。
- 前端使用 Tailwind CSS v4（`@tailwindcss/vite`），颜色通过 CSS 变量定义在 `src/styles/theme.css`。
- Vite 开发服务器固定端口 `1420`，并忽略 `src-tauri/**` 的改动以避免循环热重载。
- 当前仓库没有单元测试或集成测试；`cargo test` 会正常通过但无实际用例覆盖。


## 注意事项

- 前端优化和设计修改必须调用 frontend-design skill。
- 修复问题不要靠猜，要全面分析找出真正的问题再修复。
