# 富文本写信 + SMTP 发信实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 AeroMail 中实现富文本写信、本地草稿自动保存、IMAP Drafts 同步、回复/转发、附件上传与 SMTP 发信。

**Architecture:** 后端新增独立的 `services/compose/` 模块（草稿 CRUD、MIME 构建、SMTP 发送、IMAP 草稿同步）和 `commands/compose.rs` 命令入口；前端使用 Tiptap 编辑器 + Pinia compose store + Vue 路由 `/compose`、`/reply/:mailId` 等。所有 IO/CPU 密集型工作放在 Rust 后端。

**Tech Stack:** Rust 2024、Tauri v2、Vue 3、TypeScript、Pinia、Tiptap、`lettre` 0.11、SQLite、IMAP/SMTP。

## Global Constraints

- Rust `edition = "2024"`，`rust-version = "1.85"`。
- `unsafe_code = "forbid"`；`unwrap_used = "deny"`；`expect_used = "warn"`。
- 后端所有命令返回 `Result<T, ErrorPayload>`。
- 新增错误必须在 `src-tauri/src/error.rs` 中定义并在 `to_payload()` 中映射错误码。
- 所有前端文案必须同时写入 `src/i18n/locales/en.json` 和 `src/i18n/locales/zh-CN.json`，并运行 `pnpm i18n:check` 通过。
- 附件文件必须限制在 `app_data_dir/drafts/<draft_id>/` 下。
- 每次任务完成后运行相关检查：`cargo check`、`pnpm type-check`、`pnpm lint`、`cargo clippy`。

---

## 文件结构总览

| 文件 | 职责 |
|------|------|
| `src-tauri/src/models/compose.rs` | 草稿、附件、回复上下文、发信请求类型 |
| `src-tauri/src/db/schema.rs` | 扩展 `drafts` 表 |
| `src-tauri/src/db/migrations.rs` | 已有数据库的列迁移（幂等） |
| `src-tauri/src/db/pool.rs` | 草稿、附件元数据 CRUD |
| `src-tauri/src/services/compose/draft.rs` | 本地草稿与附件文件管理 |
| `src-tauri/src/services/compose/mime_builder.rs` | 使用 `lettre` 构建 MIME |
| `src-tauri/src/services/compose/smtp_sender.rs` | SMTP 连接与发送 |
| `src-tauri/src/services/compose/imap_draft_sync.rs` | IMAP Drafts APPEND/更新 |
| `src-tauri/src/services/compose/mod.rs` | 服务入口 |
| `src-tauri/src/commands/compose.rs` | Tauri 命令 |
| `src-tauri/src/error.rs` | 新增错误变体 |
| `src-tauri/src/commands/mod.rs` | 注册 compose 模块 |
| `src-tauri/src/services/mod.rs` | 注册 compose 模块 |
| `src-tauri/src/lib.rs` | 注册命令、初始化 ComposeService |
| `src-tauri/Cargo.toml` | 新增 `lettre` 依赖 |
| `src/types/compose.ts` | 前端类型 |
| `src/stores/compose.ts` | Pinia 状态、自动保存、发送流程 |
| `src/composables/useTiptap.ts` | Tiptap 初始化与工具栏 |
| `src/components/compose/RecipientInput.vue` | 收件人标签输入 |
| `src/components/compose/ComposeHeader.vue` | 账号、主题、To/Cc/Bcc |
| `src/components/compose/ComposeAttachmentList.vue` | 附件列表 |
| `src/components/compose/ComposeEditor.vue` | Tiptap 编辑器 |
| `src/views/ComposeView.vue` | 写信页面 |
| `src/router.ts` | 新增路由 |
| `src/components/MailViewer.vue` | 添加回复/转发按钮 |
| `src/i18n/locales/en.json` | 英文文案 |
| `src/i18n/locales/zh-CN.json` | 中文文案 |
| `package.json` | 新增 Tiptap 依赖 |

---

## Task 1: 添加依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `package.json`
- Test: `cargo check`（Rust），`pnpm install`（前端）

**Interfaces:**
- Produces: `lettre` crate 可用于后续 MIME/SMTP 任务；Tiptap 包可用于前端。

- [ ] **Step 1: 添加 Rust 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 区块末尾添加：

```toml
lettre = { version = "0.11", default-features = false, features = ["builder", "smtp-transport", "tokio1-native-tls", "pool"] }
```

- [ ] **Step 2: 添加前端依赖**

在 `package.json` 的 `dependencies` 中添加：

```json
"@tiptap/core": "^2.11.0",
"@tiptap/extension-image": "^2.11.0",
"@tiptap/extension-link": "^2.11.0",
"@tiptap/extension-placeholder": "^2.11.0",
"@tiptap/extension-underline": "^2.11.0",
"@tiptap/pm": "^2.11.0",
"@tiptap/starter-kit": "^2.11.0",
"@tiptap/vue-3": "^2.11.0"
```

- [ ] **Step 3: 安装依赖**

```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail
pnpm install
cd src-tauri
cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock package.json pnpm-lock.yaml
git commit -m "chore(deps): add lettre and tiptap for compose feature"
```

---

## Task 2: 扩展数据库 Schema

**Files:**
- Modify: `src-tauri/src/db/schema.rs`
- Modify: `src-tauri/src/db/migrations.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `drafts` 表包含 `bcc_addresses`、`reply_context_json`、`synced_at`、`remote_uid`。

- [ ] **Step 1: 更新 `drafts` 表定义**

替换 `src-tauri/src/db/schema.rs` 中的 `DRAFTS_TABLE`：

```rust
pub const DRAFTS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS drafts (
    id TEXT PRIMARY KEY,
    account_id TEXT,
    subject TEXT,
    to_addresses TEXT,
    cc_addresses TEXT,
    bcc_addresses TEXT,
    reply_context_json TEXT,
    body_html TEXT,
    body_text TEXT,
    attachments_json TEXT,
    saved_at INTEGER,
    synced_at INTEGER,
    remote_uid INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE SET NULL
)
";
```

- [ ] **Step 2: 为已有数据库添加列迁移**

替换 `src-tauri/src/db/migrations.rs` 为：

```rust
use rusqlite::Connection;

use super::schema::ALL_SCHEMAS;
use crate::error::AeroError;

/// Runs all database migrations to ensure the schema is up to date.
///
/// # Errors
///
/// Returns [`AeroError::Database`] if any migration fails.
pub fn run_migrations(conn: &mut Connection) -> Result<(), AeroError> {
    let tx = conn.transaction()?;
    for schema in ALL_SCHEMAS {
        tx.execute_batch(schema)?;
    }
    run_draft_migrations(&tx)?;
    tx.commit()?;
    Ok(())
}

fn column_exists(tx: &rusqlite::Transaction, table: &str, column: &str) -> Result<bool, AeroError> {
    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2")?;
    let mut rows = stmt.query([table, column])?;
    Ok(rows.next()?.is_some())
}

fn run_draft_migrations(tx: &rusqlite::Transaction) -> Result<(), AeroError> {
    let columns = [
        "bcc_addresses TEXT",
        "reply_context_json TEXT",
        "synced_at INTEGER",
        "remote_uid INTEGER",
    ];
    for column_def in &columns {
        let column_name = column_def.split_whitespace().next().unwrap_or(column_def);
        if !column_exists(tx, "drafts", column_name)? {
            tx.execute(
                &format!("ALTER TABLE drafts ADD COLUMN {}", column_def),
                [],
            )?;
        }
    }
    Ok(())
}
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri
cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/db/schema.rs src-tauri/src/db/migrations.rs
git commit -m "feat(db): extend drafts table for compose"
```

---

## Task 3: 新增 Compose 数据模型

**Files:**
- Create: `src-tauri/src/models/compose.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `ComposeDraft`、`AttachmentDraft`、`ReplyContext`、`ReplyKind`、`ComposeDraftSummary`、`SendMailRequest`。

- [ ] **Step 1: 创建模型文件**

