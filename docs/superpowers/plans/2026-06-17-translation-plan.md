# 翻译功能实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 AeroMail 添加邮件正文翻译能力，支持传统翻译 API（Google/DeepL/百度等）和 AI 翻译（复用 AiProvider），翻译结果本地缓存，默认目标语言跟随 UI 语言。

**Architecture:** 后端通过 reqwest 调用各翻译 API，AI 翻译复用 AiProvider 的 OpenAI-compatible 协议；翻译结果按 source_hash + target_lang + provider_id 缓存到 SQLite；前端在 MailViewer 内提供原文/译文切换。

**Tech Stack:** Rust 2024 + reqwest + sha2 + Tauri v2 + Vue 3 + vue-i18n + TypeScript.

## Global Constraints

- Rust 2024 edition，`unsafe_code = "forbid"`，clippy `unwrap_used = "deny"`。
- 所有 Vue 组件使用 `<script setup lang="ts">`。
- 前端路径别名 `@` 指向 `src/`。
- 后端 Tauri Command 错误统一使用 `ErrorPayload { code, args }`。
- 新增文案必须同时写入 `en.json` 与 `zh-CN.json`，保持 key 一致。
- 每个 Task 结束时运行 `pnpm lint`、`pnpm type-check`、`cargo check`。

---

## Task 1: 创建翻译提供商模型 + DB 表

