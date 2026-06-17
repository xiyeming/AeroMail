# AeroMail AI 助手设计规范

> **范围**：内置国内外主流 AI 厂商预设的邮件上下文感知聊天助手；支持总结、回复建议、自由问答；聊天记录本地持久化。
> **前置依赖**：i18n 错误码规范；`AiProvider` 模型被翻译模块复用。

---

## 1. 总体架构

```text
前端 Vue 3
  Sidebar ──> AiAssistantPanel.vue
  MailViewer Header ──> AiQuickActions.vue
  stores/ai.ts + composables/useAiChat.ts
       │
       │ Tauri IPC
       │
后端 Rust
  commands/ai.rs
  services/ai/
    ├── mod.rs          # 提供商管理与分发
    ├── providers.rs    # 各厂商预设
    ├── chat.rs         # 会话上下文与补全
    └── client.rs       # HTTP 请求封装
  db/ai_chat_sessions, ai_chat_messages
```

---

## 2. 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| API 调用位置 | Rust 后端 | 保护 API Key，统一流式处理 |
| 厂商预设 | 国际 + 国内主流 | 覆盖 OpenAI / Anthropic / Gemini / Azure / DeepSeek / Moonshot / Qwen / Zhipu / MiniMax / Baichuan |
| 协议格式 | OpenAI-compatible Chat Completions 为主 | 国内多数厂商兼容此格式，降低适配成本 |
| 上下文注入 | 会话可关联当前邮件，prompt 自动注入邮件摘要 | 提升总结/回复建议质量 |
| 聊天记录 | SQLite 持久化 | 本地可控，重启不丢 |
| 会话入口 | Sidebar + MailViewer 快捷按钮 + Command Palette | 与现有 UI 体系一致 |
| 流式响应 | 首期先支持非流式完整返回，接口预留 stream 字段 | 快速落地，后续可升级 SSE/事件 |

---

## 3. 数据模型

### 3.1 AI 提供商（与翻译共享）

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
```

### 3.2 厂商预设

```rust
// src-tauri/src/services/ai/providers.rs
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
            base_url: "".to_string(),
            default_model: "".to_string(),
        },
    }
}
```

### 3.3 会话与消息表

```sql
-- src-tauri/src/db/migrations/00N_add_ai_chat.sql
CREATE TABLE IF NOT EXISTS ai_chat_sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    provider_id TEXT NOT NULL,
    model TEXT NOT NULL,
    context_mail_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_chat_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_messages_session
ON ai_chat_messages(session_id, created_at);
```

### 3.4 前端类型

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
  title: string;
  providerId: string;
  model: string;
  contextMailId?: string;
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

---

## 4. 接口设计

```rust
// src-tauri/src/commands/ai.rs
use tauri::State;
use crate::state::AppState;
use crate::models::error::ErrorPayload;
use crate::models::ai::{AiProvider, AiProviderSummary, AiChatSession, AiChatMessage};

#[tauri::command]
pub async fn list_ai_providers(
    state: State<'_, AppState>,
) -> Result<Vec<AiProviderSummary>, ErrorPayload>;

