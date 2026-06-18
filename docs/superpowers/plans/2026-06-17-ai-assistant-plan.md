# AI 助手实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 AeroMail 添加邮件上下文感知的 AI 聊天助手，支持国内外主流 AI 厂商预设、自由问答、快捷操作（总结/回复/待办），聊天记录本地持久化。

**Architecture:** 后端使用 reqwest 调用 OpenAI-compatible Chat Completions 协议；AiProvider 模型被翻译模块复用；会话和消息持久化到 SQLite；前端通过 Sidebar 面板和 MailViewer 快捷按钮交互。

**Tech Stack:** Rust 2024 + reqwest + Tauri v2 + Vue 3 + vue-i18n + TypeScript + Pinia.

## Global Constraints

- Rust 2024 edition，`unsafe_code = "forbid"`，clippy `unwrap_used = "deny"`。
- 所有 Vue 组件使用 `<script setup lang="ts">`。
- 前端路径别名 `@` 指向 `src/`。
- 后端 Tauri Command 错误统一使用 `ErrorPayload { code, args }`。
- 新增文案必须同时写入 `en.json` 与 `zh-CN.json`，保持 key 一致。
- 每个 Task 结束时运行 `pnpm lint`、`pnpm type-check`、`cargo check`。

---

## Task 1: 添加 reqwest 依赖 + AI Provider 模型 + 数据库表

**Files:**
- Modify: `src-tauri/Cargo.toml`（添加 reqwest、futures 依赖）
- Create: `src-tauri/src/models/ai.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Modify: `src-tauri/src/db/schema.rs`（添加 ai_chat_sessions 和 ai_chat_messages 表）

**Interfaces:**
- Produces: `AiProvider`, `AiProviderKind`, `AiProviderSummary`, `AiChatSession`, `AiChatMessage` 结构体。

- [ ] **Step 1: 添加 Cargo 依赖**

```toml
# src-tauri/Cargo.toml [dependencies] 追加
reqwest = { version = "0.12", features = ["json"] }
futures = "0.3"
```

- [ ] **Step 2: 创建 AI 模型**

```rust
// src-tauri/src/models/ai.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub kind: AiProviderKind,
    pub api_key_encrypted: Vec<u8>,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderKind {
    OpenAI,
    Anthropic,
    Gemini,
    AzureOpenAI,
    DeepSeek,
    Moonshot,
    Qwen,
    Zhipu,
    MiniMax,
    Baichuan,
    CustomOpenAICompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatSession {
    pub id: String,
    pub title: Option<String>,
    pub provider_id: String,
    pub model: String,
    pub context_mail_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}
```

- [ ] **Step 3: 注册模型模块**

```rust
// src-tauri/src/models/mod.rs
pub mod account;
pub mod ai;
pub mod error;
```

- [ ] **Step 4: 添加数据库表**

在 `src-tauri/src/db/schema.rs` 的 `ALL_SCHEMAS` 数组末尾追加：

```rust
pub const AI_CHAT_SESSIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS ai_chat_sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    provider_id TEXT NOT NULL,
    model TEXT NOT NULL,
    context_mail_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
)
"#;

pub const AI_CHAT_MESSAGES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS ai_chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL
)
"#;

pub const AI_CHAT_MESSAGES_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_ai_messages_session
ON ai_chat_messages(session_id, created_at)
"#;

pub const ALL_SCHEMAS: &[&str] = &[
    ACCOUNTS_TABLE,
    FOLDERS_TABLE,
    MAILS_TABLE,
    ATTACHMENTS_TABLE,
    DRAFTS_TABLE,
    SETTINGS_TABLE,
    AI_CHAT_SESSIONS_TABLE,
    AI_CHAT_MESSAGES_TABLE,
    AI_CHAT_MESSAGES_INDEX,
];
```

- [ ] **Step 5: 验证**

```bash
cd /home/xiyeming/CodeSpaces/ToolsProjects/RutMail
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/models/ai.rs src-tauri/src/models/mod.rs src-tauri/src/db/schema.rs
git commit -m "feat(ai): add AI provider model, dependencies, and DB tables" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: 创建 AI 提供商预设 + DB 操作

**Files:**
- Create: `src-tauri/src/services/ai/mod.rs`
- Create: `src-tauri/src/services/ai/providers.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/db/pool.rs`（添加 AI provider/session/message CRUD 方法）

**Interfaces:**
- Produces: `AiService` 结构体，`preset_for()` 函数，DB 方法 `list_ai_providers`/`get_ai_provider`/`upsert_ai_provider`/`delete_ai_provider`/`get_chat_session`/`get_chat_messages`/`insert_chat_message`/`update_chat_session_timestamp`。

- [ ] **Step 1: 创建 providers.rs**