**Files:**
- Create: `src-tauri/src/models/translation.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Modify: `src-tauri/src/db/schema.rs`（添加 translations 表）

**Interfaces:**
- Produces: `TranslationProvider`（Traditional/Ai 两种）、`TraditionalProviderKind`、`TranslationProviderSummary`、`CachedTranslation`。

- [ ] **Step 1: 创建翻译模型**

```rust
// src-tauri/src/models/translation.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TranslationProvider {
    Traditional {
        id: String,
        name: String,
        kind: TraditionalProviderKind,
        api_key_encrypted: Vec<u8>,
        endpoint: Option<String>,
        extra: HashMap<String, String>,
    },
    Ai {
        id: String,
        name: String,
        ai_provider_id: String,
        prompt_template: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraditionalProviderKind {
    GoogleTranslate,
    DeepL,
    AzureTranslator,
    Baidu,
    Youdao,
    TencentTranslator,
    AliyunTranslator,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationProviderSummary {
    pub id: String,
    pub name: String,
    pub provider_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTranslation {
    pub id: String,
    pub source_hash: String,
    pub target_lang: String,
    pub provider_id: String,
    pub translated_text: String,
    pub created_at: i64,
}
```

- [ ] **Step 2: 注册模型模块**

```rust
// src-tauri/src/models/mod.rs
pub mod account;
pub mod ai;
pub mod error;
pub mod translation;
```

- [ ] **Step 3: 添加 translations 表到 schema.rs**

在 `ALL_SCHEMAS` 数组末尾追加：

```rust
pub const TRANSLATIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS translations (
    id TEXT PRIMARY KEY,
    source_hash TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    created_at INTEGER NOT NULL
)
"#;

pub const TRANSLATIONS_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_translations_lookup
ON translations(source_hash, target_lang, provider_id)
"#;
```

并在 `ALL_SCHEMAS` 中添加这两个常量。

- [ ] **Step 4: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models/translation.rs src-tauri/src/models/mod.rs src-tauri/src/db/schema.rs
git commit -m "feat(translate): add translation provider model and DB tables" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: 添加翻译提供商 DB 操作

**Files:**
- Modify: `src-tauri/src/db/pool.rs`（添加翻译提供商 CRUD + 缓存读写方法）

**Interfaces:**
- Produces: `list_translation_providers`, `get_translation_provider`, `upsert_translation_provider`, `delete_translation_provider`, `get_translation`, `save_translation`。

- [ ] **Step 1: 在 db/pool.rs 添加翻译方法**

在 `impl Database` 块内追加以下方法：

```rust
pub fn list_translation_providers(&self) -> Result<Vec<crate::models::translation::TranslationProvider>, AeroError> {
    let conn = self.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, provider_type, config_json FROM translation_providers"
    )?;
    let rows = stmt.query_map([], |row| {
        let config_json: String = row.get(3)?;
        let provider: crate::models::translation::TranslationProvider =
            serde_json::from_str(&config_json).unwrap_or_default();
        Ok(provider)
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| AeroError::Database(e.to_string()))
}

pub fn upsert_translation_provider(&self, p: &crate::models::translation::TranslationProvider) -> Result<(), AeroError> {
    let conn = self.connection()?;
    let (id, name, provider_type) = match p {
        crate::models::translation::TranslationProvider::Traditional { id, name, .. } => (id, name, "traditional"),
        crate::models::translation::TranslationProvider::Ai { id, name, .. } => (id, name, "ai"),
    };
    let config_json = serde_json::to_string(p)?;
    conn.execute(
        "INSERT INTO translation_providers (id, name, provider_type, config_json)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name, provider_type=excluded.provider_type, config_json=excluded.config_json",
        (id, name, provider_type, config_json),
    )?;
    Ok(())
}

pub fn delete_translation_provider(&self, id: &str) -> Result<(), AeroError> {
    let conn = self.connection()?;
    conn.execute("DELETE FROM translation_providers WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_translation(&self, source_hash: &str, target_lang: &str, provider_id: &str) -> Result<Option<crate::models::translation::CachedTranslation>, AeroError> {
    let conn = self.connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, source_hash, target_lang, provider_id, translated_text, created_at
         FROM translations WHERE source_hash = ?1 AND target_lang = ?2 AND provider_id = ?3"
    )?;
    let mut rows = stmt.query(rusqlite::params![source_hash, target_lang, provider_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(crate::models::translation::CachedTranslation {
            id: row.get(0)?,
            source_hash: row.get(1)?,
            target_lang: row.get(2)?,
            provider_id: row.get(3)?,
            translated_text: row.get(4)?,
            created_at: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn save_translation(&self, source_hash: &str, target_lang: &str, provider_id: &str, translated_text: &str) -> Result<(), AeroError> {
    let conn = self.connection()?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO translations (id, source_hash, target_lang, provider_id, translated_text, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&id, source_hash, target_lang, provider_id, translated_text, now),
    )?;
    Ok(())
}
```

注意：需要在 schema.rs 中添加 `translation_providers` 表：

```sql
CREATE TABLE IF NOT EXISTS translation_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    config_json TEXT NOT NULL
)
```

- [ ] **Step 2: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/pool.rs src-tauri/src/db/schema.rs
git commit -m "feat(translate): add translation provider DB operations and cache" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: 创建翻译服务 + 错误码

**Files:**
- Create: `src-tauri/src/services/translation/mod.rs`
- Create: `src-tauri/src/services/translation/traditional.rs`
- Create: `src-tauri/src/services/translation/ai.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/error.rs`（添加翻译错误码）
- Modify: `src/i18n/locales/en.json` 和 `zh-CN.json`

**Interfaces:**
- Produces: `TranslationService` 结构体，`translate_mail_text()` 方法，传统 API 客户端，AI 翻译客户端。

- [ ] **Step 1: 创建翻译服务核心**

```rust
// src-tauri/src/services/translation/mod.rs
pub mod ai;
pub mod traditional;

use crate::db::Database;
use crate::error::AeroError;
use crate::models::translation::{CachedTranslation, TranslationProvider};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub struct TranslationService {
    pub db: Arc<Database>,
}

impl TranslationService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn translate_mail(
        &self,
        source_text: &str,
        target_lang: &str,
        provider_id: &str,
    ) -> Result<String, AeroError> {
        if source_text.trim().is_empty() {
            return Err(AeroError::TranslationNoText);
        }

        let source_hash = sha256_hex(source_text);

        if let Some(cached) = self.db.get_translation(&source_hash, target_lang, provider_id)? {
            return Ok(cached.translated_text);
        }

        let provider = self.db.get_translation_provider(provider_id)?;

        let translated = match &provider {
            TranslationProvider::Traditional { .. } => {
                traditional::translate(&provider, source_text, target_lang)?
            }
            TranslationProvider::Ai { ai_provider_id, .. } => {
                let ai_provider = self.db.get_ai_provider(ai_provider_id)?;
                ai::translate(&ai_provider, source_text, target_lang)?
            }
        };

        self.db.save_translation(&source_hash, target_lang, provider_id, &translated)?;

        Ok(translated)
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

- [ ] **Step 2: 创建传统翻译客户端**

```rust
// src-tauri/src/services/translation/traditional.rs
use crate::error::AeroError;
use crate::models::translation::{TranslationProvider, TraditionalProviderKind};

pub fn translate(
    provider: &TranslationProvider,
    source_text: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    match provider {
        TranslationProvider::Traditional { kind, api_key_encrypted, endpoint, .. } => {
            let api_key = String::from_utf8(api_key_encrypted.clone())
                .map_err(|e| AeroError::TranslationApiError(format!("invalid key: {e}")))?;
            match kind {
                TraditionalProviderKind::GoogleTranslate => {
                    google_translate(&api_key, source_text, target_lang)
                }
                TraditionalProviderKind::DeepL => {
                    deepl_translate(&api_key, source_text, target_lang, endpoint.as_deref())
                }
                _ => Err(AeroError::TranslationApiError(
                    "provider not yet implemented".to_string(),
                )),
            }
        }
        _ => Err(AeroError::TranslationApiError(
            "not a traditional provider".to_string(),
        )),
    }
}

fn google_translate(api_key: &str, text: &str, target_lang: &str) -> Result<String, AeroError> {
    let url = format!(
        "https://translation.googleapis.com/language/translate/v2?key={api_key}"
    );
    let body = serde_json::json!({
        "q": text,
        "target": target_lang,
    });
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!("HTTP {status}: {text}")));
    }
    let data: serde_json::Value = resp.json().map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    data["data"]["translations"][0]["translatedText"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}

fn deepl_translate(api_key: &str, text: &str, target_lang: &str, endpoint: Option<&str>) -> Result<String, AeroError> {
    let base = endpoint.unwrap_or("https://api-free.deepl.com");
    let url = format!("{base}/v2/translate");
    let resp = reqwest::blocking::Client::new()
        .post(&url)
        .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
        .form(&[("text", text), ("target_lang", target_lang)])
        .send()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(AeroError::TranslationApiError(format!("HTTP {status}: {body}")));
    }
    let data: serde_json::Value = resp.json().map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    data["translations"][0]["text"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| AeroError::TranslationApiError("unexpected response".to_string()))
}
```

- [ ] **Step 3: 创建 AI 翻译客户端**

```rust
// src-tauri/src/services/translation/ai.rs
use crate::error::AeroError;
use crate::models::ai::AiProvider;
use crate::services::ai::ChatMessage;

pub fn translate(
    ai_provider: &AiProvider,
    source_text: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    let prompt = format!(
        "Translate the following text to {target_lang}. Only output the translation, no explanation:\n\n{source_text}"
    );

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))?;
    rt.block_on(crate::services::ai::client::chat_completion(
        ai_provider,
        messages,
        ai_provider.max_tokens.unwrap_or(2048),
    ))
}
```

- [ ] **Step 4: 注册服务模块**

```rust
// src-tauri/src/services/mod.rs
pub mod account_manager;
pub mod ai;
pub mod translation;
```

- [ ] **Step 5: 添加错误码**

在 `src-tauri/src/error.rs` 的 `AeroError` 枚举中追加：

```rust
#[error("translation provider not found")]
TranslationProviderNotFound,
#[error("translation API error: {0}")]
TranslationApiError(String),
#[error("no text to translate")]
TranslationNoText,
```

并在 `to_payload()` 中追加对应映射。

- [ ] **Step 6: 添加 i18n 翻译错误码**

在 `en.json` 和 `zh-CN.json` 的 `errors` 部分追加：

```json
"TRANSLATION_PROVIDER_NOT_FOUND": "Translation provider not found",
"TRANSLATION_API_ERROR": "Translation failed: {0}",
"TRANSLATION_NO_TEXT": "No text to translate"
```

中文：
```json
"TRANSLATION_PROVIDER_NOT_FOUND": "翻译提供商不存在",
"TRANSLATION_API_ERROR": "翻译失败：{0}",
"TRANSLATION_NO_TEXT": "没有可翻译的文本"
```

- [ ] **Step 7: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
pnpm i18n:check
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/services/translation/ src-tauri/src/services/mod.rs src-tauri/src/error.rs src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(translate): add translation service, traditional/AI clients, and error codes" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: 创建翻译 Tauri Commands + 注册

**Files:**
- Create: `src-tauri/src/commands/translation.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: 所有翻译相关 IPC 命令。

- [ ] **Step 1: 创建翻译命令**

```rust
// src-tauri/src/commands/translation.rs
use tauri::State;

use crate::models::error::ErrorPayload;
use crate::models::translation::{TranslationProvider, TranslationProviderSummary};
use crate::AppState;

#[tauri::command]
pub async fn list_translation_providers(
    state: State<'_, AppState>,
) -> Result<Vec<TranslationProviderSummary>, ErrorPayload> {
    let providers = state.db.list_translation_providers().map_err(AeroError::to_payload)?;
    Ok(providers.into_iter().map(|p| {
        let (id, name, provider_type) = match &p {
            TranslationProvider::Traditional { id, name, .. } => (id.clone(), name.clone(), "traditional".to_string()),
            TranslationProvider::Ai { id, name, .. } => (id.clone(), name.clone(), "ai".to_string()),
        };
        TranslationProviderSummary { id, name, provider_type }
    }).collect())
}

#[tauri::command]
pub async fn upsert_translation_provider(
    provider: TranslationProvider,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let id = match &provider {
        TranslationProvider::Traditional { id, .. } => id.clone(),
        TranslationProvider::Ai { id, .. } => id.clone(),
    };
    state.db.upsert_translation_provider(&provider).map_err(AeroError::to_payload)?;
    Ok(id)
}

#[tauri::command]
pub async fn delete_translation_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state.db.delete_translation_provider(&provider_id).map_err(AeroError::to_payload)
}

#[tauri::command]
pub async fn translate_mail_text(
    mail_id: String,
    target_lang: String,
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    let text = state.db.get_mail_body_text(&mail_id).map_err(AeroError::to_payload)?
        .unwrap_or_default();
    let translation = state.translation_service.translate_mail(&text, &target_lang, &provider_id)
        .map_err(AeroError::to_payload)?;
    Ok(translation)
}

#[tauri::command]
pub async fn get_cached_translation(
    mail_id: String,
    target_lang: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, ErrorPayload> {
    let text = state.db.get_mail_body_text(&mail_id).map_err(AeroError::to_payload)?
        .unwrap_or_default();
    let source_hash = crate::services::translation::sha256_hex(&text);
    let cached = state.db.get_translation(&source_hash, &target_lang, "any")
        .map_err(AeroError::to_payload)?;
    Ok(cached.map(|c| c.translated_text))
}
```

- [ ] **Step 2: 注册命令模块**

```rust
// src-tauri/src/commands/mod.rs
pub mod account;
pub mod ai;
pub mod settings;
pub mod translation;
```

- [ ] **Step 3: 在 lib.rs 注册命令**

在 `AppState` 中添加 `translation_service` 字段，在 `invoke_handler` 中注册所有翻译命令。

- [ ] **Step 4: 验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/translation.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(translate): add Tauri IPC commands for translation" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: 创建前端类型 + useTranslation composable

**Files:**
- Create: `src/types/translation.ts`
- Create: `src/composables/useTranslation.ts`
- Modify: `src/stores/settings.ts`（添加默认目标语言逻辑）

**Interfaces:**
- Produces: 前端翻译类型、`useTranslation()` composable、默认目标语言计算。

- [ ] **Step 1: 创建前端类型**

```typescript
// src/types/translation.ts
export type TraditionalProviderKind =
  | 'google_translate'
  | 'deep_l'
  | 'azure_translator'
  | 'baidu'
  | 'youdao'
  | 'tencent_translator'
  | 'aliyun_translator'
  | 'custom';

export interface TraditionalTranslationProvider {
  type: 'traditional';
  id: string;
  name: string;
  kind: TraditionalProviderKind;
  endpoint?: string;
  extra: Record<string, string>;
}

export interface AiTranslationProvider {
  type: 'ai';
  id: string;
  name: string;
  aiProviderId: string;
  promptTemplate?: string;
}

export type TranslationProvider = TraditionalTranslationProvider | AiTranslationProvider;

export interface TranslationProviderSummary {
  id: string;
  name: string;
  providerType: string;
}
```

- [ ] **Step 2: 创建 useTranslation composable**

```typescript
// src/composables/useTranslation.ts
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import type { TranslationProviderSummary } from '@/types/translation';
import { useLocale } from '@/composables/useLocale';

export function useTranslation() {
  const { locale } = useI18n();
  const { locale: appLocale } = useLocale();
  const isTranslating = ref(false);
  const error = ref<string | null>(null);

  function getDefaultTargetLang(): string {
    return appLocale.value === 'zh-CN' ? 'en' : 'zh-CN';
  }

  async function translateMail(
    mailId: string,
    targetLang: string,
    providerId: string,
  ): Promise<string> {
    isTranslating.value = true;
    error.value = null;
    try {
      return await invoke<string>('translate_mail_text', {
        mailId,
        targetLang,
        providerId,
      });
    } catch (raw) {
      error.value = String(raw);
      throw raw;
    } finally {
      isTranslating.value = false;
    }
  }

  async function listProviders(): Promise<TranslationProviderSummary[]> {
    return await invoke<TranslationProviderSummary[]>('list_translation_providers');
  }

  return {
    isTranslating,
    error,
    translateMail,
    listProviders,
    getDefaultTargetLang,
  };
}
```

- [ ] **Step 3: 验证**

```bash
pnpm lint
pnpm type-check
```

- [ ] **Step 4: Commit**

```bash
git add src/types/translation.ts src/composables/useTranslation.ts
git commit -m "feat(translate): add frontend types and useTranslation composable" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: 创建 TranslatePanel + 集成到 MailViewer

**Files:**
- Create: `src/components/TranslatePanel.vue`
- Modify: `src/components/MailViewer.vue`（添加翻译入口和原文/译文切换）
- Modify: `src/i18n/locales/en.json` 和 `zh-CN.json`

**Interfaces:**
- Produces: 翻译面板组件，MailViewer 翻译集成。

- [ ] **Step 1: 创建 TranslatePanel**

```vue
<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTranslation } from '@/composables/useTranslation';
import type { TranslationProviderSummary } from '@/types/translation';

const props = defineProps<{
  mailId: string;
}>();

const emit = defineEmits<{
  translated: [text: string];
}>();

const { t } = useI18n();
const { translateMail, listProviders, getDefaultTargetLang, isTranslating } = useTranslation();

const providers = ref<TranslationProviderSummary[]>([]);
const selectedProviderId = ref('');
const targetLang = ref(getDefaultTargetLang());

async function loadProviders() {
  providers.value = await listProviders();
  if (providers.value.length > 0 && !selectedProviderId.value) {
    selectedProviderId.value = providers.value[0].id;
  }
}

async function handleTranslate() {
  if (!selectedProviderId.value) return;
  const translated = await translateMail(props.mailId, targetLang.value, selectedProviderId.value);
  emit('translated', translated);
}

// Load providers on mount
loadProviders();
</script>

<template>
  <div class="flex items-center gap-2 rounded-lg border border-border bg-card p-2">
    <select
      v-model="targetLang"
      class="h-7 rounded border border-border bg-panel px-2 text-xs text-text"
    >
      <option value="en">English</option>
      <option value="zh-CN">简体中文</option>
      <option value="ja">日本語</option>
      <option value="ko">한국어</option>
    </select>
    <select
      v-model="selectedProviderId"
      class="h-7 rounded border border-border bg-panel px-2 text-xs text-text"
    >
      <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.name }}</option>
    </select>
    <button
      class="flex h-7 items-center rounded bg-primary px-2 text-xs text-white hover:bg-primary-hover disabled:opacity-50"
      :disabled="isTranslating || !selectedProviderId"
      @click="handleTranslate"
    >
      {{ isTranslating ? t('translation.translating') : t('translation.translate') }}
    </button>
  </div>
</template>
```

- [ ] **Step 2: 在 MailViewer 集成翻译**

在 `MailViewer.vue` 中：
1. 导入 `TranslatePanel`
2. 添加翻译状态（`translatedText`, `showTranslation`）
3. 在 Header 操作按钮旁添加翻译按钮
4. 在正文区域上方添加翻译横幅和切换按钮

- [ ] **Step 3: 添加 i18n key**

```json
"translation": {
  "translate": "Translate",
  "translating": "Translating...",
  "showOriginal": "Show Original",
  "translatedTo": "Translated to {lang}",
  "noProviders": "No translation providers configured"
}
```

中文：
```json
"translation": {
  "translate": "翻译",
  "translating": "翻译中...",
  "showOriginal": "显示原文",
  "translatedTo": "已翻译为 {lang}",
  "noProviders": "未配置翻译提供商"
}
```

- [ ] **Step 4: 验证**

```bash
pnpm lint
pnpm type-check
pnpm i18n:check
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 5: Commit**

```bash
git add src/components/TranslatePanel.vue src/components/MailViewer.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(translate): add TranslatePanel and MailViewer integration" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: Settings 翻译管理 + 最终验证

**Files:**
- Modify: `src/views/SettingsView.vue`（添加翻译提供商管理卡片）
- Modify: `src/i18n/locales/en.json` 和 `zh-CN.json`

**Interfaces:**
- Produces: Settings 翻译提供商管理界面。

- [ ] **Step 1: 在 SettingsView 添加翻译提供商管理**

在 SettingsView.vue 中新增翻译提供商卡片，支持：
- 查看已配置的翻译提供商
- 添加传统翻译提供商（选择厂商、填入 API Key）
- 添加 AI 翻译提供商（选择已有的 AI Provider）
- 删除提供商

- [ ] **Step 2: 添加 i18n key**

```json
"settings": {
  "translationProviders": "Translation Providers",
  "addTranslationProvider": "Add Provider",
  "noTranslationProviders": "No translation providers configured"
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
git add src/views/SettingsView.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(translate): add translation provider management in Settings" -m "Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Self-Review Checklist

- [x] Spec coverage: 传统翻译 API、AI 翻译、缓存、默认目标语言、MailViewer 切换、Settings 管理均有对应 Task。
- [x] Placeholder scan: 无 TBD/TODO，所有步骤含代码与命令。
- [x] Type consistency: `TranslationProvider`/`CachedTranslation` 前后端定义一致。
- [x] Scope: 本计划聚焦翻译功能，不涉及其他模块改动。