#[tauri::command]
pub async fn upsert_ai_provider(
    provider: AiProvider,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload>;

#[tauri::command]
pub async fn delete_ai_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload>;

#[tauri::command]
pub async fn test_ai_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload>;

#[tauri::command]
pub async fn create_chat_session(
    provider_id: String,
    context_mail_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<AiChatSession, ErrorPayload>;

#[tauri::command]
pub async fn send_chat_message(
    session_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<AiChatMessage, ErrorPayload>;

#[tauri::command]
pub async fn list_chat_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<AiChatSession>, ErrorPayload>;

#[tauri::command]
pub async fn get_chat_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AiChatMessage>, ErrorPayload>;

#[tauri::command]
pub async fn delete_chat_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload>;
```

---

## 5. 后端实现

### 5.1 AI 服务核心

```rust
// src-tauri/src/services/ai/mod.rs
use crate::db::Database;
use crate::models::ai::{AiProvider, AiProviderKind, AiProviderSummary};
use crate::error::AeroError;

pub struct AiService {
    db: std::sync::Arc<Database>,
}

impl AiService {
    pub fn list_providers(&self,
    ) -> Result<Vec<AiProviderSummary>, AeroError> {
        let providers = self.db.list_ai_providers()?;
        Ok(providers.into_iter().map(|p| AiProviderSummary {
            id: p.id,
            name: p.name,
            kind: format!("{:?}", p.kind),
            model: p.model,
        }).collect())
    }

    pub async fn complete(
        &self,
        provider_id: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String, AeroError> {
        let provider = self.db.get_ai_provider(provider_id)?;
        client::chat_completion(&provider,
            messages,
            provider.max_tokens.unwrap_or(2048),
        ).await
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
```

### 5.2 OpenAI-compatible 请求封装

```rust
// src-tauri/src/services/ai/client.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::models::ai::AiProvider;
use crate::services::ai::ChatMessage;
use crate::error::AeroError;

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
    let api_key = crate::crypto::decrypt(&provider.api_key_encrypted)?;
    let base_url = provider.base_url.as_deref().unwrap_or("");
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let payload_messages: Vec<ChatMessagePayload> = messages.into_iter()
        .map(|m| ChatMessagePayload { role: m.role, content: m.content })
        .collect();

    let body = ChatCompletionRequest {
        model: provider.model.clone(),
        messages: payload_messages,
        max_tokens,
    };

    let client = Client::new();
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    if !res.status().is_success() {
        return Err(AeroError::AiApiError(res.text().await.unwrap_or_default()));
    }

    let data: ChatCompletionResponse = res.json().await
        .map_err(|e| AeroError::AiApiError(e.to_string()))?;

    data.choices.into_iter().next()
        .map(|c| c.message.content)
        .ok_or_else(|| AeroError::AiApiError("empty response".to_string()))
}
```

### 5.3 上下文注入

```rust
// src-tauri/src/services/ai/chat.rs
use crate::db::Database;
use crate::services::ai::{AiService, ChatMessage};
use crate::error::AeroError;

pub async fn send_message(
    ai: &AiService,
    db: &Database,
    session_id: &str,
    user_content: &str,
) -> Result<String, AeroError> {
    let session = db.get_chat_session(session_id)?;
    let history = db.get_chat_messages(session_id)?;

    let mut messages: Vec<ChatMessage> = Vec::new();

    messages.push(ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful email assistant. Be concise.".to_string(),
    });

    if let Some(mail_id) = session.context_mail_id {
        let mail = db.get_mail(&mail_id)?;
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!(
                "[Current Email Context]\nSubject: {}\nFrom: {}\n\n{}",
                mail.subject,
                mail.from_address,
                mail.body_text.as_deref().unwrap_or("")
            ),
        });
    }

    for msg in history {
        messages.push(ChatMessage {
            role: msg.role,
            content: msg.content,
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_content.to_string(),
    });

    let reply = ai.complete(&session.provider_id, messages).await?;

    db.insert_chat_message(session_id, "user", user_content)?;
    db.insert_chat_message(session_id, "assistant", &reply)?;
    db.update_chat_session_timestamp(session_id)?;

    Ok(reply)
}
```

### 5.4 错误类型扩展

```rust
// src-tauri/src/error.rs
pub enum AeroError {
    // ... existing variants
    AiProviderNotFound,
    AiApiError(String),
    AiRateLimited,
    AiContextMailNotFound,
}

impl AeroError {
    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            // ... existing mappings
            AeroError::AiProviderNotFound => ErrorPayload {
                code: "AI_PROVIDER_NOT_FOUND".to_string(),
                args: vec![],
            },
            AeroError::AiApiError(msg) => ErrorPayload {
                code: "AI_API_ERROR".to_string(),
                args: vec![msg.clone()],
            },
            AeroError::AiRateLimited => ErrorPayload {
                code: "AI_RATE_LIMITED".to_string(),
                args: vec![],
            },
            AeroError::AiContextMailNotFound => ErrorPayload {
                code: "AI_CONTEXT_MAIL_NOT_FOUND".to_string(),
                args: vec![],
            },
        }
    }
}
```

---

## 6. 前端实现

### 6.1 useAiChat composable

```typescript
// src/composables/useAiChat.ts
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AiChatSession, AiChatMessage, AiProviderSummary } from '@/types/ai';