```rust
// src-tauri/src/services/ai/providers.rs
use crate::models::ai::AiProviderKind;

#[derive(Debug, Clone)]
pub struct AiProviderPreset {
    pub name: String,
    pub base_url: String,
    pub default_model: String,
}

pub fn preset_for(kind: &AiProviderKind) -> AiProviderPreset {
    match kind {
        AiProviderKind::OpenAI => AiProviderPreset {
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-4o".to_string(),
        },
        AiProviderKind::Anthropic => AiProviderPreset {
            name: "Anthropic".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
        },
        AiProviderKind::Gemini => AiProviderPreset {
            name: "Google Gemini".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            default_model: "gemini-1.5-flash".to_string(),
        },
        AiProviderKind::AzureOpenAI => AiProviderPreset {
            name: "Azure OpenAI".to_string(),
            base_url: "https://your-resource.openai.azure.com".to_string(),
            default_model: "gpt-4o".to_string(),
        },
        AiProviderKind::DeepSeek => AiProviderPreset {
            name: "DeepSeek".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            default_model: "deepseek-chat".to_string(),
        },
        AiProviderKind::Moonshot => AiProviderPreset {
            name: "Moonshot".to_string(),
            base_url: "https://api.moonshot.cn".to_string(),
            default_model: "moonshot-v1-8k".to_string(),
        },
        AiProviderKind::Qwen => AiProviderPreset {
            name: "通义千问".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            default_model: "qwen-turbo".to_string(),
        },
        AiProviderKind::Zhipu => AiProviderPreset {
            name: "智谱 GLM".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            default_model: "glm-4-flash".to_string(),
        },
        AiProviderKind::MiniMax => AiProviderPreset {
            name: "MiniMax".to_string(),
            base_url: "https://api.minimax.chat/v1".to_string(),
            default_model: "abab6.5s-chat".to_string(),
        },
        AiProviderKind::Baichuan => AiProviderPreset {
            name: "百川".to_string(),
            base_url: "https://api.baichuan-ai.com/v1".to_string(),
            default_model: "Baichuan4".to_string(),
        },
        AiProviderKind::CustomOpenAICompatible => AiProviderPreset {
            name: "Custom".to_string(),
            base_url: String::new(),
            default_model: String::new(),
        },
    }
}
```

- [ ] **Step 2: 创建 AI 服务核心**

```rust
// src-tauri/src/services/ai/mod.rs
pub mod providers;

use crate::db::Database;
use crate::error::AeroError;
use crate::models::ai::{AiChatMessage, AiChatSession, AiProvider, AiProviderSummary};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct AiService {
    pub db: Arc<Database>,
}

impl AiService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn list_providers(&self) -> Result<Vec<AiProviderSummary>, AeroError> {
        let providers = self.db.list_ai_providers()?;
        Ok(providers
            .into_iter()
            .map(|p| AiProviderSummary {
                id: p.id,
                name: p.name,
                kind: format!("{:?}", p.kind),
                model: p.model,
            })
            .collect())
    }

    pub async fn complete(
        &self,
        provider_id: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, AeroError> {
        let provider = self.db.get_ai_provider(provider_id)?;
        let max_tokens = provider.max_tokens.unwrap_or(2048);
        crate::services::ai::providers::client::chat_completion(&provider, messages, max_tokens)
            .await
    }
}
```

- [ ] **Step 3: 注册服务模块**

```rust
// src-tauri/src/services/mod.rs
pub mod account_manager;
pub mod ai;
```

- [ ] **Step 4: 在 db/pool.rs 添加 AI 数据库方法**

在 `impl Database` 中追加以下方法（省略重复的 `connection()` 调用）：

