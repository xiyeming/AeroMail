# AeroMail 设计规范（第一阶段骨架）

## 1. 项目概述

AeroMail 是一款基于 **Rust + Tauri v2 + Vue 3** 的跨平台现代化邮件桌面客户端，采用"重后端、轻前端"架构。本文档是项目启动阶段的正式设计规范，基于《PRD.md》《UI设计系统.md》《技术实现方案.md》进行提炼和固化，明确第一阶段的骨架实现范围与后续迭代路线。

## 2. 设计目标

- 搭建可运行、可扩展的项目骨架
- 建立前后端通信、状态管理、主题系统、数据库等基础设施
- 实现账户管理的端到端闭环（UI → IPC → Rust → SQLite）
- 为三栏邮件主界面、搜索、写信等后续功能预留清晰的扩展接口

## 3. 架构设计

### 3.1 整体架构

```text
┌─────────────────────────────────────────────────────────────┐
│ 前端 UI 层 (Webview)                                          │
│ Vue 3 + TypeScript + Vite + Tailwind CSS + Shadcn UI        │
├─────────────────────────────────────────────────────────────┤
│ Tauri v2 IPC (Commands / Events)                              │
├─────────────────────────────────────────────────────────────┤
│ 后端核心层 (Rust 主进程)                                       │
│ • Commands  (IPC 入口)                                        │
│ • Services  (account_manager, sync_engine, ...)             │
│ • Models    (account, mail, settings)                       │
│ • DB        (rusqlite + schema migrations)                  │
├─────────────────────────────────────────────────────────────┤
│ 异步运行时 (Tokio)                                            │
│ 后续接入 IMAP Worker / SMTP Worker / Search Indexer           │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 进程/线程模型

| 组件 | 进程/线程 | 职责 |
|------|-----------|------|
| Webview | 每窗口独立进程 | Vue 渲染、用户交互 |
| Rust 主进程 | 单进程多线程 | IPC 路由、状态管理 |
| Tokio Runtime | 多线程线程池 | 异步任务调度（后续 IMAP/SMTP） |
| SQLite | 主线程/连接池 | 本地数据持久化 |

## 4. 技术栈与版本

### 4.1 核心版本约束

| 层级 | 技术 | 版本 |
|------|------|------|
| 后端语言 | Rust | 2024 edition |
| 桌面框架 | Tauri | 2.11 |
| 前端框架 | Vue | 3.5+ |
| 构建工具 | Vite | 6+ |
| 样式 | Tailwind CSS | 4.x |
| UI 组件 | Shadcn UI (Vue) | latest |
| 图标 | Lucide Vue | latest |
| 类型语言 | TypeScript | 5.7+ strict |

### 4.2 后端依赖

| 用途 | Crate |
|------|-------|
| 异步运行时 | tokio |
| 数据库 | rusqlite |
| 序列化 | serde, serde_json |
| 错误处理 | thiserror, anyhow |
| 配置/状态 | tauri::State |
| 日志 | tracing, tracing-subscriber |
| 唯一 ID | uuid |
| 时间 | chrono |
| 加密（后续） | aes-gcm, keyring |
| 邮件协议（后续） | async-imap, lettre, mailparse |
| 全文检索（后续） | tantivy, tantivy-jieba |

### 4.3 前端依赖

| 用途 | 包 |
|------|-----|
| 状态管理 | pinia |
| 路由 | vue-router |
| 工具库 | @vueuse/core |
| IPC | @tauri-apps/api |
| 图标 | lucide-vue-next |
| 组件基础 | radix-vue |
| 类名工具 | clsx, tailwind-merge |

## 5. 第一阶段骨架范围

### 5.1 后端骨架

1. **项目结构**：按 `commands / services / models / db` 分层组织
2. **数据库**：使用 rusqlite 初始化本地 SQLite，执行 schema 建表
3. **账户管理**：
   - `add_account`
   - `list_accounts`
   - `delete_account`
   - `test_account_connection`（占位实现，先返回成功/失败的模拟结果）
4. **错误模型**：定义 `AeroError`（使用 `thiserror`），统一转换为字符串返回前端
5. **状态管理**：Tauri `State` 持有 `AppState`（含 `AccountManager` 和 `Database`）

### 5.2 前端骨架

1. **项目结构**：`src/components / layouts / views / stores / composables / types / styles`
2. **主题系统**：CSS 变量定义 Dark/Light，Tailwind 扩展颜色映射
3. **三栏布局**：`AppLayout.vue` 响应式布局（Desktop / Tablet / Compact / Mobile）
4. **Sidebar**：品牌区、New Mail 按钮、邮件文件夹列表、账户列表、底部功能区
5. **MailList**：邮件列表项 Compact Card、hover 操作占位
6. **MailViewer**：阅读区占位，后续接入 iframe 渲染
7. **全局组件**：
   - `CommandPalette.vue`：搜索入口、快捷键导航
   - `StatusBar.vue`：同步状态、未读统计、在线状态
   - `ToastContainer.vue`：Toast 队列与动画
8. **状态**：Pinia stores（status, toast, account, mail 占位）
9. **IPC 封装**：统一 `invoke` 调用后端命令

### 5.3 明确不包含在第一阶段的内容

- 真实的 IMAP/SMTP 连接与同步
- 邮件解析（mailparse）
- 全文检索（Tantivy）
- HTML 邮件渲染沙箱
- 富文本写信编辑器
- 多窗口独立进程
- OAuth2 完整授权流程
- 密码/Token 加密存储（先用明文占位，接口预留）
- 系统托盘/通知/全局快捷键的完整实现

## 6. 目录结构

```
AeroMail/
├── src/                          # 前端源码
│   ├── components/
│   │   ├── AppSidebar.vue
│   │   ├── MailList.vue
│   │   ├── MailViewer.vue
│   │   ├── CommandPalette.vue
│   │   ├── StatusBar.vue
│   │   ├── ToastContainer.vue
│   │   └── WindowTitleBar.vue
│   ├── layouts/
│   │   └── AppLayout.vue
│   ├── views/
│   │   ├── InboxView.vue
│   │   ├── ReaderView.vue
│   │   └── ComposeView.vue
│   ├── stores/
│   │   ├── account.ts
│   │   ├── mail.ts
│   │   ├── status.ts
│   │   └── toast.ts
│   ├── composables/
│   │   ├── useTheme.ts
│   │   ├── useResponsive.ts
│   │   ├── useWindowManager.ts
│   │   └── useTauriInvoke.ts
│   ├── types/
│   │   ├── account.ts
│   │   ├── mail.ts
│   │   └── window.ts
│   ├── styles/
│   │   ├── theme.css
│   │   └── fonts.css
│   ├── App.vue
│   ├── main.ts
│   └── router.ts
├── src-tauri/                    # Tauri / Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── account.rs
│   │   │   ├── mail.rs
│   │   │   ├── search.rs
│   │   │   ├── compose.rs
│   │   │   └── window.rs
│   │   ├── services/
│   │   │   ├── account_manager.rs
│   │   │   ├── sync_engine.rs
│   │   │   ├── mail_parser.rs
│   │   │   ├── smtp_queue.rs
│   │   │   └── search_indexer.rs
│   │   ├── models/
│   │   │   ├── account.rs
│   │   │   ├── mail.rs
│   │   │   └── settings.rs
│   │   ├── db/
│   │   │   ├── schema.rs
│   │   │   ├── pool.rs
│   │   │   └── migrations.rs
│   │   └── error.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
│       └── default.json
├── docs/
│   ├── PRD.md
│   ├── UI设计系统.md
│   ├── 技术实现方案.md
│   └── superpowers/specs/
│       └── 2026-06-17-aeromail-design.md
├── public/
│   └── fonts/
├── package.json
├── tailwind.config.ts
├── tsconfig.json
├── vite.config.ts
├── rustfmt.toml
└── clippy.toml
```

## 7. 数据模型

### 7.1 后端 Rust 模型（骨架阶段）

```rust
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsMode {
    Required,
    StartTls,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    OAuth2 { access_token: String, refresh_token: String, expires_at: i64 },
    Password { password_encrypted: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub ca_cert_path: Option<String>,
    pub verify_certificate: bool,
    pub connect_timeout_secs: u64,
    pub read_timeout_secs: u64,
    pub keepalive: bool,
}
```

### 7.2 数据库 Schema

骨架阶段创建以下表：

- `accounts`：账户配置
- `folders`：邮件文件夹
- `mails`：邮件元数据与正文
- `attachments`：附件元数据
- `drafts`：草稿
- `settings`：应用设置

具体字段与《技术实现方案.md》第 4.2 节保持一致。

## 8. 关键设计决策

### 8.1 IPC 与错误处理

- 所有 Tauri Command 返回 `Result<T, String>`，后端错误通过 `thiserror` 转换为前端可读的字符串
- 后续迭代中可引入结构化的错误码与国际化消息

### 8.2 状态管理

- 后端使用 `tauri::State<'_, AppState>` 共享 `AccountManager` 和 `Database`
- 前端使用 Pinia 管理全局状态，组件内部使用 Composition API

### 8.3 主题系统

- CSS 变量定义在 `:root` 和 `[data-theme="light"]`
- Tailwind 扩展 `colors` 映射到 CSS 变量
- 默认 Dark 主题，支持跟随系统 `prefers-color-scheme`

### 8.4 响应式布局

| 断点 | 宽度 | 行为 |
|------|------|------|
| Desktop | ≥1400px | 三栏完整显示 |
| Tablet | 1140px–1399px | Sidebar 可折叠 |
| Compact | 800px–1139px | 两栏，Viewer 全屏覆盖 |
| Mobile | <800px | 单栏 |

## 9. 安全与性能考虑

### 9.1 安全

- 骨架阶段不处理真实密码/token 加密，但接口已预留 `AuthConfig`
- HTML 邮件渲染将使用 `sandbox="allow-same-origin"` 禁用脚本（后续实现）
- CSP 策略在 `tauri.conf.json` 中配置

### 9.2 性能

- SQLite 使用连接池管理（r2d2 或 deadpool）
- 前端邮件列表后续接入虚拟滚动
- Tantivy 索引在同步时批量写入（后续实现）

## 10. 测试策略

- Rust 单元测试：AccountManager、Database 操作
- Rust Clippy + rustfmt 静态检查
- 前端 ESLint + TypeScript 类型检查
- 手动验证：三栏布局、主题切换、账户 CRUD

## 11. 迭代路线图

### 第一阶段（当前）：骨架
项目初始化 + 三栏布局 + 主题 + 账户管理 CRUD + SQLite + 全局 UI 占位

### 第二阶段：邮件同步
- IMAP 连接与认证
- UID 增量同步
- 后台同步状态上报
- 邮件列表真实数据

### 第三阶段：阅读与搜索
- HTML 邮件 iframe 渲染沙箱
- 追踪像素拦截
- 纯文本/HTML 切换
- Tantivy 全文检索

### 第四阶段：写信与多窗口
- 富文本/Markdown 编辑器
- 附件拖拽
- SMTP 发信队列
- 独立读信/写信窗口

### 第五阶段：平台适配与优化
- 系统托盘/通知
- Wayland 原生优化
- 性能调优与打包

## 12. 开放决策

1. 是否使用 Cargo workspace 拆分前端无关的 crate？**当前决定**：单 crate，简化项目结构。
2. 是否引入迁移工具（如 `refinery`）管理 SQLite schema？**当前决定**：手动执行 migrations，保持简单。
3. 前端是否使用 `unplugin-auto-import`？**当前决定**：不引入，显式导入以保持清晰。

---

*设计规范基于 PRD.md、UI设计系统.md、技术实现方案.md 整理，作为第一阶段实现的依据。*