export function useAiChat() {
  const sessions = ref<AiChatSession[]>([]);
  const messages = ref<AiChatMessage[]>([]);
  const isLoading = ref(false);

  async function loadSessions() {
    sessions.value = await invoke<AiChatSession[]>('list_chat_sessions');
  }

  async function createSession(providerId: string, contextMailId?: string) {
    const session = await invoke<AiChatSession>('create_chat_session', {
      providerId,
      contextMailId,
    });
    sessions.value.unshift(session);
    return session;
  }

  async function sendMessage(sessionId: string, content: string) {
    isLoading.value = true;
    try {
      const reply = await invoke<AiChatMessage>('send_chat_message', {
        sessionId,
        content,
      });
      messages.value.push(reply);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMessages(sessionId: string) {
    messages.value = await invoke<AiChatMessage[]>('get_chat_messages', { sessionId });
  }

  return {
    sessions,
    messages,
    isLoading,
    loadSessions,
    createSession,
    sendMessage,
    loadMessages,
  };
}
```

### 6.2 AiAssistantPanel 组件

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAiChat } from '@/composables/useAiChat';
import AiMessageList from '@/components/AiMessageList.vue';

const { t } = useI18n();
const { sessions, messages, isLoading, loadSessions, sendMessage, loadMessages } = useAiChat();
const input = ref('');
const activeSessionId = ref<string | null>(null);

onMounted(() => {
  loadSessions();
});

async function handleSend() {
  if (!input.value.trim() || !activeSessionId.value) return;
  await sendMessage(activeSessionId.value, input.value);
  input.value = '';
}
</script>

<template>
  <div class="flex flex-col h-full w-[360px] bg-panel border-l border-border">
    <div class="h-12 flex items-center px-4 border-b border-border font-medium">
      {{ t('aiAssistant.title') }}
    </div>
    <AiMessageList :messages="messages" class="flex-1 overflow-y-auto p-3" />
    <div class="p-3 border-t border-border">
      <textarea
        v-model="input"
        class="w-full h-20 bg-card rounded-md p-2 text-sm resize-none"
        :placeholder="t('aiAssistant.inputPlaceholder')"
        @keydown.enter.prevent="handleSend"
      />
      <button
        class="mt-2 w-full h-8 bg-primary rounded-md text-sm"
        :disabled="isLoading"
        @click="handleSend"
      >
        {{ isLoading ? t('aiAssistant.thinking') : t('aiAssistant.send') }}
      </button>
    </div>
  </div>
</template>
```

---

## 7. UI 集成点

- **Sidebar**：新增「AI 助手」入口，点击展开右侧 360px 面板。
- **MailViewer Header**：新增「AI 总结」「AI 回复」「AI 提取待办」三个快捷按钮，点击后自动创建关联会话并发送第一条 prompt。
- **Command Palette**：新增「打开 AI 助手」「新建 AI 会话」命令。
- **SettingsView**：新增「AI 提供商」卡片，可添加/删除/测试厂商，选择默认模型。

---

## 8. 快捷操作 Prompt

| 操作 | Prompt（中文示例） |
|------|-------------------|
| 总结 | `请用 3 句话总结以下邮件。` |
| 生成回复 | `请根据这封邮件帮我写一封礼貌的回复草稿，保持简洁。` |
| 提取待办 | `请从邮件中提取所有待办事项，按优先级列出。` |

快捷操作时，系统自动追加 `[Current Email Context]` 到 system 消息中。

---

## 9. 安全与隐私

- API Key 使用 AES-256-GCM 加密后存入 SQLite。
- 聊天记录仅本地持久化，不上传。
- 发送给 LLM 的内容不包含邮件完整 HTML，仅注入去除标签后的纯文本摘要。
- 用户删除 AI 提供商时，提示是否同步删除相关会话。

---

## 10. 错误处理

沿用 i18n 错误码体系，新增：

| 错误码 | 含义 |
|--------|------|
| `AI_PROVIDER_NOT_FOUND` | 指定 AI 提供商不存在 |
| `AI_API_ERROR` | LLM API 返回错误 |
| `AI_RATE_LIMITED` | 触发速率限制 |
| `AI_CONTEXT_MAIL_NOT_FOUND` | 关联邮件已删除 |

---

## 11. 测试策略

| 测试项 | 方法 |
|--------|------|
| 预设填充 | 选择 DeepSeek/OpenAI 后 base URL / model 自动正确 |
| 密钥加密 | 数据库中 `api_key_encrypted` 不可读 |
| 上下文注入 | 关联邮件的会话，prompt 包含邮件主题/正文 |
| 持久化 | 重启后会话和消息可恢复 |
| 快捷操作 | 点击「总结」后返回有效总结文本 |
| 错误码 | 无效 Key / 无网络返回对应 ErrorPayload |

---

## 12. 依赖变更

### 后端

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
futures = "0.3"
```

### 前端

无需新增依赖。

---

## 13. 影响范围

- 新增数据库表 `ai_chat_sessions`、`ai_chat_messages`。
- 新增 `src-tauri/src/services/ai/` 目录。
- 新增 `src-tauri/src/commands/ai.rs`。
- 修改 `src-tauri/src/error.rs`，新增 AI 相关错误码。
- 修改 `src/components/AppSidebar.vue`，新增 AI 助手入口。
- 修改 `src/components/MailViewer.vue` Header，新增 AI 快捷按钮。
- `AiProvider` 模型被翻译模块复用。