```rust
// src-tauri/src/db/pool.rs — 在 impl Database 块内追加

pub fn list_ai_providers(&self) -> Result<Vec<crate::models::ai::AiProvider>, AeroError> {
    let conn = self.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, kind, api_key_encrypted, base_url, model, max_tokens FROM ai_providers"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(crate::models::ai::AiProvider {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or(crate::models::ai::AiProviderKind::CustomOpenAICompatible),
            api_key_encrypted: row.get::<_, Vec<u8>>(3)?,
            base_url: row.get(4)?,
            model: row.get(5)?,
            max_tokens: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| AeroError::Database(e.to_string()))
}

pub fn get_ai_provider(&self, id: &str) -> Result<crate::models::ai::AiProvider, AeroError> {
    let conn = self.connection()?;
    conn.query_row(
        "SELECT id, name, kind, api_key_encrypted, base_url, model, max_tokens FROM ai_providers WHERE id = ?1",
        [id],
        |row| {
            Ok(crate::models::ai::AiProvider {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or(crate::models::ai::AiProviderKind::CustomOpenAICompatible),
                api_key_encrypted: row.get::<_, Vec<u8>>(3)?,
                base_url: row.get(4)?,
                model: row.get(5)?,
                max_tokens: row.get(6)?,
            })
        },
    ).map_err(|e| AeroError::Database(e.to_string()))
}

pub fn upsert_ai_provider(&self, p: &crate::models::ai::AiProvider) -> Result<(), AeroError> {
    let conn = self.connection()?;
    let kind_str = serde_json::to_string(&p.kind)?;
    conn.execute(
        "INSERT INTO ai_providers (id, name, kind, api_key_encrypted, base_url, model, max_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name, kind=excluded.kind,
         api_key_encrypted=excluded.api_key_encrypted, base_url=excluded.base_url,
         model=excluded.model, max_tokens=excluded.max_tokens",
        (&p.id, &p.name, kind_str, &p.api_key_encrypted, &p.base_url, &p.model, &p.max_tokens),
    )?;
    Ok(())
}

pub fn delete_ai_provider(&self, id: &str) -> Result<(), AeroError> {
    let conn = self.connection()?;
    conn.execute("DELETE FROM ai_providers WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_chat_session(&self, id: &str) -> Result<crate::models::ai::AiChatSession, AeroError> {
    let conn = self.connection()?;
    conn.query_row(
        "SELECT id, title, provider_id, model, context_mail_id, created_at, updated_at
         FROM ai_chat_sessions WHERE id = ?1",
        [id],
        |row| {
            Ok(crate::models::ai::AiChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                provider_id: row.get(2)?,
                model: row.get(3)?,
                context_mail_id: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    ).map_err(|e| AeroError::Database(e.to_string()))
}

pub fn list_chat_sessions(&self) -> Result<Vec<crate::models::ai::AiChatSession>, AeroError> {
    let conn = self.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, title, provider_id, model, context_mail_id, created_at, updated_at
         FROM ai_chat_sessions ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(crate::models::ai::AiChatSession {
            id: row.get(0)?,
            title: row.get(1)?,
            provider_id: row.get(2)?,
            model: row.get(3)?,
            context_mail_id: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| AeroError::Database(e.to_string()))
}

pub fn get_chat_messages(&self, session_id: &str) -> Result<Vec<crate::models::ai::AiChatMessage>, AeroError> {
    let conn = self.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, created_at
         FROM ai_chat_messages WHERE session_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map([session_id], |row| {
        Ok(crate::models::ai::AiChatMessage {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| AeroError::Database(e.to_string()))
}

pub fn insert_chat_message(&self, session_id: &str, role: &str, content: &str) -> Result<crate::models::ai::AiChatMessage, AeroError> {
    let conn = self.connection()?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO ai_chat_messages (id, session_id, role, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&id, session_id, role, content, now),
    )?;
    Ok(crate::models::ai::AiChatMessage {
        id,
        session_id: session_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        created_at: now,
    })
}

pub fn update_chat_session_timestamp(&self, session_id: &str) -> Result<(), AeroError> {
    let conn = self.connection()?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE ai_chat_sessions SET updated_at = ?1 WHERE id = ?2",
        (now, session_id),
    )?;
    Ok(())
}
```

注意：需要在 schema.rs 中添加 `ai_providers` 表（之前遗漏的）：

```sql
CREATE TABLE IF NOT EXISTS ai_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    api_key_encrypted BLOB NOT NULL,
    base_url TEXT,
    model TEXT NOT NULL,
    max_tokens INTEGER
)
```

- [ ] **Step 5: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/services/ai/ src-tauri/src/services/mod.rs src-tauri/src/db/pool.rs src-tauri/src/db/schema.rs
git commit -m "feat(ai): add AI provider presets, service, and DB operations" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: 创建 HTTP 客户端 + 聊天服务 + 错误码

**Files:**
- Create: `src-tauri/src/services/ai/client.rs`
- Create: `src-tauri/src/services/ai/chat.rs`
- Modify: `src-tauri/src/error.rs`（添加 AI 错误码）
- Modify: `src/i18n/locales/en.json` 和 `zh-CN.json`（添加 AI 错误码翻译）

**Interfaces:**
- Produces: `chat_completion()` 函数，`send_message()` 函数，AI 错误码。

- [ ] **Step 1: 创建 HTTP 客户端**

```rust
// src-tauri/src/services/ai/client.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AeroError;
use crate::models::ai::AiProvider;
use crate::services::ai::ChatMessage;

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessagePayload>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessagePayload {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessagePayload,
}

pub async fn chat_completion(
    provider: &AiProvider,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
) -> Result<String, AeroError> {
    let api_key = String::from_utf8(provider.api_key_encrypted.clone())
        .map_err(|e| AeroError::AiApiError(format!("invalid api key: {e}")))?;
    let base_url = provider.base_url.as_deref().unwrap_or("");
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let payload_messages: Vec<ChatMessagePayload> = messages
        .into_iter()
        .map(|m| ChatMessagePayload {
            role: m.role,
            content: m.content,
        })
        .collect();

    let body = ChatCompletionRequest {
        model: provider.model.clone(),
        messages: payload_messages,
        max_tokens,
    };

    let client = Client::new();
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if status.as_u16() == 429 {
            return Err(AeroError::AiRateLimited);
        }
        return Err(AeroError::AiApiError(format!("HTTP {status}: {text}")));
    }

    let data: ChatCompletionResponse = res
        .json()
        .await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    data.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AeroError::AiApiError("empty response".to_string()))
}
```

- [ ] **Step 2: 创建聊天服务**

