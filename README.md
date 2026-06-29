<p align="center">
  <img src="assets/logo.svg" alt="AeroMail Logo" width="120">
</p>

<h1 align="center">AeroMail</h1>

<p align="center">
  基于 Rust + Tauri v2 + Vue 3 的现代化跨平台桌面邮件客户端
</p>

<p align="center">
  <a href="README.en.md">English</a> · 中文
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri" alt="Tauri v2">
  <img src="https://img.shields.io/badge/Vue-3-4FC08D?logo=vue.js" alt="Vue 3">
  <img src="https://img.shields.io/badge/Rust-2024-000000?logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT">
</p>

---

## 目录

- [项目简介](#项目简介)
- [功能特性](#功能特性)
- [技术栈](#技术栈)
- [环境要求](#环境要求)
- [快速开始](#快速开始)
  - [安装依赖](#安装依赖)
  - [启动开发](#启动开发)
  - [构建生产包](#构建生产包)
- [项目结构](#项目结构)
- [架构说明](#架构说明)
- [开发命令](#开发命令)
- [贡献指南](#贡献指南)
- [许可证](#许可证)
- [致谢](#致谢)

## 项目简介

**AeroMail** 是一款面向桌面端用户的现代化邮件客户端，采用“重后端、轻前端”的设计理念：
所有 IO 密集型（数据库、网络）和 CPU 密集型（邮件解析、搜索分词）任务均由 Rust 后端处理，
前端仅负责响应式渲染，确保流畅的用户体验与较低的系统资源占用。

AeroMail 支持 IMAP 收信/同步、SMTP 发信、本地 SQLite 缓存、Tantivy 全文索引、AI 助手、
翻译、系统托盘、最小化到托盘、多语言等特性，适用于 Linux（原生 Wayland）、macOS 和 Windows。

## 功能特性

- 📧 **邮件收发**：支持 IMAP 同步收件箱/自定义文件夹，SMTP 发送邮件。
- 🔍 **全文搜索**：基于 Tantivy 的全文索引，支持中文分词（tantivy-jieba）。
- 🤖 **AI 助手**：内置 AI 聊天会话、邮件摘要、待办提取、写作辅助，支持自定义 AI 厂商。
- 🌐 **翻译**：支持传统翻译 API 与 AI 翻译，结果可缓存；邮件正文支持划词翻译。
- 🔒 **安全渲染**：邮件 HTML 通过沙箱 iframe 渲染，配合 CSP  nonce 与事件转发，
  物理隔离脚本执行与 XSS 攻击。
- 🖼️ **远程内容控制**：自动识别邮件中的远程图片/资源域名，支持“允许一次”“始终信任”。
- 🌙 **深色主题**：默认深色模式，邮件内容锁定 `color-scheme: light` 防止排版错乱。
- 🌐 **多语言**：内置简体中文与英文，语言偏好持久化存储。
- 🔔 **系统托盘**：最小化到托盘，支持显示/退出菜单。
- ⌨️ **键盘导航**：支持 J/K 切换邮件、Enter 打开、Esc 关闭等快捷键。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri v2 |
| 前端框架 | Vue 3 Composition API |
| 构建工具 | Vite |
| 样式 | Tailwind CSS v4 |
| 状态管理 | Pinia |
| 国际化 | vue-i18n v10 |
| 后端语言 | Rust 2024 edition |
| 异步运行时 | Tokio |
| 数据库 | SQLite（rusqlite bundled） |
| 搜索索引 | Tantivy + tantivy-jieba |
| 邮件协议 | IMAP（async-imap）、SMTP（lettre） |
| 邮件解析 | mail-parser |
| TLS | native-tls / tokio-native-tls |

## 环境要求

- **Rust**：1.85+（项目使用 `edition = "2024"`）
- **Node.js**：20+
- **包管理器**：pnpm
- **Linux 开发依赖**：WebKitGTK 开发包，例如：
  - `webkit2gtk-4.1-devel`
  - `libgtk-3-devel`

> 不同发行版的包名可能不同，请根据本地环境安装对应的 WebKitGTK 4.1 开发包。

## 快速开始

### 安装依赖

```bash
pnpm install
```

### 启动开发

```bash
./scripts/run-dev.sh
```

该脚本会自动检测 Wayland 会话并设置 `WEBKIT_DISABLE_COMPOSITING_MODE=1`，
然后拉起 Tauri 开发模式（自动启动 Vite 前端服务器，端口 `1420`）。

如果你希望直接使用 `cargo tauri dev`，在 Wayland 下可手动设置：

```bash
export GDK_BACKEND=wayland
export WEBKIT_DISABLE_COMPOSITING_MODE=1
cargo tauri dev
```

### 构建生产包

```bash
# 前端生产构建
pnpm build

# 完整应用打包（按平台输出 deb/appimage/dmg/msi/nsis）
cargo tauri build
```

## 项目结构

```text
AeroMail/
├── src/                    # 前端源码（Vue 3 + TypeScript）
│   ├── components/         # 可复用组件
│   ├── composables/        # 组合式函数
│   ├── layouts/            # 布局组件
│   ├── router.ts           # Vue Router
│   ├── stores/             # Pinia 状态管理
│   ├── i18n/               # 国际化文案
│   ├── styles/             # 主题与全局样式
│   └── main.ts             # 前端入口
├── src-tauri/              # Rust + Tauri 后端
│   ├── src/
│   │   ├── commands/       # Tauri IPC 命令入口
│   │   ├── services/       # 业务服务实现
│   │   ├── models/         # 共享数据模型
│   │   ├── db/             # SQLite schema、连接池、迁移
│   │   └── lib.rs          # 应用状态与 Tauri 运行时
│   └── Cargo.toml
├── docs/                   # 文档
├── scripts/                # 脚本工具（如 i18n 检查）
├── assets/                 # 静态资源（logo 等）
├── package.json
├── LICENSE
└── README.md
```

## 架构说明

### 进程与运行时

- **Webview 进程**：每个窗口独立进程，运行 Vue 3 前端。
- **Rust 主进程**：单进程多线程，负责 IPC 路由、状态管理、插件调度。
- **Tokio Runtime**：多线程线程池，调度 IMAP 同步、SMTP 发信、AI/翻译 API 调用等异步任务。

### 后端状态

后端状态通过 `src-tauri/src/lib.rs` 中的 `AppState` 集中管理，
所有服务以 `Arc<RwLock<T>>` 形式挂载到 Tauri 应用状态：

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

Tauri Command 通过 `tauri::State<'_, AppState>` 访问这些服务。

### 同步引擎

`SyncService` 为每个账户维护独立的 Tokio Task：

1. UIDVALIDITY 校验
2. 获取本地最大 UID
3. IMAP FETCH 新增邮件
4. `mail-parser` 解析 MIME
5. 写入 SQLite
6. 写入 Tantivy 索引

失败时指数退避重试，网络恢复后自动续传。

### HTML 邮件渲染安全

邮件 HTML 通过 `<iframe sandbox="allow-same-origin allow-scripts">` 渲染，
配合 CSP nonce 仅允许可信内联脚本，脚本仅负责将 `<a>` 链接点击通过 `postMessage` 转发给父页面。
外部图片默认被拦截，防止泄露隐私与 XSS。

## 开发命令

```bash
# 前端类型检查
pnpm type-check

# ESLint 检查
pnpm lint

# 前端代码格式化
pnpm format

# i18n key 一致性检查
pnpm i18n:check

# Rust 格式化
cargo fmt

# Rust 静态检查
cargo clippy

# Rust 测试
cargo test
```

## 贡献指南

欢迎提交 Issue 与 Pull Request！

1. Fork 本仓库并创建功能分支：`git checkout -b feat/your-feature`
2. 确保代码通过 `pnpm type-check`、`pnpm lint`、`pnpm i18n:check`、`cargo clippy`
3. 提交清晰的 commit message，建议遵循 [Conventional Commits](https://www.conventionalcommits.org/)
4. 提交 Pull Request 并描述改动动机与影响范围

## 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。

Copyright (c) 2026 [xiyeming](mailto:xiyeming@163.com)

## 致谢

- [Tauri](https://tauri.app/)
- [Vue.js](https://vuejs.org/)
- [Vite](https://vitejs.dev/)
- [Tailwind CSS](https://tailwindcss.com/)
- [Tokio](https://tokio.rs/)
- [Tantivy](https://tantivy-py.github.io/)
- [mail-parser](https://github.com/stalwartlabs/mail-parser)
- [lettre](https://lettre.rs/)
- [async-imap](https://github.com/async-email/async-imap)