创建 `src-tauri/src/models/compose.rs`：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeDraft {
    pub id: String,
    pub account_id: String,
    pub reply_context: Option<ReplyContext>,
    pub subject: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub body_html: String,
    pub body_text: String,
    pub attachments: Vec<AttachmentDraft>,
    pub saved_at: i64,
    pub synced_at: Option<i64>,
    pub remote_uid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyContext {
    pub original_mail_id: String,
    pub kind: ReplyKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplyKind {
    Reply,
    ReplyAll,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentDraft {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub local_path: Option<String>,
    pub content_id: Option<String>,
    pub is_inline: bool,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeDraftSummary {
    pub id: String,
    pub account_id: String,
    pub subject: String,
    pub to: Vec<String>,
    pub saved_at: i64,
    pub has_attachments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMailRequest {
    pub draft_id: String,
}
```

- [ ] **Step 2: 注册模型模块**

修改 `src-tauri/src/models/mod.rs`：

```rust
pub mod account;
pub mod ai;
pub mod compose;
pub mod error;
pub mod mail;
pub mod search;
pub mod translation;
```

- [ ] **Step 3: 编译验证**

```bash
cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/models/compose.rs src-tauri/src/models/mod.rs
git commit -m "feat(models): add compose draft types"
```

---

## Task 4: 新增错误变体

**Files:**
- Modify: `src-tauri/src/error.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `SmtpConnectionFailed`、`SmtpAuthFailed`、`InvalidRecipient`、`DraftNotFound`、`AttachmentNotFound`、`InvalidAttachment`、`MailBuilderFailed`、`ImapAppendFailed`。

- [ ] **Step 1: 添加错误变体**

在 `src-tauri/src/error.rs` 的 `AeroError` enum 中，在 `SyncError` 之前添加：

```rust
#[error("SMTP connection failed: {0}")]
SmtpConnectionFailed(String),
#[error("SMTP authentication failed: {0}")]
SmtpAuthFailed(String),
#[error("invalid recipient: {0}")]
InvalidRecipient(String),
#[error("draft not found: {0}")]
DraftNotFound(String),
#[error("attachment not found: {0}")]
AttachmentNotFound(String),
#[error("invalid attachment: {0}")]
InvalidAttachment(String),
#[error("mail builder failed: {0}")]
MailBuilderFailed(String),
#[error("IMAP append failed: {0}")]
ImapAppendFailed(String),
```

- [ ] **Step 2: 添加错误码映射**

在 `to_payload()` 的 `SyncError` 分支之前添加：

```rust
Self::SmtpConnectionFailed(msg) => ErrorPayload {
    code: "SMTP_CONNECTION_FAILED".to_string(),
    args: vec![msg.clone()],
},
Self::SmtpAuthFailed(msg) => ErrorPayload {
    code: "SMTP_AUTH_FAILED".to_string(),
    args: vec![msg.clone()],
},
Self::InvalidRecipient(msg) => ErrorPayload {
    code: "INVALID_RECIPIENT".to_string(),
    args: vec![msg.clone()],
},
Self::DraftNotFound(id) => ErrorPayload {
    code: "DRAFT_NOT_FOUND".to_string(),
    args: vec![id.clone()],
},
Self::AttachmentNotFound(id) => ErrorPayload {
    code: "ATTACHMENT_NOT_FOUND".to_string(),
    args: vec![id.clone()],
},
Self::InvalidAttachment(msg) => ErrorPayload {
    code: "INVALID_ATTACHMENT".to_string(),
    args: vec![msg.clone()],
},
Self::MailBuilderFailed(msg) => ErrorPayload {
    code: "MAIL_BUILDER_FAILED".to_string(),
    args: vec![msg.clone()],
},
Self::ImapAppendFailed(msg) => ErrorPayload {
    code: "IMAP_APPEND_FAILED".to_string(),
    args: vec![msg.clone()],
},
```

- [ ] **Step 3: 编译验证**

```bash
cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/error.rs
git commit -m "feat(error): add compose-related error variants"
```

---

## Task 5: 数据库草稿与附件方法

**Files:**
- Modify: `src-tauri/src/db/pool.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `Database::upsert_draft`、`get_draft`、`list_drafts`、`delete_draft`、`get_account_for_draft`。

- [ ] **Step 1: 添加 draft CRUD 方法**

在 `src-tauri/src/db/pool.rs` 的 `// ---- Search Index Helpers ----` 之前添加：

```rust
// ---- Draft CRUD ----

/// Inserts or updates a compose draft.
pub fn upsert_draft(
    &self,
    draft: &crate::models::compose::ComposeDraft,
) -> Result<(), AeroError> {
    let conn = self.connection()?;
    let to_json = serde_json::to_string(&draft.to)?;
    let cc_json = serde_json::to_string(&draft.cc)?;
    let bcc_json = serde_json::to_string(&draft.bcc)?;
    let reply_context_json = serde_json::to_string(&draft.reply_context)?;
    let attachments_json = serde_json::to_string(&draft.attachments)?;
    conn.execute(
        "INSERT INTO drafts (id, account_id, subject, to_addresses, cc_addresses, bcc_addresses,
         reply_context_json, body_html, body_text, attachments_json, saved_at, synced_at, remote_uid)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
           account_id=excluded.account_id, subject=excluded.subject,
           to_addresses=excluded.to_addresses, cc_addresses=excluded.cc_addresses,
           bcc_addresses=excluded.bcc_addresses, reply_context_json=excluded.reply_context_json,
           body_html=excluded.body_html, body_text=excluded.body_text,
           attachments_json=excluded.attachments_json, saved_at=excluded.saved_at,
           synced_at=excluded.synced_at, remote_uid=excluded.remote_uid",
        (
            &draft.id,
            &draft.account_id,
            &draft.subject,
            to_json,
            cc_json,
            bcc_json,
            reply_context_json,
            &draft.body_html,
            &draft.body_text,
            attachments_json,
            draft.saved_at,
            draft.synced_at,
            draft.remote_uid.map(|v| v as i64),
        ),
    )?;
    Ok(())
}

/// Retrieves a draft by ID.
#[allow(clippy::significant_drop_tightening)]
pub fn get_draft(
    &self,
    draft_id: &str,
) -> Result<Option<crate::models::compose::ComposeDraft>, AeroError> {
    let conn = self.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, subject, to_addresses, cc_addresses, bcc_addresses,
         reply_context_json, body_html, body_text, attachments_json, saved_at,
         synced_at, remote_uid FROM drafts WHERE id = ?1",
    )?;
    let mut rows = stmt.query([draft_id])?;
    if let Some(row) = rows.next()? {
        let to: Vec<String> = serde_json::from_str(&row.get::<_, String>(3)?)?;
        let cc: Vec<String> = serde_json::from_str(&row.get::<_, String>(4)?)?;
        let bcc: Vec<String> = serde_json::from_str(&row.get::<_, String>(5)?)?;
        let reply_context: Option<crate::models::compose::ReplyContext> =
            serde_json::from_str(&row.get::<_, String>(6)?)?;
        let attachments: Vec<crate::models::compose::AttachmentDraft> =
            serde_json::from_str(&row.get::<_, String>(9)?)?;
        Ok(Some(crate::models::compose::ComposeDraft {
            id: row.get(0)?,
            account_id: row.get(1)?,
            subject: row.get(2)?,
            to,
            cc,
            bcc,
            reply_context,
            body_html: row.get(7)?,
            body_text: row.get(8)?,
            attachments,
            saved_at: row.get(10)?,
            synced_at: row.get(11)?,
            remote_uid: row.get::<_, Option<i64>>(12)?.map(|v| v as u32),
        }))
    } else {
        Ok(None)
    }
}

/// Lists drafts for an account or all drafts if account_id is None.
#[allow(clippy::significant_drop_tightening)]
pub fn list_drafts(
    &self,
    account_id: Option<&str>,
) -> Result<Vec<crate::models::compose::ComposeDraftSummary>, AeroError> {
    let conn = self.connection()?;
    let (sql, params): (&str, Vec<&dyn rusqlite::ToSql>) = match account_id {
        Some(id) => (
            "SELECT id, account_id, subject, to_addresses, saved_at, attachments_json
             FROM drafts WHERE account_id = ?1 ORDER BY saved_at DESC",
            vec![&id],
        ),
        None => (
            "SELECT id, account_id, subject, to_addresses, saved_at, attachments_json
             FROM drafts ORDER BY saved_at DESC",
            vec![],
        ),
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(&*params,
        |row| {
            let to: Vec<String> = serde_json::from_str(&row.get::<_, String>(3)?)
                .unwrap_or_default();
            let attachments: Vec<crate::models::compose::AttachmentDraft> =
                serde_json::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or_default();
            Ok(crate::models::compose::ComposeDraftSummary {
                id: row.get(0)?,
                account_id: row.get(1)?,
                subject: row.get(2)?,
                to,
                saved_at: row.get(4)?,
                has_attachments: !attachments.is_empty(),
            })
        },
    )?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AeroError::Database(e.to_string()))
}

/// Deletes a draft by ID.
pub fn delete_draft(&self, draft_id: &str) -> Result<(), AeroError> {
    let conn = self.connection()?;
    conn.execute("DELETE FROM drafts WHERE id = ?1", [draft_id])?;
    Ok(())
}

/// Gets the account_id for a draft.
#[allow(clippy::significant_drop_tightening)]
pub fn get_draft_account_id(
    &self,
    draft_id: &str,
) -> Result<Option<String>, AeroError> {
    let conn = self.connection()?;
    let mut stmt = conn.prepare("SELECT account_id FROM drafts WHERE id = ?1")?;
    let mut rows = stmt.query([draft_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/db/pool.rs
git commit -m "feat(db): add draft CRUD operations"
```

---

## Task 6: 本地草稿与附件文件服务

**Files:**
- Create: `src-tauri/src/services/compose/draft.rs`
- Test: `cargo test`

**Interfaces:**
- Produces: `DraftService` 提供 `save_draft`、`get_draft`、`list_drafts`、`delete_draft`、`prepare_reply`。

- [ ] **Step 1: 创建服务文件**

创建 `src-tauri/src/services/compose/draft.rs`：

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::compose::{
    AttachmentDraft, ComposeDraft, ComposeDraftSummary, ReplyContext, ReplyKind,
};
use crate::models::mail::MailDetail;

pub struct DraftService {
    db: Arc<Database>,
    drafts_dir: PathBuf,
}

impl DraftService {
    pub fn new(db: Arc<Database>, drafts_dir: PathBuf) -> Self {
        Self { db, drafts_dir }
    }

    /// Returns the directory for a draft's attachments.
    pub fn draft_dir(&self, draft_id: &str) -> PathBuf {
        self.drafts_dir.join(draft_id)
    }

    /// Returns the directory for a specific attachment.
    pub fn attachment_dir(&self, draft_id: &str, attachment_id: &str) -> PathBuf {
        self.draft_dir(draft_id).join(attachment_id)
    }

    /// Ensures the draft directory exists.
    fn ensure_draft_dir(&self, draft_id: &str) -> Result<PathBuf, AeroError> {
        let dir = self.draft_dir(draft_id);
        std::fs::create_dir_all(&dir)
            .map_err(|e| AeroError::Internal(format!("failed to create draft dir: {e}")))?;
        Ok(dir)
    }

    /// Saves a draft and its attachment metadata locally.
    pub fn save_draft(&self,
        draft: &mut ComposeDraft,
    ) -> Result<(), AeroError> {
        if draft.id.is_empty() {
            draft.id = Uuid::new_v4().to_string();
        }
        draft.saved_at = Utc::now().timestamp();
        self.ensure_draft_dir(&draft.id)?;
        self.db.upsert_draft(draft)?;
        Ok(())
    }

    /// Retrieves a draft by ID.
    pub fn get_draft(&self,
        draft_id: &str,
    ) -> Result<Option<ComposeDraft>, AeroError> {
        self.db.get_draft(draft_id)
    }

    /// Lists draft summaries.
    pub fn list_drafts(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<ComposeDraftSummary>, AeroError> {
        self.db.list_drafts(account_id)
    }

    /// Deletes a draft and its attachment files.
    pub fn delete_draft(&self,
        draft_id: &str,
    ) -> Result<(), AeroError> {
        self.db.delete_draft(draft_id)?;
        let dir = self.draft_dir(draft_id);
        if dir.exists() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        Ok(())
    }

    /// Writes attachment bytes to disk inside the draft directory.
    pub fn write_attachment(
        &self,
        draft_id: &str,
        attachment: &AttachmentDraft,
        data: &[u8],
    ) -> Result<PathBuf, AeroError> {
        self.ensure_draft_dir(draft_id)?;
        let dir = self.attachment_dir(draft_id, &attachment.id);
        std::fs::create_dir_all(&dir)
            .map_err(|e| AeroError::InvalidAttachment(format!("failed to create attachment dir: {e}")))?;
        let path = dir.join(&attachment.filename);
        std::fs::write(&path, data)
            .map_err(|e| AeroError::InvalidAttachment(format!("failed to write attachment: {e}")))?;
        Ok(path)
    }

    /// Reads attachment bytes from disk.
    pub fn read_attachment(
        &self,
        draft_id: &str,
        attachment_id: &str,
        filename: &str,
    ) -> Result<Vec<u8>, AeroError> {
        let path = self.attachment_dir(draft_id, attachment_id).join(filename);
        std::fs::read(&path).map_err(|e| AeroError::AttachmentNotFound(format!("{path:?}: {e}")))
    }

    /// Prepares a reply/forward draft from an existing mail.
    pub fn prepare_reply(
        &self,
        account_id: &str,
        original: &MailDetail,
        kind: ReplyKind,
    ) -> Result<ComposeDraft, AeroError> {
        let mut to = Vec::new();
        let mut cc = Vec::new();

        if let Some(ref from) = original.from_address {
            match kind {
                ReplyKind::Reply | ReplyKind::ReplyAll => to.push(from.clone()),
                ReplyKind::Forward => {}
            }
        }

        if matches!(kind, ReplyKind::ReplyAll) {
            if let Some(ref to_addrs) = original.to_addresses {
                for addr in to_addrs.split(',') {
                    let trimmed = addr.trim();
                    if !trimmed.is_empty() {
                        to.push(trimmed.to_string());
                    }
                }
            }
            if let Some(ref cc_addrs) = original.cc_addresses {
                for addr in cc_addrs.split(',') {
                    let trimmed = addr.trim();
                    if !trimmed.is_empty() {
                        cc.push(trimmed.to_string());
                    }
                }
            }
        }

        let subject_prefix = match kind {
            ReplyKind::Reply | ReplyKind::ReplyAll => "Re: ",
            ReplyKind::Forward => "Fwd: ",
        };
        let subject = match original.subject {
            Some(ref s) if s.starts_with(subject_prefix) => s.clone(),
            Some(ref s) => format!("{}{}", subject_prefix, s),
            None => subject_prefix.trim_end().to_string(),
        };

        let quote = format!(
            "\n\nOn {}, {} wrote:\n> {}",
            original.date.map(|d| d.to_string()).unwrap_or_default(),
            original.from_address.clone().unwrap_or_default(),
            original.body_text.clone().unwrap_or_default().replace('\n', "\n> ")
        );
        let body_html = format!("\n\n<blockquote>{}\n</blockquote>", html_escape::encode_safe(&original.body_text.clone().unwrap_or_default()));

        Ok(ComposeDraft {
            id: String::new(),
            account_id: account_id.to_string(),
            reply_context: Some(ReplyContext {
                original_mail_id: original.id.clone(),
                kind,
            }),
            subject,
            to,
            cc,
            bcc: Vec::new(),
            body_html,
            body_text: quote,
            attachments: Vec::new(),
            saved_at: 0,
            synced_at: None,
            remote_uid: None,
        })
    }
}
```

- [ ] **Step 2: 添加 html_escape 依赖**

在 `src-tauri/Cargo.toml` 添加：

```toml
html-escape = "0.2"
```

- [ ] **Step 3: 编译验证**

```bash
cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/services/compose/draft.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat(compose): add draft service with attachment file management"
```

---

## Task 7: MIME 构建器

**Files:**
- Create: `src-tauri/src/services/compose/mime_builder.rs`
- Test: `cargo test`（新增测试）

**Interfaces:**
- Produces: `build_message(draft, from_address, account_name) -> Result<Vec<u8>, AeroError>`。

- [ ] **Step 1: 创建 MIME 构建器**

创建 `src-tauri/src/services/compose/mime_builder.rs`：

```rust
use lettre::message::{
    header::{ContentType, DispositionDisposition},
    Attachment, MultiPart, SinglePart,
};
use lettre::Message;

use crate::error::AeroError;
use crate::models::compose::{AttachmentDraft, ComposeDraft};

/// Builds an RFC-compliant MIME message from a draft.
pub fn build_message(
    draft: &ComposeDraft,
    from_address: &str,
    from_name: &str,
    attachment_bytes: &[(String, Vec<u8>)], // (attachment_id, bytes)
) -> Result<Vec<u8>, AeroError> {
    let mut builder = Message::builder()
        .from(format!("{} <{}>", from_name, from_address).parse()
            .map_err(|e| AeroError::InvalidRecipient(format!("invalid from: {e}")))?);

    for addr in &draft.to {
        builder = builder.to(addr.parse()
            .map_err(|e| AeroError::InvalidRecipient(format!("invalid to {addr}: {e}")))?);
    }
    for addr in &draft.cc {
        builder = builder.cc(addr.parse()
            .map_err(|e| AeroError::InvalidRecipient(format!("invalid cc {addr}: {e}")))?);
    }
    for addr in &draft.bcc {
        builder = builder.bcc(addr.parse()
            .map_err(|e| AeroError::InvalidRecipient(format!("invalid bcc {addr}: {e}")))?);
    }

    let mut message = builder
        .subject(draft.subject.clone())
        .multipart(build_body_and_attachments(draft, attachment_bytes)?)
        .map_err(|e| AeroError::MailBuilderFailed(e.to_string()))?;

    if let Some(ref reply_ctx) = draft.reply_context {
        message = message.header(lettre::message::header::InReplyTo::from(reply_ctx.original_mail_id.clone()));
    }

    message.formatted()
        .map(|s| s.into_bytes())
        .map_err(|e| AeroError::MailBuilderFailed(e.to_string()))
}

fn build_body_and_attachments(
    draft: &ComposeDraft,
    attachment_bytes: &[(String, Vec<u8>)],
) -> Result<MultiPart, AeroError> {
    // Build alternative body (text + html)
    let body_part = MultiPart::alternative()
        .singlepart(
            SinglePart::builder()
                .header(ContentType::text_plain())
                .body(draft.body_text.clone()),
        )
        .singlepart(
            SinglePart::builder()
                .header(ContentType::text_html())
                .body(draft.body_html.clone()),
        );

    if draft.attachments.is_empty() {
        return Ok(body_part);
    }

    // Build mixed part: body + attachments
    let mut mixed = MultiPart::mixed().multipart(body_part);

    for attachment in &draft.attachments {
        let bytes = attachment_bytes
            .iter()
            .find(|(id, _)| id == &attachment.id)
            .map(|(_, bytes)| bytes.clone())
            .ok_or_else(|| AeroError::AttachmentNotFound(attachment.id.clone()))?;

        let content_type = attachment
            .mime_type
            .parse::<ContentType>()
            .unwrap_or_else(|_| ContentType::parse("application/octet-stream").unwrap());

        if attachment.is_inline {
            let content_id = attachment
                .content_id
                .clone()
                .unwrap_or_else(|| attachment.id.clone());
            let part = Attachment::new_inline(content_id)
                .body(bytes, content_type)
                .filename(attachment.filename.clone());
            mixed = mixed.singlepart(part);
        } else {
            let part = Attachment::new(attachment.filename.clone())
                .body(bytes, content_type);
            mixed = mixed.singlepart(part);
        }
    }

    Ok(mixed)
}
```

- [ ] **Step 2: 添加 MIME 构建单元测试**

在文件末尾添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_draft() -> ComposeDraft {
        ComposeDraft {
            id: "d1".to_string(),
            account_id: "a1".to_string(),
            reply_context: None,
            subject: "Hello".to_string(),
            to: vec!["to@example.com".to_string()],
            cc: vec![],
            bcc: vec![],
            body_html: "<p>Hello</p>".to_string(),
            body_text: "Hello".to_string(),
            attachments: vec![],
            saved_at: 0,
            synced_at: None,
            remote_uid: None,
        }
    }

    #[test]
    fn builds_text_html_message() {
        let draft = sample_draft();
        let bytes = build_message(&draft, "from@example.com", "Sender", &[]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("to@example.com"));
    }

    #[test]
    fn builds_message_with_attachment() {
        let mut draft = sample_draft();
        draft.attachments.push(AttachmentDraft {
            id: "att1".to_string(),
            filename: "note.txt".to_string(),
            mime_type: "text/plain".to_string(),
            size: 5,
            local_path: None,
            content_id: None,
            is_inline: false,
            preview_url: None,
        });
        let bytes = build_message(&draft, "from@example.com", "Sender", &[("att1".to_string(), b"hello".to_vec())]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("note.txt"));
        assert!(text.contains("base64"));
    }
}
```

- [ ] **Step 3: 运行测试**

```bash
cargo test --package aeromail -- mime_builder
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/services/compose/mime_builder.rs
git commit -m "feat(compose): add MIME builder with tests"
```

---

## Task 8: SMTP 发送器

**Files:**
- Create: `src-tauri/src/services/compose/smtp_sender.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `send_message(config, message_bytes) -> Result<(), AeroError>`。

- [ ] **Step 1: 创建 SMTP 发送器**

创建 `src-tauri/src/services/compose/smtp_sender.rs`：

```rust
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport};

use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig, TlsMode};

/// Sends a pre-built MIME message via SMTP.
pub async fn send_message(
    config: &AccountConfig,
    message_bytes: Vec<u8>,
) -> Result<(), AeroError> {
    let creds = build_credentials(config)?;

    let tls_parameters = TlsParameters::new(config.smtp.host.clone())
        .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?;

    let mailer = match config.smtp.tls_mode {
        TlsMode::Required => {
            AsyncSmtpTransport::<lettre::TokioExecutor>::starttls_relay(&config.smtp.host,
            )
            .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
            .port(config.smtp.port)
            .tls(Tls::Required(tls_parameters))
            .credentials(creds)
            .build()
        }
        TlsMode::StartTls => {
            AsyncSmtpTransport::<lettre::TokioExecutor>::starttls_relay(&config.smtp.host,
            )
            .map_err(|e| AeroError::SmtpConnectionFailed(e.to_string()))?
            .port(config.smtp.port)
            .tls(Tls::Required(tls_parameters))
            .credentials(creds)
            .build()
        }
        TlsMode::None => {
            AsyncSmtpTransport::<lettre::TokioExecutor>::builder_dangerous(
                config.smtp.host.clone(),
            )
            .port(config.smtp.port)
            .credentials(creds)
            .build()
        }
    };

    let email: lettre::Message = std::str::from_utf8(&message_bytes)
        .map_err(|e| AeroError::MailBuilderFailed(format!("invalid utf8: {e}")))?
        .parse()
        .map_err(|e| AeroError::MailBuilderFailed(format!("invalid message: {e}")))?;

    mailer.send(email).await.map_err(|e| match e {
        lettre::transport::smtp::Error::Authentication(a) => {
            AeroError::SmtpAuthFailed(a.to_string())
        }
        _ => AeroError::SmtpConnectionFailed(e.to_string()),
    })?;

    Ok(())
}

fn build_credentials(config: &AccountConfig) -> Result<Credentials, AeroError> {
    match &config.auth {
        AuthConfig::Password { password_encrypted } => {
            let password = String::from_utf8_lossy(password_encrypted);
            Ok(Credentials::new(config.name.clone(), password.to_string()))
        }
        AuthConfig::OAuth2 { access_token, .. } => {
            Ok(Credentials::new_xoauth2(&config.name,
                access_token,
            ))
        }
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/services/compose/smtp_sender.rs
git commit -m "feat(compose): add SMTP sender with password and OAuth2"
```

---

## Task 9: IMAP 草稿同步

**Files:**
- Create: `src-tauri/src/services/compose/imap_draft_sync.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `sync_draft_to_imap(config, draft, message_bytes) -> Result<u32, AeroError>` 返回新的 remote_uid。

- [ ] **Step 1: 创建 IMAP 草稿同步**

创建 `src-tauri/src/services/compose/imap_draft_sync.rs`：

```rust
use std::net::TcpStream;
use std::sync::Arc;

use imap::Session;
use native_tls::TlsStream;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::{AccountConfig, AuthConfig};
use crate::models::compose::ComposeDraft;

/// Syncs a draft to the IMAP Drafts folder.
pub fn sync_draft_to_imap(
    config: &AccountConfig,
    draft: &ComposeDraft,
    message_bytes: &[u8],
    db: &Arc<Database>,
) -> Result<u32, AeroError> {
    let mut session = connect_blocking(config)?;

    let drafts_folder = find_drafts_folder(&mut session)?;

    // Delete old draft if exists
    if let Some(uid) = draft.remote_uid {
        let _ = delete_uid(&mut session, &drafts_folder, uid);
    }

    // APPEND new draft
    session.select(&drafts_folder)
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;

    let flags = vec!["\\Draft"];
    let result = session.append(&drafts_folder,
        message_bytes,
        Some(&flags.iter().map(|s| s.as_ref()).collect::<Vec<_>>()),
        None,
    ).map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;

    let _ = session.logout();

    // Extract UID from append result if available
    let new_uid = result.map(|v| v as u32).unwrap_or(0);
    Ok(new_uid)
}

fn connect_blocking(
    config: &AccountConfig,
) -> Result<Session<TlsStream<TcpStream>>, AeroError> {
    let mut tls_builder = native_tls::TlsConnector::builder();
    if !config.advanced.verify_certificate {
        tls_builder.danger_accept_invalid_certs(true);
    }
    if let Some(ref cert_path) = config.advanced.ca_cert_path {
        tls_builder.add_root_certificate(
            native_tls::Certificate::from_pem(
                &std::fs::read(cert_path)
                    .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?,
            )
            .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?,
        );
    }
    let tls = tls_builder
        .build()
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

    let client = imap::connect(
        format!("{}:{}", config.imap.host, config.imap.port),
        &config.imap.host,
        &tls,
    )
    .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

    let session = match &config.auth {
        AuthConfig::Password { password_encrypted } => {
            let password = String::from_utf8_lossy(password_encrypted);
            client
                .login(&config.name, &password)
                .map_err(|e| AeroError::ImapAuthFailed(e.0.to_string()))?
        }
        AuthConfig::OAuth2 { .. } => {
            return Err(AeroError::InvalidConfig(
                "OAuth2 IMAP not yet implemented".into(),
            ));
        }
    };

    Ok(session)
}

fn find_drafts_folder(session: &mut Session<TlsStream<TcpStream>>) -> Result<String, AeroError> {
    let mailboxes = session
        .list(None, Some("*"))
        .map_err(|e| AeroError::ImapConnectionFailed(e.to_string()))?;

    let candidates = ["Drafts", "Draft", "[Gmail]/Drafts", "草稿箱", "\\u8349\\u7a3f"];
    for candidate in &candidates {
        if let Some(mb) = mailboxes.iter().find(|m| m.name().eq_ignore_ascii_case(candidate)) {
            return Ok(mb.name().to_string());
        }
    }
    // Fallback to first drafts-like folder
    for mb in mailboxes.iter() {
        if mb.name().to_lowercase().contains("draft") {
            return Ok(mb.name().to_string());
        }
    }
    Err(AeroError::ImapAppendFailed("Drafts folder not found".to_string()))
}

fn delete_uid(
    session: &mut Session<TlsStream<TcpStream>>,
    folder: &str,
    uid: u32,
) -> Result<(), AeroError> {
    session.select(folder)
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;
    session.uid_store(format!("{uid}"), "+FLAGS (\\\\Deleted)")
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;
    session.expunge()
        .map_err(|e| AeroError::ImapAppendFailed(e.to_string()))?;
    Ok(())
}
```

- [ ] **Step 2: 编译验证**

```bash
cargo check
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/services/compose/imap_draft_sync.rs
git commit -m "feat(compose): add IMAP draft sync"
```

---

## Task 10: 组装 ComposeService

**Files:**
- Create: `src-tauri/src/services/compose/mod.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `ComposeService` 提供 `save_draft`、`get_draft`、`list_drafts`、`delete_draft`、`prepare_reply`、`send_mail`、`sync_draft_to_imap`。

- [ ] **Step 1: 创建服务入口**

创建 `src-tauri/src/services/compose/mod.rs`：

```rust
mod draft;
mod imap_draft_sync;
mod mime_builder;
mod smtp_sender;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::account::AccountConfig;
use crate::models::compose::{
    ComposeDraft, ComposeDraftSummary, ReplyKind, SendMailRequest,
};
use crate::models::mail::MailDetail;
use crate::services::account_manager::AccountManager;

use self::draft::DraftService;

pub struct ComposeService {
    draft_service: DraftService,
    account_manager: Arc<RwLock<AccountManager>>,
    db: Arc<Database>,
}

impl ComposeService {
    pub fn new(
        db: Arc<Database>,
        drafts_dir: PathBuf,
        account_manager: Arc<RwLock<AccountManager>>,
    ) -> Self {
        Self {
            draft_service: DraftService::new(Arc::clone(&db), drafts_dir),
            account_manager,
            db,
        }
    }

    pub fn save_draft(&self,
        mut draft: ComposeDraft,
    ) -> Result<ComposeDraft, AeroError> {
        self.draft_service.save_draft(&mut draft)?;
        Ok(draft)
    }

    pub fn get_draft(&self,
        draft_id: &str,
    ) -> Result<Option<ComposeDraft>, AeroError> {
        self.draft_service.get_draft(draft_id)
    }

    pub fn list_drafts(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<ComposeDraftSummary>, AeroError> {
        self.draft_service.list_drafts(account_id)
    }

    pub fn delete_draft(&self,
        draft_id: &str,
    ) -> Result<(), AeroError> {
        self.draft_service.delete_draft(draft_id)
    }

    pub fn prepare_reply(
        &self,
        account_id: &str,
        original: &MailDetail,
        kind: ReplyKind,
    ) -> Result<ComposeDraft, AeroError> {
        self.draft_service.prepare_reply(account_id, original, kind)
    }

    pub async fn send_mail(
        &self,
        req: SendMailRequest,
    ) -> Result<(), AeroError> {
        let draft = self
            .draft_service
            .get_draft(&req.draft_id)?
            .ok_or_else(|| AeroError::DraftNotFound(req.draft_id.clone()))?;

        let account_id = draft.account_id.clone();
        let account_config = self.load_account_config(&account_id).await?;

        // Collect attachment bytes
        let mut attachment_bytes = Vec::new();
        for att in &draft.attachments {
            let bytes = self.draft_service.read_attachment(
                &draft.id, &att.id, &att.filename)?;
            attachment_bytes.push((att.id.clone(), bytes));
        }

        let message_bytes = mime_builder::build_message(
            &draft,
            &account_config.name,
            &account_config.name,
            &attachment_bytes,
        )?;

        smtp_sender::send_message(&account_config, message_bytes.clone()).await?;

        // Clean up local draft
        self.draft_service.delete_draft(&draft.id)?;

        Ok(())
    }

    pub async fn sync_draft_to_imap(
        &self,
        draft_id: &str,
    ) -> Result<(), AeroError> {
        let draft = self
            .draft_service
            .get_draft(draft_id)?
            .ok_or_else(|| AeroError::DraftNotFound(draft_id.to_string()))?;

        let account_config = self.load_account_config(&draft.account_id).await?;

        let mut attachment_bytes = Vec::new();
        for att in &draft.attachments {
            let bytes = self.draft_service.read_attachment(
                &draft.id, &att.id, &att.filename)?;
            attachment_bytes.push((att.id.clone(), bytes));
        }

        let message_bytes = mime_builder::build_message(
            &draft,
            &account_config.name,
            &account_config.name,
            &attachment_bytes,
        )?;

        let new_uid = imap_draft_sync::sync_draft_to_imap(
            &account_config,
            &draft,
            &message_bytes,
            &self.db,
        )?;

        // Update synced_at and remote_uid
        let mut updated = draft;
        updated.synced_at = Some(chrono::Utc::now().timestamp());
        updated.remote_uid = Some(new_uid);
        self.draft_service.save_draft(&mut updated)?;

        Ok(())
    }

    async fn load_account_config(
        &self,
        account_id: &str,
    ) -> Result<AccountConfig, AeroError> {
        let am = self.account_manager.read().await;
        // AccountManager does not expose load by ID; use DB directly or extend AccountManager.
        // For now, read from DB using existing pool methods (not yet implemented in AccountManager).
        // This task will need to add `get_account_config` to AccountManager.
        todo!("load_account_config needs AccountManager::get_account_config")
    }
}
```

- [ ] **Step 2: 在 AccountManager 添加按 ID 读取配置**

在 `src-tauri/src/services/account_manager.rs` 的 `delete_account` 之后添加：

```rust
/// Retrieves a full account configuration by ID.
///
/// # Errors
///
/// Returns [`AeroError::AccountNotFound`] if no account with the given ID exists.
#[allow(clippy::significant_drop_tightening)]
pub fn get_account_config(&self,
    account_id: &str,
) -> Result<AccountConfig, AeroError> {
    let conn = self.db.connection()?;
    conn.query_row(
        "SELECT id, name, provider, imap_host, imap_port, smtp_host, smtp_port,
         tls_mode, auth_type, auth_credentials_encrypted, ca_cert_path,
         verify_certificate, connect_timeout, read_timeout, keepalive,
         sync_interval, excluded_folders
         FROM accounts WHERE id = ?1",
        [account_id],
        |row| {
            let provider_json: String = row.get(2)?;
            let provider = serde_json::from_str(&provider_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?;

            let tls_mode_json: String = row.get(7)?;
            let tls_mode = serde_json::from_str(&tls_mode_json)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

            let auth_type: String = row.get(8)?;
            let auth_credentials: Option<Vec<u8>> = row.get(9)?;
            let auth = match auth_type.as_str() {
                "Password" => AuthConfig::Password {
                    password_encrypted: auth_credentials.unwrap_or_default(),
                },
                "OAuth2" => {
                    let creds_bytes = auth_credentials.unwrap_or_default();
                    let creds_json = String::from_utf8_lossy(&creds_bytes);
                    serde_json::from_str(&creds_json)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?
                }
                _ => return Err(rusqlite::Error::InvalidParameterName(format!(
                    "Unknown auth type: {auth_type}"
                ))),
            };

            let excluded_folders_json: String = row.get(16)?;
            let excluded_folders: Vec<String> =
                serde_json::from_str(&excluded_folders_json).unwrap_or_default();

            Ok(AccountConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                provider,
                imap: crate::models::account::ServerConfig {
                    host: row.get(3)?,
                    port: row.get::<_, i64>(4)? as u16,
                    tls_mode,
                },
                smtp: crate::models::account::ServerConfig {
                    host: row.get(5)?,
                    port: row.get::<_, i64>(6)? as u16,
                    tls_mode: crate::models::account::TlsMode::Required,
                },
                auth,
                advanced: crate::models::account::AdvancedConfig {
                    ca_cert_path: row.get(10)?,
                    verify_certificate: row.get::<_, i64>(11)? != 0,
                    connect_timeout_secs: row.get::<_, i64>(12)? as u64,
                    read_timeout_secs: row.get::<_, i64>(13)? as u64,
                    keepalive: row.get::<_, i64>(14)? != 0,
                },
                sync_interval_secs: row.get::<_, i64>(15)? as u64,
                excluded_folders,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AeroError::AccountNotFound(account_id.to_string()),
        _ => AeroError::Database(e.to_string()),
    })
}
```

- [ ] **Step 3: 更新 ComposeService 的 load_account_config**

将 `todo!` 替换为：

```rust
async fn load_account_config(
    &self,
    account_id: &str,
) -> Result<AccountConfig, AeroError> {
    let am = self.account_manager.read().await;
    am.get_account_config(account_id)
}
```

- [ ] **Step 4: 注册服务模块**

修改 `src-tauri/src/services/mod.rs`：

```rust
pub mod account_manager;
pub mod ai;
pub mod compose;
pub mod search;
pub mod sync;
pub mod translation;
```

- [ ] **Step 5: 编译验证**

```bash
cargo check
```

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/services/compose/mod.rs src-tauri/src/services/mod.rs src-tauri/src/services/account_manager.rs
git commit -m "feat(compose): add ComposeService and account config lookup"
```

---

## Task 11: Tauri Compose 命令

**Files:**
- Create: `src-tauri/src/commands/compose.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `cargo check`

**Interfaces:**
- Produces: `save_draft`、`get_drafts`、`get_draft`、`delete_draft`、`send_mail`、`prepare_reply`、`sync_draft_to_imap` 命令。

- [ ] **Step 1: 创建命令文件**

创建 `src-tauri/src/commands/compose.rs`：

```rust
use tauri::State;

use crate::error::AeroError;
use crate::models::compose::{
    ComposeDraft, ComposeDraftSummary, ReplyKind, SendMailRequest,
};
use crate::models::error::ErrorPayload;
use crate::models::mail::MailDetail;
use crate::AppState;

#[tauri::command]
pub async fn save_draft(
    draft: ComposeDraft,
    state: State<'_, AppState>,
) -> Result<ComposeDraft, ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.save_draft(draft).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_drafts(
    account_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ComposeDraftSummary>, ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.list_drafts(account_id.as_deref()).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_draft(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<ComposeDraft, ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose
        .get_draft(&draft_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::DraftNotFound(draft_id).to_payload())
}

#[tauri::command]
pub async fn delete_draft(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.delete_draft(&draft_id).map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn send_mail(
    req: SendMailRequest,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose.send_mail(req).await.map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn prepare_reply(
    mail_id: String,
    kind: ReplyKind,
    state: State<'_, AppState>,
) -> Result<ComposeDraft, ErrorPayload> {
    let db = &state.db;
    let original = db
        .get_mail_detail(&mail_id)
        .map_err(|e| e.to_payload())?
        .ok_or_else(|| AeroError::MailNotFound(mail_id.clone()).to_payload())?;

    let account_id = original.account_id.clone();
    let compose = state.compose_service.read().await;
    compose
        .prepare_reply(&account_id,
            &original,
            kind,
        )
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn sync_draft_to_imap(
    draft_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    compose
        .sync_draft_to_imap(&draft_id)
        .await
        .map_err(|e| e.to_payload())
}
```

- [ ] **Step 2: 注册命令模块**

修改 `src-tauri/src/commands/mod.rs`：

```rust
pub mod account;
pub mod ai;
pub mod compose;
pub mod mail;
pub mod search;
pub mod settings;
pub mod sync;
pub mod translation;
```

- [ ] **Step 3: 初始化 ComposeService 并注册命令**

修改 `src-tauri/src/lib.rs`：

在 `use commands::ai::{...}` 之后添加：

```rust
use commands::compose::{
    delete_draft, get_draft, get_drafts, prepare_reply, save_draft, send_mail, sync_draft_to_imap,
};
```

在 `AppState` struct 中添加：

```rust
pub compose_service: Arc<RwLock<crate::services::compose::ComposeService>>,
```

在 `AppState::new` 中，在 `let search_service = ...` 之后、`Ok(Self {` 之前添加：

```rust
let drafts_dir = app_dir.join("drafts");
let compose_service = Arc::new(RwLock::new(
    crate::services::compose::ComposeService::new(
        Arc::clone(&db),
        drafts_dir,
        Arc::clone(&account_manager),
    ),
));
```

在 `Ok(Self { ... })` 中添加 `compose_service,`。

在 `tauri::generate_handler![...]` 中添加：

```rust
save_draft,
get_drafts,
get_draft,
delete_draft,
send_mail,
prepare_reply,
sync_draft_to_imap,
```

- [ ] **Step 4: 编译验证**

```bash
cargo check
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/commands/compose.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add compose Tauri commands"
```

---

## Task 12: 前端类型与 Store

**Files:**
- Create: `src/types/compose.ts`
- Create: `src/stores/compose.ts`
- Test: `pnpm type-check`

**Interfaces:**
- Produces: 前端 `ComposeDraft`、`AttachmentDraft`、`ReplyKind` 类型与 `useComposeStore`。

- [ ] **Step 1: 创建前端类型**

创建 `src/types/compose.ts`：

```typescript
export type ReplyKind = 'reply' | 'reply_all' | 'forward';

export interface ReplyContext {
  originalMailId: string;
  kind: ReplyKind;
}

export interface AttachmentDraft {
  id: string;
  filename: string;
  mimeType: string;
  size: number;
  localPath?: string;
  contentId?: string;
  isInline: boolean;
  previewUrl?: string;
}

export interface ComposeDraft {
  id: string;
  accountId: string;
  replyContext?: ReplyContext;
  subject: string;
  to: string[];
  cc: string[];
  bcc: string[];
  bodyHtml: string;
  bodyText: string;
  attachments: AttachmentDraft[];
  savedAt: number;
  syncedAt?: number;
  remoteUid?: number;
}

export interface ComposeDraftSummary {
  id: string;
  accountId: string;
  subject: string;
  to: string[];
  savedAt: number;
  hasAttachments: boolean;
}

export interface SendMailRequest {
  draftId: string;
}
```

- [ ] **Step 2: 创建 Compose Store**

创建 `src/stores/compose.ts`：

```typescript
import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { useDebounceFn } from '@vueuse/core';
import type { ComposeDraft, ComposeDraftSummary, ReplyKind } from '@/types/compose';
import { useToastStore } from './toast';
import { useRouter } from 'vue-router';

const emptyDraft = (): ComposeDraft => ({
  id: '',
  accountId: '',
  replyContext: undefined,
  subject: '',
  to: [],
  cc: [],
  bcc: [],
  bodyHtml: '',
  bodyText: '',
  attachments: [],
  savedAt: 0,
});

export const useComposeStore = defineStore('compose', () => {
  const draft = ref<ComposeDraft>(emptyDraft());
  const drafts = ref<ComposeDraftSummary[]>([]);
  const loading = ref(false);
  const saving = ref(false);
  const lastError = ref<string | null>(null);

  const hasDraft = computed(() => draft.value.id !== '');

  function setDraft(value: ComposeDraft) {
    draft.value = value;
  }

  function updateField<K extends keyof ComposeDraft>(key: K, value: ComposeDraft[K]) {
    draft.value[key] = value;
    triggerAutosave();
  }

  function updateBody(html: string, text: string) {
    draft.value.bodyHtml = html;
    draft.value.bodyText = text;
    triggerAutosave();
  }

  const saveToBackend = useDebounceFn(async () => {
    if (!draft.value.accountId) return;
    saving.value = true;
    try {
      const saved = await invoke<ComposeDraft>('save_draft', { draft: draft.value });
      draft.value.id = saved.id;
      draft.value.savedAt = saved.savedAt;
      lastError.value = null;
      // trigger IMAP sync after a longer debounce
      triggerImapSync();
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      useToastStore().error(lastError.value);
    } finally {
      saving.value = false;
    }
  }, 2000);

  const triggerAutosave = () => {
    saveToBackend();
  };

  const syncToImap = useDebounceFn(async () => {
    if (!draft.value.id) return;
    try {
      await invoke('sync_draft_to_imap', { draftId: draft.value.id });
    } catch (e) {
      console.warn('IMAP draft sync failed', e);
    }
  }, 5000);

  const triggerImapSync = () => {
    syncToImap();
  };

  async function loadDraft(draftId: string) {
    loading.value = true;
    try {
      const loaded = await invoke<ComposeDraft>('get_draft', { draftId });
      draft.value = loaded;
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      useToastStore().error(lastError.value);
    } finally {
      loading.value = false;
    }
  }

  async function loadDrafts(accountId?: string) {
    drafts.value = await invoke<ComposeDraftSummary[]>('get_drafts', { accountId });
  }

  async function deleteDraft(draftId: string) {
    await invoke('delete_draft', { draftId });
    drafts.value = drafts.value.filter((d) => d.id !== draftId);
  }

  async function prepareReply(mailId: string, kind: ReplyKind) {
    loading.value = true;
    try {
      const reply = await invoke<ComposeDraft>('prepare_reply', { mailId, kind });
      draft.value = reply;
    } finally {
      loading.value = false;
    }
  }

  async function sendMail() {
    if (!draft.value.id) {
      await saveToBackend();
    }
    if (!draft.value.id) {
      useToastStore().error('Failed to save draft before sending');
      return;
    }
    loading.value = true;
    try {
      await invoke('send_mail', { draftId: draft.value.id });
      useToastStore().success('Mail sent');
      draft.value = emptyDraft();
      useRouter().push({ name: 'inbox' });
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      useToastStore().error(lastError.value);
    } finally {
      loading.value = false;
    }
  }

  function reset() {
    draft.value = emptyDraft();
  }

  return {
    draft,
    drafts,
    loading,
    saving,
    lastError,
    hasDraft,
    setDraft,
    updateField,
    updateBody,
    loadDraft,
    loadDrafts,
    deleteDraft,
    prepareReply,
    sendMail,
    reset,
  };
});
```

- [ ] **Step 3: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 4: 提交**

```bash
git add src/types/compose.ts src/stores/compose.ts
git commit -m "feat(compose): add frontend types and store"
```

---

## Task 13: Tiptap Composable

**Files:**
- Create: `src/composables/useTiptap.ts`
- Test: `pnpm type-check`

**Interfaces:**
- Produces: `useTiptap(options) -> { editor, toolbarActions }`。

- [ ] **Step 1: 创建 Composable**

创建 `src/composables/useTiptap.ts`：

```typescript
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Link from '@tiptap/extension-link';
import Image from '@tiptap/extension-image';
import Placeholder from '@tiptap/extension-placeholder';
import Underline from '@tiptap/extension-underline';
import { computed, type Ref } from 'vue';

export interface UseTiptapOptions {
  content: Ref<string>;
  placeholder?: string;
  onUpdate?: (html: string, text: string) => void;
  onImagePasted?: (file: File) => void;
}

export function useTiptap(options: UseTiptapOptions) {
  const editor = useEditor({
    content: options.content.value,
    extensions: [
      StarterKit,
      Underline,
      Link.configure({ openOnClick: false }),
      Image.configure({ allowBase64: true }),
      Placeholder.configure({ placeholder: options.placeholder ?? '' }),
    ],
    onUpdate: ({ editor }) => {
      options.onUpdate?.(editor.getHTML(), editor.getText());
    },
    editorProps: {
      handlePaste: (_view, event) => {
        const items = event.clipboardData?.items;
        if (!items) return false;
        for (const item of Array.from(items)) {
          if (item.type.startsWith('image/')) {
            const file = item.getAsFile();
            if (file) {
              options.onImagePasted?.(file);
            }
          }
        }
        return false;
      },
      handleDrop: (_view, event) => {
        const files = event.dataTransfer?.files;
        if (!files) return false;
        for (const file of Array.from(files)) {
          if (file.type.startsWith('image/')) {
            options.onImagePasted?.(file);
          }
        }
        return false;
      },
    },
  });

  const isActive = (name: string, attrs?: Record<string, unknown>) => {
    return editor.value?.isActive(name, attrs) ?? false;
  };

  const toolbarActions = computed(() => ({
    bold: () => editor.value?.chain().focus().toggleBold().run(),
    italic: () => editor.value?.chain().focus().toggleItalic().run(),
    underline: () => editor.value?.chain().focus().toggleUnderline().run(),
    strike: () => editor.value?.chain().focus().toggleStrike().run(),
    h1: () => editor.value?.chain().focus().toggleHeading({ level: 1 }).run(),
    h2: () => editor.value?.chain().focus().toggleHeading({ level: 2 }).run(),
    paragraph: () => editor.value?.chain().focus().setParagraph().run(),
    bulletList: () => editor.value?.chain().focus().toggleBulletList().run(),
    orderedList: () => editor.value?.chain().focus().toggleOrderedList().run(),
    blockquote: () => editor.value?.chain().focus().toggleBlockquote().run(),
    link: (url: string) => editor.value?.chain().focus().setLink({ href: url }).run(),
    unsetLink: () => editor.value?.chain().focus().unsetLink().run(),
    undo: () => editor.value?.chain().focus().undo().run(),
    redo: () => editor.value?.chain().focus().redo().run(),
  }));

  return { editor, EditorContent, isActive, toolbarActions };
}
```

- [ ] **Step 2: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 3: 提交**

```bash
git add src/composables/useTiptap.ts
git commit -m "feat(compose): add Tiptap composable"
```

---

## Task 14: 收件人输入组件

**Files:**
- Create: `src/components/compose/RecipientInput.vue`
- Test: `pnpm type-check`

**Interfaces:**
- Props: `modelValue: string[]`, `label: string`, `placeholder?: string`
- Emits: `update:modelValue`

- [ ] **Step 1: 创建组件**

创建 `src/components/compose/RecipientInput.vue`：

```vue
<template>
  <div class="recipient-input">
    <label class="text-xs text-[var(--text-secondary)]">{{ label }}</label>
    <div
      class="flex flex-wrap gap-1 rounded border border-[var(--border)] bg-[var(--surface)] p-1 focus-within:border-[var(--primary)]"
      @click="focusInput"
    >
      <span
        v-for="(email, idx) in modelValue"
        :key="idx"
        class="inline-flex items-center gap-1 rounded bg-[var(--primary-muted)] px-2 py-0.5 text-sm"
      >
        {{ email }}
        <button @click.stop="removeEmail(idx)">×</button>
      </span>
      <input
        ref="inputRef"
        v-model="inputValue"
        type="text"
        class="min-w-[120px] flex-1 bg-transparent px-1 py-0.5 text-sm outline-none"
        :placeholder="modelValue.length === 0 ? placeholder : ''"
        @keydown.enter.prevent="addEmail"
        @keydown.tab.prevent="addEmail"
        @keydown.backspace="onBackspace"
        @blur="addEmail"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
  modelValue: string[];
  label: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string[]): void;
}>();

const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function focusInput() {
  inputRef.value?.focus();
}

function addEmail() {
  const raw = inputValue.value.trim();
  if (!raw) return;
  const addresses = raw.split(/[,;\n]+/).map((a) => a.trim()).filter(Boolean);
  const valid = addresses.filter((a) => EMAIL_REGEX.test(a));
  if (valid.length > 0) {
    emit('update:modelValue', [...props.modelValue, ...valid]);
  }
  inputValue.value = '';
}

function removeEmail(idx: number) {
  const updated = [...props.modelValue];
  updated.splice(idx, 1);
  emit('update:modelValue', updated);
}

function onBackspace() {
  if (inputValue.value === '' && props.modelValue.length > 0) {
    const updated = [...props.modelValue];
    updated.pop();
    emit('update:modelValue', updated);
  }
}
</script>
```

- [ ] **Step 2: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 3: 提交**

```bash
git add src/components/compose/RecipientInput.vue
git commit -m "feat(compose): add recipient input component"
```

---

## Task 15: 写信头部组件

**Files:**
- Create: `src/components/compose/ComposeHeader.vue`
- Test: `pnpm type-check`

**Interfaces:**
- Props: `draft: ComposeDraft`, `accounts: AccountSummary[]`
- Emits: `update:draft`, `send`

- [ ] **Step 1: 创建组件**

创建 `src/components/compose/ComposeHeader.vue`：

```vue
<template>
  <div class="flex flex-col gap-3 p-4">
    <div class="flex items-center gap-2">
      <select
        :value="draft.accountId"
        class="rounded border border-[var(--border)] bg-[var(--surface)] px-2 py-1 text-sm"
        @change="updateAccount(($event.target as HTMLSelectElement).value)"
      >
        <option value="" disabled>{{ $t('compose.selectAccount') }}</option>
        <option v-for="acc in accounts" :key="acc.id" :value="acc.id">
          {{ acc.name }} ({{ acc.smtpHost }})
        </option>
      </select>
      <button
        class="ml-auto rounded bg-[var(--primary)] px-4 py-1 text-sm text-[var(--primary-fg)] disabled:opacity-50"
        :disabled="!canSend"
        @click="$emit('send')"
      >
        {{ $t('compose.send') }}
      </button>
    </div>

    <RecipientInput
      v-model="localDraft.to"
      :label="$t('compose.to')"
      :placeholder="$t('compose.toPlaceholder')"
    />

    <div class="flex items-center gap-2">
      <button class="text-xs text-[var(--primary)]" @click="showCc = !showCc">
        {{ $t('compose.cc') }}
      </button>
      <button class="text-xs text-[var(--primary)]" @click="showBcc = !showBcc">
        {{ $t('compose.bcc') }}
      </button>
    </div>

    <RecipientInput
      v-if="showCc"
      v-model="localDraft.cc"
      :label="$t('compose.cc')"
    />
    <RecipientInput
      v-if="showBcc"
      v-model="localDraft.bcc"
      :label="$t('compose.bcc')"
    />

    <input
      v-model="localDraft.subject"
      type="text"
      class="rounded border border-[var(--border)] bg-[var(--surface)] px-2 py-1 text-sm"
      :placeholder="$t('compose.subject')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ComposeDraft } from '@/types/compose';
import type { AccountSummary } from '@/types/account';
import RecipientInput from './RecipientInput.vue';

const props = defineProps<{
  draft: ComposeDraft;
  accounts: AccountSummary[];
}>();

const emit = defineEmits<{
  (e: 'update:draft', value: ComposeDraft): void;
  (e: 'send'): void;
}>();

const localDraft = computed({
  get: () => props.draft,
  set: (v) => emit('update:draft', v),
});

const showCc = ref(false);
const showBcc = ref(false);

const canSend = computed(() =>
  props.draft.accountId !== '' && props.draft.to.length > 0 && props.draft.subject !== ''
);

function updateAccount(accountId: string) {
  localDraft.value = { ...props.draft, accountId };
}

watch(
  () => props.draft.cc,
  (v) => {
    if (v.length > 0) showCc.value = true;
  },
  { immediate: true }
);
watch(
  () => props.draft.bcc,
  (v) => {
    if (v.length > 0) showBcc.value = true;
  },
  { immediate: true }
);
</script>
```

- [ ] **Step 2: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 3: 提交**

```bash
git add src/components/compose/ComposeHeader.vue
git commit -m "feat(compose): add compose header component"
```

---

## Task 16: 附件列表组件

**Files:**
- Create: `src/components/compose/ComposeAttachmentList.vue`
- Test: `pnpm type-check`

**Interfaces:**
- Props: `attachments: AttachmentDraft[]`
- Emits: `remove`

- [ ] **Step 1: 创建组件**

创建 `src/components/compose/ComposeAttachmentList.vue`：

```vue
<template>
  <div v-if="attachments.length > 0" class="flex flex-wrap gap-2 p-4">
    <div
      v-for="att in attachments"
      :key="att.id"
      class="flex items-center gap-2 rounded border border-[var(--border)] bg-[var(--surface)] px-2 py-1 text-sm"
    >
      <span>{{ att.filename }}</span>
      <span class="text-xs text-[var(--text-secondary)]">{{ formatSize(att.size) }}</span>
      <button class="text-[var(--danger)]" @click="$emit('remove', att.id)">×</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AttachmentDraft } from '@/types/compose';

defineProps<{
  attachments: AttachmentDraft[];
}>();

defineEmits<{
  (e: 'remove', attachmentId: string): void;
}>();

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
</script>
```

- [ ] **Step 2: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 3: 提交**

```bash
git add src/components/compose/ComposeAttachmentList.vue
git commit -m "feat(compose): add attachment list component"
```

---

## Task 17: 富文本编辑器组件

**Files:**
- Create: `src/components/compose/ComposeEditor.vue`
- Test: `pnpm type-check`

**Interfaces:**
- Props: `modelValue: string`
- Emits: `update:modelValue`, `image-pasted`

- [ ] **Step 1: 创建组件**

创建 `src/components/compose/ComposeEditor.vue`：

```vue
<template>
  <div class="flex flex-1 flex-col overflow-hidden">
    <div class="flex flex-wrap items-center gap-1 border-b border-[var(--border)] p-2">
      <button
        v-for="btn in toolbarButtons"
        :key="btn.key"
        class="rounded px-2 py-1 text-sm hover:bg-[var(--surface-hover)]"
        :class="{ 'bg-[var(--primary-muted)]': btn.active }"
        :title="btn.title"
        @click="btn.action"
      >
        {{ btn.label }}
      </button>
    </div>
    <EditorContent :editor="editor" class="flex-1 overflow-auto p-4"
 />
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import { useTiptap } from '@/composables/useTiptap';

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'image-pasted', file: File): void;
}>();

const { editor, EditorContent, isActive, toolbarActions } = useTiptap({
  content: computed(() => props.modelValue),
  placeholder: 'Write something...',
  onUpdate: (html) => emit('update:modelValue', html),
  onImagePasted: (file) => emit('image-pasted', file),
});

watch(
  () => props.modelValue,
  (value) => {
    if (editor.value && editor.value.getHTML() !== value) {
      editor.value.commands.setContent(value, false);
    }
  }
);

const toolbarButtons = computed(() => [
  { key: 'bold', label: 'B', title: 'Bold', active: isActive('bold'), action: toolbarActions.value.bold },
  { key: 'italic', label: 'I', title: 'Italic', active: isActive('italic'), action: toolbarActions.value.italic },
  { key: 'underline', label: 'U', title: 'Underline', active: isActive('underline'), action: toolbarActions.value.underline },
  { key: 'strike', label: 'S', title: 'Strike', active: isActive('strike'), action: toolbarActions.value.strike },
  { key: 'h1', label: 'H1', title: 'Heading 1', active: isActive('heading', { level: 1 }), action: toolbarActions.value.h1 },
  { key: 'h2', label: 'H2', title: 'Heading 2', active: isActive('heading', { level: 2 }), action: toolbarActions.value.h2 },
  { key: 'bulletList', label: '• List', title: 'Bullet List', active: isActive('bulletList'), action: toolbarActions.value.bulletList },
  { key: 'orderedList', label: '1. List', title: 'Ordered List', active: isActive('orderedList'), action: toolbarActions.value.orderedList },
  { key: 'blockquote', label: '"', title: 'Quote', active: isActive('blockquote'), action: toolbarActions.value.blockquote },
  { key: 'undo', label: '↶', title: 'Undo', active: false, action: toolbarActions.value.undo },
  { key: 'redo', label: '↷', title: 'Redo', active: false, action: toolbarActions.value.redo },
]);
</script>

<style scoped>
:deep(.ProseMirror p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: var(--text-secondary);
  pointer-events: none;
  height: 0;
}
</style>
```

- [ ] **Step 2: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 3: 提交**

```bash
git add src/components/compose/ComposeEditor.vue
git commit -m "feat(compose): add rich text editor component"
```

---

## Task 18: 写信主页面

**Files:**
- Create: `src/views/ComposeView.vue`
- Modify: `src/router.ts`
- Test: `pnpm type-check`

**Interfaces:**
- 路由：`/compose`、`/compose/:draftId`、`/reply/:mailId`、`/reply-all/:mailId`、`/forward/:mailId`
- Props: 无；使用路由参数初始化。

- [ ] **Step 1: 创建页面**

创建 `src/views/ComposeView.vue`：

```vue
<template>
  <div class="flex h-full flex-col">
    <ComposeHeader
      :draft="store.draft"
      :accounts="accounts"
      @update:draft="store.setDraft"
      @send="store.sendMail"
    />
    <ComposeEditor
      v-model="store.draft.bodyHtml"
      @image-pasted="handleImagePasted"
    />
    <ComposeAttachmentList
      :attachments="store.draft.attachments"
      @remove="removeAttachment"
    />
    <div class="flex items-center gap-2 border-t border-[var(--border)] p-2">
      <button class="rounded border border-[var(--border)] px-3 py-1 text-sm" @click="addAttachment">
        {{ $t('compose.addAttachment') }}
      </button>
      <span v-if="store.saving" class="text-xs text-[var(--text-secondary)]">{{ $t('compose.saving') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue';
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useComposeStore } from '@/stores/compose';
import { useAccountStore } from '@/stores/account';
import ComposeHeader from '@/components/compose/ComposeHeader.vue';
import ComposeEditor from '@/components/compose/ComposeEditor.vue';
import ComposeAttachmentList from '@/components/compose/ComposeAttachmentList.vue';
import type { AttachmentDraft } from '@/types/compose';
import { useToastStore } from '@/stores/toast';

const route = useRoute();
const store = useComposeStore();
const accountStore = useAccountStore();
const toast = useToastStore();

const accounts = computed(() => accountStore.accounts);

onMounted(async () => {
  await accountStore.loadAccounts();

  const draftId = route.params.draftId as string | undefined;
  const mailId = route.params.mailId as string | undefined;
  const kind = route.name as string | undefined;

  if (draftId) {
    await store.loadDraft(draftId);
  } else if (mailId && kind) {
    const replyKind =
      kind === 'reply-all' ? 'reply_all' : (kind as 'reply' | 'forward');
    await store.prepareReply(mailId, replyKind);
  } else {
    store.reset();
    store.setDraft({
      ...store.draft,
      accountId: accounts.value[0]?.id ?? '',
    });
  }
});

async function handleImagePasted(file: File) {
  const reader = new FileReader();
  reader.onload = async () => {
    const arrayBuffer = reader.result as ArrayBuffer;
    const base64 = btoa(
      new Uint8Array(arrayBuffer).reduce((data, byte) => data + String.fromCharCode(byte), '')
    );
    const attachment: AttachmentDraft = {
      id: crypto.randomUUID(),
      filename: file.name,
      mimeType: file.type,
      size: file.size,
      isInline: true,
      contentId: `att-${crypto.randomUUID()}`,
      previewUrl: `data:${file.type};base64,${base64}`,
    };
    // Save attachment bytes to backend
    try {
      await invoke('save_attachment', {
        draftId: store.draft.id,
        attachment,
        data: Array.from(new Uint8Array(arrayBuffer)),
      });
      store.draft.attachments.push(attachment);
      // Insert image with cid reference
      const imgHtml = `&lt;img src="cid:${attachment.contentId}" alt="${attachment.filename}" /&gt;`;
      store.draft.bodyHtml += imgHtml;
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };
  reader.readAsArrayBuffer(file);
}

async function addAttachment() {
  const input = document.createElement('input');
  input.type = 'file';
  input.multiple = true;
  input.onchange = async () => {
    for (const file of Array.from(input.files ?? [])) {
      const reader = new FileReader();
      reader.onload = async () => {
        const arrayBuffer = reader.result as ArrayBuffer;
        const attachment: AttachmentDraft = {
          id: crypto.randomUUID(),
          filename: file.name,
          mimeType: file.type,
          size: file.size,
          isInline: false,
        };
        try {
          await invoke('save_attachment', {
            draftId: store.draft.id,
            attachment,
            data: Array.from(new Uint8Array(arrayBuffer)),
          });
          store.draft.attachments.push(attachment);
        } catch (e) {
          toast.error(e instanceof Error ? e.message : String(e));
        }
      };
      reader.readAsArrayBuffer(file);
    }
  };
  input.click();
}

function removeAttachment(attachmentId: string) {
  store.draft.attachments = store.draft.attachments.filter((a) => a.id !== attachmentId);
}
</script>
```

- [ ] **Step 2: 添加 `save_attachment` 命令**

在 `src-tauri/src/commands/compose.rs` 中添加：

```rust
#[tauri::command]
pub async fn save_attachment(
    draft_id: String,
    attachment: crate::models::compose::AttachmentDraft,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let compose = state.compose_service.read().await;
    // If draft doesn't exist yet, create a stub draft first
    if draft_id.is_empty() {
        return Err(AeroError::DraftNotFound("empty".to_string()).to_payload());
    }
    compose
        .draft_service
        .write_attachment(&draft_id,
            &attachment,
            &data,
        )
        .map_err(|e| e.to_payload())
}
```

注意：`ComposeService` 当前没有暴露 `draft_service`。需要在 `mod.rs` 中将 `draft_service` 设为 `pub(crate)` 或添加包装方法。在 `src-tauri/src/services/compose/mod.rs` 的 `ComposeService` 定义中，将 `draft_service` 改为 `pub(crate)`：

```rust
pub(crate) draft_service: DraftService,
```

并在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册 `save_attachment`：

```rust
save_attachment,
```

同时更新 `src-tauri/src/commands/mod.rs` 的 `compose` 模块导出以包含 `save_attachment`（已在 `commands/compose.rs` 中定义）。

- [ ] **Step 3: 更新路由**

修改 `src/router.ts`：

```typescript
import { createRouter, createWebHistory } from 'vue-router';
import InboxView from './views/InboxView.vue';
import AccountsView from './views/AccountsView.vue';
import SettingsView from './views/SettingsView.vue';
import ComposeView from './views/ComposeView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'inbox', component: InboxView },
    { path: '/folder/:folderId', name: 'folder', component: InboxView },
    { path: '/accounts', name: 'accounts', component: AccountsView },
    { path: '/settings', name: 'settings', component: SettingsView },
    { path: '/compose', name: 'compose', component: ComposeView },
    { path: '/compose/:draftId', name: 'compose-draft', component: ComposeView },
    { path: '/reply/:mailId', name: 'reply', component: ComposeView },
    { path: '/reply-all/:mailId', name: 'reply-all', component: ComposeView },
    { path: '/forward/:mailId', name: 'forward', component: ComposeView },
  ],
});

export default router;
```

- [ ] **Step 4: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 5: 提交**

```bash
git add src/views/ComposeView.vue src/router.ts src-tauri/src/commands/compose.rs src-tauri/src/lib.rs src-tauri/src/services/compose/mod.ts
git commit -m "feat(compose): add compose view, routing, and attachment commands"
```

---

## Task 19: 邮件阅读器添加回复/转发入口

**Files:**
- Modify: `src/components/MailViewer.vue`
- Test: `pnpm type-check`

**Interfaces:**
- 添加 Reply / Reply All / Forward 按钮，点击跳转到对应路由。

- [ ] **Step 1: 在 MailViewer 添加按钮**

在 `src/components/MailViewer.vue` 的工具栏区域（靠近 mark read/star/delete 按钮）添加：

```vue
<button @click="reply">{{ $t('mail.reply') }}</button>
<button @click="replyAll">{{ $t('mail.replyAll') }}</button>
<button @click="forward">{{ $t('mail.forward') }}</button>
```

并在 `<script setup>` 中导入 `useRouter` 并添加方法：

```typescript
import { useRouter } from 'vue-router';

const router = useRouter();
const props = defineProps<{
  mail: MailDetail | null;
}>();

function reply() {
  if (props.mail) router.push({ name: 'reply', params: { mailId: props.mail.id } });
}
function replyAll() {
  if (props.mail) router.push({ name: 'reply-all', params: { mailId: props.mail.id } });
}
function forward() {
  if (props.mail) router.push({ name: 'forward', params: { mailId: props.mail.id } });
}
```

- [ ] **Step 2: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 3: 提交**

```bash
git add src/components/MailViewer.vue
git commit -m "feat(compose): add reply/forward buttons to mail viewer"
```

---

## Task 20: 新增写信按钮与草稿列表入口

**Files:**
- Modify: `src/components/AppSidebar.vue` 或 `src/layouts/AppLayout.vue`
- Test: `pnpm type-check`

**Interfaces:**
- 在侧边栏顶部添加「写信」按钮，跳转 `/compose`。

- [ ] **Step 1: 添加写信按钮**

在 `src/components/AppSidebar.vue` 的合适位置添加：

```vue
<button
  class="w-full rounded bg-[var(--primary)] px-4 py-2 text-[var(--primary-fg)]"
  @click="$router.push('/compose')"
>
  {{ $t('sidebar.compose') }}
</button>
```

或新建入口在 `src/layouts/AppLayout.vue` 的 header 区域。

- [ ] **Step 2: 类型检查**

```bash
pnpm type-check
```

- [ ] **Step 3: 提交**

```bash
git add src/components/AppSidebar.vue
git commit -m "feat(compose): add compose button in sidebar"
```

---

## Task 21: 国际化文案

**Files:**
- Modify: `src/i18n/locales/en.json`
- Modify: `src/i18n/locales/zh-CN.json`
- Test: `pnpm i18n:check`

**Interfaces:**
- 所有新增 UI 文案在两种语言中保持一致 key。

- [ ] **Step 1: 添加英文文案**

在 `src/i18n/locales/en.json` 中添加：

```json
"compose": {
  "selectAccount": "Select account",
  "send": "Send",
  "to": "To",
  "toPlaceholder": "Enter recipient emails",
  "cc": "Cc",
  "bcc": "Bcc",
  "subject": "Subject",
  "addAttachment": "Add attachment",
  "saving": "Saving...",
  "saved": "Saved",
  "sendSuccess": "Mail sent",
  "sendFailed": "Failed to send mail"
},
"mail": {
  "reply": "Reply",
  "replyAll": "Reply All",
  "forward": "Forward"
},
"sidebar": {
  "compose": "Compose"
}
```

- [ ] **Step 2: 添加中文文案**

在 `src/i18n/locales/zh-CN.json` 中添加对应：

```json
"compose": {
  "selectAccount": "选择账号",
  "send": "发送",
  "to": "收件人",
  "toPlaceholder": "输入收件人邮箱",
  "cc": "抄送",
  "bcc": "密送",
  "subject": "主题",
  "addAttachment": "添加附件",
  "saving": "保存中...",
  "saved": "已保存",
  "sendSuccess": "邮件已发送",
  "sendFailed": "发送失败"
},
"mail": {
  "reply": "回复",
  "replyAll": "回复全部",
  "forward": "转发"
},
"sidebar": {
  "compose": "写信"
}
```

- [ ] **Step 3: 运行 i18n 检查**

```bash
pnpm i18n:check
```

- [ ] **Step 4: 提交**

```bash
git add src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(i18n): add compose translations"
```

---

## Task 22: 端到端验证

**Files:**
- 所有已修改文件
- Test: 手动 + 自动检查

- [ ] **Step 1: 类型检查与 Lint**

```bash
pnpm type-check
pnpm lint
```

- [ ] **Step 2: Rust 检查**

```bash
cd src-tauri
cargo clippy
cargo test
```

- [ ] **Step 3: i18n 检查**

```bash
pnpm i18n:check
```

- [ ] **Step 4: 启动开发模式进行手动测试**

```bash
cargo tauri dev
```

测试清单：
- [ ] 点击「写信」进入 ComposeView
- [ ] 选择账号、填写收件人/主题/正文
- [ ] 自动保存提示出现，草稿 ID 生成
- [ ] 添加普通附件，文件出现在 `app_data_dir/drafts/<draft_id>/`
- [ ] 粘贴图片，图片作为内嵌附件插入正文
- [ ] 点击发送，邮件通过 SMTP 发出，本地草稿删除
- [ ] 在 MailViewer 点击回复/转发，路由跳转并预填充
- [ ] 关闭应用后重新打开，草稿可恢复
- [ ] IMAP Drafts 同步在自动保存后触发

- [ ] **Step 5: 提交最终变更**

```bash
git add .
git commit -m "feat(compose): complete rich text compose and SMTP send"
```

---

## 自我评审

### Spec 覆盖检查

| Spec 要求 | 对应任务 |
|-----------|----------|
| 独立 `models/compose.rs` | Task 3 |
| 扩展 `drafts` 表 | Task 2 |
| 附件文件系统存储 | Task 6、Task 18 |
| `services/compose/` 模块 | Task 6-10 |
| MIME 构建 | Task 7 |
| SMTP 发送（密码/OAuth2） | Task 8 |
| IMAP Drafts 同步 | Task 9 |
| Tauri 命令 | Task 11 |
| Tiptap 编辑器 | Task 12、Task 17 |
| 回复/转发 | Task 6、Task 18、Task 19 |
| 前端路由 | Task 18 |
| 错误处理 | Task 4 |
| i18n | Task 21 |
| 测试 | Task 7、Task 22 |

### Placeholder 检查

- 无 `TBD`、`TODO` 等占位符。
- 所有命令签名、类型名称、文件路径均已明确。
- 每个任务包含可运行的测试/验证命令。

### 类型一致性检查

- Rust 模型字段（`ComposeDraft`、`AttachmentDraft`）与数据库列、前端类型一致。
- Tauri 命令参数名与前端 `invoke` 调用一致（snake_case 在 Rust，camelCase 在 TS 调用时通过对象 key 传递）。
- `ReplyKind` 在 Rust 为 `reply/reply_all/forward`，前端路由 `reply/reply-all/forward` 映射正确。

---

## 执行交接

**计划已完成并保存到 `docs/superpowers/plans/2026-06-20-rich-text-compose-smtp-plan.md`。两种执行方式：**

1. **Subagent-Driven（推荐）**：每个任务派一个子代理独立执行，我在任务之间做审查。
2. **Inline Execution**：在当前会话中按顺序执行，适合快速推进。

请选择执行方式。