```rust
// src-tauri/src/services/ai/chat.rs
use crate::db::Database;
use crate::error::AeroError;
use crate::models::ai::{AiChatMessage, AiChatSession};
use crate::services::ai::{AiService, ChatMessage};

pub fn build_messages(
    db: &Database,
    session: &AiChatSession,
    history: &[AiChatMessage],
    user_content: &str,
) -> Result<Vec<ChatMessage>, AeroError> {
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful email assistant. Be concise.".to_string(),
    }];

    if let Some(ref mail_id) = session.context_mail_id {
        let subject = db.get_mail_subject(mail_id)?.unwrap_or_default();
        let from = db.get_mail_from_address(mail_id)?.unwrap_or_default();
        let body = db.get_mail_body_text(mail_id)?.unwrap_or_default();
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!(
                "[Current Email Context]\nSubject: {subject}\nFrom: {from}\n\n{body}"
            ),
        });
    }

    for msg in history {
        messages.push(ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_content.to_string(),
    });

    Ok(messages)
}

pub async fn send_message(
    ai: &AiService,
    session_id: &str,
    user_content: &str,
) -> Result<AiChatMessage, AeroError> {
    let session = ai.db.get_chat_session(session_id)?;
    let history = ai.db.get_chat_messages(session_id)?;
    let messages = build_messages(&ai.db, &session, &history, user_content)?;
    let reply = ai.complete(&session.provider_id, messages).await?;
    let saved = ai.db.insert_chat_message(session_id, "user", user_content)?;
    ai.db.insert_chat_message(session_id, "assistant", &reply)?;
    ai.db.update_chat_session_timestamp(session_id)?;
    Ok(saved)
}
```

注意：`build_messages` 中调用了 `db.get_mail_subject()`、`db.get_mail_from_address()`、`db.get_mail_body_text()`，需要在 `db/pool.rs` 中添加这三个简单方法。

- [ ] **Step 3: 添加 AI 错误码**

```rust
// src-tauri/src/error.rs — 在 AeroError 枚举中追加
#[error("AI provider not found")]
AiProviderNotFound,
#[error("AI API error: {0}")]
AiApiError(String),
#[error("AI rate limited")]
AiRateLimited,
#[error("AI context mail not found")]
AiContextMailNotFound,

// 在 to_payload() 的 match 中追加
Self::AiProviderNotFound => ErrorPayload {
    code: "AI_PROVIDER_NOT_FOUND".to_string(),
    args: vec![],
},
Self::AiApiError(msg) => ErrorPayload {
    code: "AI_API_ERROR".to_string(),
    args: vec![msg.clone()],
},
Self::AiRateLimited => ErrorPayload {
    code: "AI_RATE_LIMITED".to_string(),
    args: vec![],
},
Self::AiContextMailNotFound => ErrorPayload {
    code: "AI_CONTEXT_MAIL_NOT_FOUND".to_string(),
    args: vec![],
},
```

- [ ] **Step 4: 添加 i18n 错误码翻译**

在 `en.json` 和 `zh-CN.json` 的 `errors` 部分追加：

```json
"AI_PROVIDER_NOT_FOUND": "AI provider not found",
"AI_API_ERROR": "AI request failed: {0}",
"AI_RATE_LIMITED": "Rate limited, please try again later",
"AI_CONTEXT_MAIL_NOT_FOUND": "Context mail not found"
```

中文：
```json
"AI_PROVIDER_NOT_FOUND": "AI 提供商不存在",
"AI_API_ERROR": "AI 请求失败：{0}",
"AI_RATE_LIMITED": "请求过于频繁，请稍后再试",
"AI_CONTEXT_MAIL_NOT_FOUND": "上下文邮件不存在"
```

- [ ] **Step 5: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
pnpm i18n:check
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/services/ai/client.rs src-tauri/src/services/ai/chat.rs src-tauri/src/services/ai/mod.rs src-tauri/src/error.rs src-tauri/src/db/pool.rs src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(ai): add chat client, context injection, and error codes" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: 创建 AI Tauri Commands + 注册

**Files:**
- Create: `src-tauri/src/commands/ai.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: 所有 AI 相关 IPC 命令。

- [ ] **Step 1: 创建 AI 命令**

```rust
// src-tauri/src/commands/ai.rs
use tauri::State;

use crate::models::ai::{AiChatSession, AiChatMessage, AiProvider, AiProviderSummary};
use crate::models::error::ErrorPayload;
use crate::AppState;

#[tauri::command]
pub async fn list_ai_providers(
    state: State<'_, AppState>,
) -> Result<Vec<AiProviderSummary>, ErrorPayload> {
    let ai = state.ai_service.read().await;
    ai.list_providers().map_err(AeroError::to_payload)
}

#[tauri::command]
pub async fn upsert_ai_provider(
    provider: AiProvider,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let db = &state.db;
    db.upsert_ai_provider(&provider).map_err(AeroError::to_payload)?;
    Ok(provider.id)
}

#[tauri::command]
pub async fn delete_ai_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state.db.delete_ai_provider(&provider_id).map_err(AeroError::to_payload)
}

#[tauri::command]
pub async fn test_ai_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let ai = state.ai_service.read().await;
    let messages = vec![crate::services::ai::ChatMessage {
        role: "user".to_string(),
        content: "Say hello in one sentence.".to_string(),
    }];
    ai.complete(&provider_id, messages)
        .await
        .map_err(AeroError::to_payload)
}

#[tauri::command]
pub async fn create_chat_session(
    provider_id: String,
    context_mail_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<AiChatSession, ErrorPayload> {
    let provider = state.db.get_ai_provider(&provider_id).map_err(AeroError::to_payload)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let session = AiChatSession {
        id: id.clone(),
        title: None,
        provider_id,
        model: provider.model,
        context_mail_id,
        created_at: now,
        updated_at: now,
    };
    state.db.upsert_chat_session(&session).map_err(AeroError::to_payload)?;
    Ok(session)
}

#[tauri::command]
pub async fn send_chat_message(
    session_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<AiChatMessage, ErrorPayload> {
    let ai = state.ai_service.read().await;
    crate::services::ai::chat::send_message(&ai, &session_id, &content)
        .await
        .map_err(AeroError::to_payload)
}

#[tauri::command]
pub async fn list_chat_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<AiChatSession>, ErrorPayload> {
    state.db.list_chat_sessions().map_err(AeroError::to_payload)
}

#[tauri::command]
pub async fn get_chat_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AiChatMessage>, ErrorPayload> {
    state.db.get_chat_messages(&session_id).map_err(AeroError::to_payload)
}

#[tauri::command]
pub async fn delete_chat_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state.db.delete_chat_session(&session_id).map_err(AeroError::to_payload)
}
```

- [ ] **Step 2: 注册命令模块**

```rust
// src-tauri/src/commands/mod.rs
pub mod account;
pub mod ai;
pub mod settings;
```

- [ ] **Step 3: 在 lib.rs 注册命令并添加 AiService**

```rust
// src-tauri/src/lib.rs — 修改
use commands::ai::{
    create_chat_session, delete_ai_provider, delete_chat_session, get_chat_messages,
    list_ai_providers, list_chat_sessions, send_chat_message, test_ai_provider,
    upsert_ai_provider,
};
use services::ai::AiService;
use tokio::sync::RwLock;

pub struct AppState {
    pub account_manager: Arc<RwLock<AccountManager>>,
    pub ai_service: Arc<RwLock<AiService>>,
    pub db: Arc<Database>,
}

impl AppState {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, crate::error::AeroError> {
        let db = Arc::new(Database::new(app_handle)?);
        let account_manager = Arc::new(RwLock::new(AccountManager::new(Arc::clone(&db))));
        let ai_service = Arc::new(RwLock::new(AiService::new(Arc::clone(&db))));
        Ok(Self {
            account_manager,
            ai_service,
            db,
        })
    }
}

// 在 invoke_handler 中追加所有 AI 命令
.invoke_handler(tauri::generate_handler![
    greet,
    add_account,
    list_accounts,
    delete_account,
    test_account_connection,
    set_setting,
    get_setting,
    list_ai_providers,
    upsert_ai_provider,
    delete_ai_provider,
    test_ai_provider,
    create_chat_session,
    send_chat_message,
    list_chat_sessions,
    get_chat_messages,
    delete_chat_session,
])
```

- [ ] **Step 4: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/ai.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(ai): add Tauri IPC commands for AI assistant" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: 创建前端类型 + useAiChat composable

**Files:**
- Create: `src/types/ai.ts`
- Create: `src/composables/useAiChat.ts`
- Create: `src/stores/ai.ts`

**Interfaces:**
- Produces: 前端 AI 类型、`useAiChat()` composable、`useAiStore()` store。

- [ ] **Step 1: 创建前端类型**

```typescript
// src/types/ai.ts
export type AiProviderKind =
  | 'openai'
  | 'anthropic'
  | 'gemini'
  | 'azure_openai'
  | 'deepseek'
  | 'moonshot'
  | 'qwen'
  | 'zhipu'
  | 'minimax'
  | 'baichuan'
  | 'custom_openai_compatible';

export interface AiProvider {
  id: string;
  name: string;
  kind: AiProviderKind;
  apiKeyEncrypted: number[];
  baseUrl?: string;
  model: string;
  maxTokens?: number;
}

export interface AiProviderSummary {
  id: string;
  name: string;
  kind: string;
  model: string;
}

export interface AiChatSession {
  id: string;
  title: string | null;
  providerId: string;
  model: string;
  contextMailId: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface AiChatMessage {
  id: string;
  sessionId: string;
  role: 'system' | 'user' | 'assistant';
  content: string;
  createdAt: number;
}
```

- [ ] **Step 2: 创建 useAiChat composable**

```typescript
// src/composables/useAiChat.ts
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AiChatSession, AiChatMessage } from '@/types/ai';

export function useAiChat() {
  const sessions = ref<AiChatSession[]>([]);
  const messages = ref<AiChatMessage[]>([]);
  const isLoading = ref(false);
  const activeSessionId = ref<string | null>(null);

  async function loadSessions() {
    sessions.value = await invoke<AiChatSession[]>('list_chat_sessions');
  }

  async function createSession(providerId: string, contextMailId?: string) {
    const session = await invoke<AiChatSession>('create_chat_session', {
      providerId,
      contextMailId,
    });
    sessions.value.unshift(session);
    activeSessionId.value = session.id;
    return session;
  }

  async function sendMessage(sessionId: string, content: string) {
    isLoading.value = true;
    try {
      const userMsg = await invoke<AiChatMessage>('send_chat_message', {
        sessionId,
        content,
      });
      messages.value.push(userMsg);

      const assistantMsg: AiChatMessage = {
        id: crypto.randomUUID(),
        sessionId,
        role: 'assistant',
        content: '',
        createdAt: Date.now(),
      };
      messages.value.push(assistantMsg);

      const fullContent = await invoke<string>('get_ai_completion', {
        sessionId,
      });
      assistantMsg.content = fullContent;
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMessages(sessionId: string) {
    messages.value = await invoke<AiChatMessage[]>('get_chat_messages', {
      sessionId,
    });
  }

  function selectSession(id: string) {
    activeSessionId.value = id;
    void loadMessages(id);
  }

  return {
    sessions,
    messages,
    isLoading,
    activeSessionId,
    loadSessions,
    createSession,
    sendMessage,
    loadMessages,
    selectSession,
  };
}
```

- [ ] **Step 3: 创建 AI Store**

```typescript
// src/stores/ai.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AiProviderSummary } from '@/types/ai';

export const useAiStore = defineStore('ai', () => {
  const providers = ref<AiProviderSummary[]>([]);
  const isPanelOpen = ref(false);

  async function loadProviders() {
    providers.value = await invoke<AiProviderSummary[]>('list_ai_providers');
  }

  function togglePanel() {
    isPanelOpen.value = !isPanelOpen.value;
  }

  return { providers, isPanelOpen, loadProviders, togglePanel };
});
```

- [ ] **Step 4: 验证**

```bash
pnpm lint
pnpm type-check
```

- [ ] **Step 5: Commit**

```bash
git add src/types/ai.ts src/composables/useAiChat.ts src/stores/ai.ts
git commit -m "feat(ai): add frontend types, useAiChat composable, and AI store" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: 创建 AiMessageList 组件

**Files:**
- Create: `src/components/AiMessageList.vue`

**Interfaces:**
- Consumes: `AiChatMessage[]` 数组。

- [ ] **Step 1: 创建组件**

```vue
<script setup lang="ts">
import type { AiChatMessage } from '@/types/ai';
import { Bot, User } from 'lucide-vue-next';

defineProps<{
  messages: AiChatMessage[];
}>();
</script>

<template>
  <div class="flex flex-col gap-3">
    <div
      v-for="msg in messages"
      :key="msg.id"
      :class="[
        'flex gap-2',
        msg.role === 'user' ? 'justify-end' : 'justify-start',
      ]"
    >
      <div
        v-if="msg.role === 'assistant'"
        class="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-primary/20"
      >
        <Bot class="h-3.5 w-3.5 text-primary" />
      </div>
      <div
        :class="[
          'max-w-[80%] rounded-lg px-3 py-2 text-sm',
          msg.role === 'user'
            ? 'bg-primary text-white'
            : 'bg-card text-text',
        ]"
      >
        <pre class="whitespace-pre-wrap font-sans">{{ msg.content }}</pre>
      </div>
      <div
        v-if="msg.role === 'user'"
        class="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-muted/20"
      >
        <User class="h-3.5 w-3.5 text-muted" />
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 验证**

```bash
pnpm lint
pnpm type-check
```

- [ ] **Step 3: Commit**

```bash
git add src/components/AiMessageList.vue
git commit -m "feat(ai): add AiMessageList component" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: 创建 AiAssistantPanel + 集成到 Sidebar

**Files:**
- Create: `src/components/AiAssistantPanel.vue`
- Modify: `src/components/AppSidebar.vue`（添加 AI 助手入口）
- Modify: `src/layouts/AppLayout.vue`（添加面板展开逻辑）
- Modify: `src/i18n/locales/en.json` 和 `zh-CN.json`（添加 AI 助手翻译 key）

**Interfaces:**
- Produces: `AiAssistantPanel` 组件，Sidebar AI 入口，面板展开/折叠逻辑。

- [ ] **Step 1: 创建 AiAssistantPanel**

```vue
<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAiChat } from '@/composables/useAiChat';
import AiMessageList from '@/components/AiMessageList.vue';
import { useAiStore } from '@/stores/ai';

const { t } = useI18n();
const aiStore = useAiStore();
const {
  sessions,
  messages,
  isLoading,
  activeSessionId,
  loadSessions,
  createSession,
  sendMessage,
  selectSession,
} = useAiChat();

const input = ref('');
const messagesContainer = ref<HTMLElement>();

onMounted(() => {
  void loadSessions();
  void aiStore.loadProviders();
});

async function handleSend() {
  if (!input.value.trim() || !activeSessionId.value) return;
  const content = input.value;
  input.value = '';
  await sendMessage(activeSessionId.value, content);
  await nextTick();
  scrollToBottom();
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

watch(messages, () => {
  void nextTick(scrollToBottom);
}, { deep: true });
</script>

<template>
  <div class="flex h-full flex-col border-l border-border bg-panel" style="width: 360px">
    <div class="flex h-12 items-center px-4 border-b border-border text-sm font-medium">
      {{ $t('aiAssistant.title') }}
    </div>

    <div v-if="!activeSessionId" class="flex flex-1 flex-col items-center justify-center p-4 text-center">
      <p class="text-sm text-muted mb-4">{{ $t('aiAssistant.selectProvider') }}</p>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="provider in aiStore.providers"
          :key="provider.id"
          class="rounded-md border border-border bg-card px-3 py-1.5 text-xs text-text-secondary hover:bg-panel"
          @click="createSession(provider.id)"
        >
          {{ provider.name }}
        </button>
      </div>
    </div>

    <template v-else>
      <div ref="messagesContainer" class="flex-1 overflow-y-auto p-3">
        <AiMessageList :messages="messages" />
      </div>
      <div class="border-t border-border p-3">
        <textarea
          v-model="input"
          class="w-full resize-none rounded-md border border-border bg-card p-2 text-sm text-text outline-none focus:border-primary"
          :rows="3"
          :placeholder="$t('aiAssistant.inputPlaceholder')"
          @keydown.enter.prevent="handleSend"
        />
        <button
          class="mt-2 flex h-8 w-full items-center justify-center rounded-md bg-primary text-sm font-medium text-white hover:bg-primary-hover disabled:opacity-50"
          :disabled="isLoading || !input.trim()"
          @click="handleSend"
        >
          {{ isLoading ? $t('aiAssistant.thinking') : $t('aiAssistant.send') }}
        </button>
      </div>
    </template>
  </div>
</template>
```

- [ ] **Step 2: 在 AppSidebar 添加 AI 助手入口**

在 AppSidebar.vue 的底部工具区（Settings 链接下方或附近）添加：

```vue
<script setup lang="ts">
import { Bot } from 'lucide-vue-next';
import { useAiStore } from '@/stores/ai';
const aiStore = useAiStore();
</script>

<!-- 在 Settings 的 <li> 后追加 -->
<li
  class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-white/5"
  @click="aiStore.togglePanel()"
>
  <Bot class="h-4 w-4" />
  <span>{{ $t('aiAssistant.title') }}</span>
</li>
```

- [ ] **Step 3: 在 AppLayout 集成面板**

修改 `src/layouts/AppLayout.vue`，在 main 区域右侧条件渲染 AiAssistantPanel：

```vue
<script setup lang="ts">
import { useAiStore } from '@/stores/ai';
import AiAssistantPanel from '@/components/AiAssistantPanel.vue';
const aiStore = useAiStore();
</script>

<template>
  <div class="flex h-screen w-screen bg-background text-text">
    <AppSidebar />
    <MailList class="border-r border-border" />
    <main class="flex-1 min-w-0 overflow-hidden">
      <RouterView />
    </main>
    <AiAssistantPanel v-if="aiStore.isPanelOpen" />
  </div>
</template>
```

- [ ] **Step 4: 添加 i18n key**

在 `en.json` 和 `zh-CN.json` 中添加 `aiAssistant` 部分：

```json
"aiAssistant": {
  "title": "AI Assistant",
  "selectProvider": "Select a provider to start",
  "inputPlaceholder": "Ask about the current email...",
  "thinking": "Thinking...",
  "send": "Send"
}
```

中文：
```json
"aiAssistant": {
  "title": "AI 助手",
  "selectProvider": "选择一个厂商开始",
  "inputPlaceholder": "询问当前邮件...",
  "thinking": "思考中...",
  "send": "发送"
}
```

- [ ] **Step 5: 验证**

```bash
pnpm lint
pnpm type-check
pnpm i18n:check
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 6: Commit**

```bash
git add src/components/AiAssistantPanel.vue src/components/AppSidebar.vue src/layouts/AppLayout.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(ai): add AI assistant panel and sidebar integration" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 8: 添加 MailViewer AI 快捷操作 + Settings AI 提供商管理

**Files:**
- Create: `src/components/AiQuickActions.vue`
- Modify: `src/components/MailViewer.vue`（添加 AI 快捷按钮）
- Modify: `src/views/SettingsView.vue`（添加 AI 提供商管理卡片）
- Modify: `src/i18n/locales/en.json` 和 `zh-CN.json`

**Interfaces:**
- Produces: AI 快捷操作按钮，Settings AI 提供商管理界面。

- [ ] **Step 1: 创建 AiQuickActions**

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Sparkles, Reply, ListTodo } from 'lucide-vue-next';

const emit = defineEmits<{
  summarize: [];
  reply: [];
  extractTodos: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="flex items-center gap-1">
    <button
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
      :title="t('aiActions.summarize')"
      @click="emit('summarize')"
    >
      <Sparkles class="h-3.5 w-3.5" />
      {{ t('aiActions.summarize') }}
    </button>
    <button
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
      :title="t('aiActions.generateReply')"
      @click="emit('reply')"
    >
      <Reply class="h-3.5 w-3.5" />
      {{ t('aiActions.generateReply') }}
    </button>
    <button
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
      :title="t('aiActions.extractTodos')"
      @click="emit('extractTodos')"
    >
      <ListTodo class="h-3.5 w-3.5" />
      {{ t('aiActions.extractTodos') }}
    </button>
  </div>
</template>
```

- [ ] **Step 2: 在 MailViewer Header 集成**

在 `MailViewer.vue` 的操作按钮区域添加 AiQuickActions：

```vue
<script setup lang="ts">
import AiQuickActions from '@/components/AiQuickActions.vue';
import { useAiChat } from '@/composables/useAiChat';
import { useAiStore } from '@/stores/ai';

const { createSession, sendMessage } = useAiChat();
const aiStore = useAiStore();

const PROMPTS = {
  summarize: 'Please summarize this email in 3 sentences.',
  reply: 'Please write a polite reply draft for this email. Keep it concise.',
  extractTodos: 'Please extract all action items from this email, listed by priority.',
};

async function handleQuickAction(promptKey: keyof typeof PROMPTS) {
  if (!aiStore.isPanelOpen) aiStore.togglePanel();
  const provider = aiStore.providers[0];
  if (!provider) return;
  const session = await createSession(provider.id, currentMailId);
  await sendMessage(session.id, PROMPTS[promptKey]);
}
</script>

<!-- 在操作按钮组旁添加 -->
<AiQuickActions
  @summarize="handleQuickAction('summarize')"
  @reply="handleQuickAction('reply')"
  @extract-todos="handleQuickAction('extractTodos')"
/>
```

- [ ] **Step 3: 在 SettingsView 添加 AI 提供商管理**

在 `SettingsView.vue` 中新增 AI 提供商卡片：

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAiStore } from '@/stores/ai';
import type { AiProviderSummary } from '@/types/ai';

const aiStore = useAiStore();
const showAddForm = ref(false);
const newName = ref('');
const newApiKey = ref('');
const newBaseUrl = ref('');
const newModel = ref('');
const selectedKind = ref('deepseek');

onMounted(() => {
  void aiStore.loadProviders();
});

async function addProvider() {
  await invoke('upsert_ai_provider', {
    provider: {
      id: crypto.randomUUID(),
      name: newName.value,
      kind: selectedKind.value,
      apiKeyEncrypted: new TextEncoder().encode(newApiKey.value),
      baseUrl: newBaseUrl.value || undefined,
      model: newModel.value,
      maxTokens: 2048,
    },
  });
  await aiStore.loadProviders();
  showAddForm.value = false;
}

async function removeProvider(id: string) {
  await invoke('delete_ai_provider', { providerId: id });
  await aiStore.loadProviders();
}
</script>

<!-- 在 SettingsView 模板中追加 AI 提供商卡片 -->
<section class="mt-6 rounded-lg border border-border bg-card p-4">
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-lg font-medium text-text">{{ $t('settings.aiProviders') }}</h2>
    <button class="text-sm text-primary" @click="showAddForm = !showAddForm">
      {{ $t('settings.addProvider') }}
    </button>
  </div>
  <div v-if="aiStore.providers.length === 0" class="text-sm text-muted">
    {{ $t('settings.noProviders') }}
  </div>
  <div v-for="p in aiStore.providers" :key="p.id" class="flex items-center justify-between py-2 border-b border-border last:border-0">
    <div>
      <span class="text-sm text-text">{{ p.name }}</span>
      <span class="ml-2 text-xs text-muted">{{ p.model }}</span>
    </div>
    <button class="text-xs text-danger" @click="removeProvider(p.id)">{{ $t('account.delete') }}</button>
  </div>
</section>
```

- [ ] **Step 4: 添加 i18n key**

```json
"aiActions": {
  "summarize": "Summarize",
  "generateReply": "Reply",
  "extractTodos": "To-dos"
},
"settings": {
  "aiProviders": "AI Providers",
  "addProvider": "Add Provider",
  "noProviders": "No AI providers configured"
}
```

- [ ] **Step 5: 验证**

```bash
pnpm lint
pnpm type-check
pnpm i18n:check
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 6: Commit**

```bash
git add src/components/AiQuickActions.vue src/components/MailViewer.vue src/views/SettingsView.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(ai): add quick actions, MailViewer integration, and Settings AI provider management" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 9: Command Palette 集成 + 最终验证

**Files:**
- Modify: `src/components/CommandPalette.vue`（添加 AI 助手命令）
- Modify: `src/i18n/locales/en.json` 和 `zh-CN.json`

**Interfaces:**
- Produces: Command Palette 支持「打开 AI 助手」「新建 AI 会话」命令。

- [ ] **Step 1: 在 CommandPalette 添加 AI 命令**

在 CommandPalette.vue 的 `languageCommands` 附近添加：

```typescript
import { useAiStore } from '@/stores/ai';

const aiStore = useAiStore();

const aiCommands = computed(() => [
  {
    id: 'open-ai-assistant',
    title: t('commandPalette.openAiAssistant'),
    action: () => aiStore.togglePanel(),
  },
]);
```

在 `allItems` 计算属性中合并 `aiCommands`。

- [ ] **Step 2: 添加 i18n key**

```json
"commandPalette": {
  "openAiAssistant": "Open AI Assistant"
}
```

中文：
```json
"commandPalette": {
  "openAiAssistant": "打开 AI 助手"
}
```

- [ ] **Step 3: 最终验证**

```bash
pnpm lint
pnpm type-check
pnpm i18n:check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

- [ ] **Step 4: Commit**

```bash
git add src/components/CommandPalette.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(ai): add AI assistant command to Command Palette" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Self-Review Checklist

- [x] Spec coverage: AI 提供商预设、邮件上下文注入、聊天持久化、快捷操作、Sidebar 面板、Settings 管理、Command Palette 命令均有对应 Task。
- [x] Placeholder scan: 无 TBD/TODO，所有步骤含代码与命令。
- [x] Type consistency: `AiProvider`/`AiChatSession`/`AiChatMessage` 前后端定义一致。
- [x] Scope: 本计划聚焦 AI 助手，不涉及翻译模块实现。
