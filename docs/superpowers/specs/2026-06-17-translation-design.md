# AeroMail 邮件翻译设计规范

> **范围**：邮件正文翻译；支持传统翻译 API + AI 翻译；翻译结果本地缓存；默认目标语言跟随 UI 语言。
> **前置依赖**：复用「AI 助手」模块定义的 `AiProvider` 配置；错误处理复用 i18n 规范。

---

## 1. 总体架构

```text
前端 Vue 3
  MailViewer.vue
  TranslatePanel.vue
  useTranslation.ts
       │
       │ Tauri IPC
       │
后端 Rust
  commands/translation.rs
  services/translation/
    ├── mod.rs          # 调度 + 缓存
    ├── traditional.rs  # 传统翻译 API 客户端
    └── ai.rs           # AI 翻译客户端（复用 AiProvider）
  db/
    └── translations    # SQLite 缓存表
```

---

## 2. 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| API 调用位置 | Rust 后端 | 保护 API Key，规避 CORS |
| AI 翻译密钥来源 | 复用 AI 助手的 `AiProvider` | 避免重复配置 |
| 传统 API 支持 | Google / DeepL / Azure / 百度 / 有道 / 腾讯 / 阿里 / Custom | 覆盖国内外主流厂商 |
| 缓存策略 | SQLite 缓存，按 source_hash + target_lang + provider_id | 减少 API 调用、支持离线回看 |
| 默认目标语言 | UI 中文默认英，UI 英文默认中；可设置覆盖 | 符合直觉 |
| 翻译展示 | 在 MailViewer 内切换「原文 / 译文」 | 不破坏阅读流 |
| 错误处理 | 返回 `ErrorPayload { code, args }`，前端按 i18n 映射 | 与 i18n 规范一致 |

---

## 3. 数据模型

### 3.1 共享 AI 提供商（由 AI 助手模块定义）

```rust
// src-tauri/src/models/ai.rs
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
    DeepSeek,
    Moonshot,
    Qwen,
    CustomOpenAICompatible,
}
```

### 3.2 翻译提供商配置

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
    pub is_ready: bool,
}
```

### 3.3 缓存表结构

```sql
-- src-tauri/src/db/migrations/00N_add_translations.sql
CREATE TABLE IF NOT EXISTS translations (
    id TEXT PRIMARY KEY,
    source_hash TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_translations_lookup
ON translations(source_hash, target_lang, provider_id);
```

---

## 4. 接口设计

### 4.1 Tauri Commands

```rust
// src-tauri/src/commands/translation.rs
use tauri::State;
use crate::state::AppState;
use crate::models::error::ErrorPayload;
use crate::models::translation::{TranslationProvider, TranslationProviderSummary};

#[tauri::command]
pub async fn list_translation_providers(
    state: State<'_, AppState>,
) -> Result<Vec<TranslationProviderSummary>, ErrorPayload> {
    state.translation_service.list_providers().await.map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn upsert_translation_provider(
    provider: TranslationProvider,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    state.translation_service.upsert_provider(provider).await.map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn delete_translation_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    state.translation_service.delete_provider(&provider_id).await.map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn test_translation_provider(
    provider_id: String,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    state.translation_service.test_provider(&provider_id).await.map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn translate_mail_text(
    mail_id: String,
    target_lang: String,
    provider_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, ErrorPayload> {
    state.translation_service
        .translate_mail(&mail_id, &target_lang, provider_id.as_deref())
        .await
        .map_err(|e| e.to_payload())
}

#[tauri::command]
pub async fn get_cached_translation(
    mail_id: String,
    target_lang: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, ErrorPayload> {
    state.translation_service
        .get_cached(&mail_id, &target_lang)
        .await
        .map_err(|e| e.to_payload())
}
```

### 4.2 前端类型

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

export interface TraditionalProvider {
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

export type TranslationProvider = TraditionalProvider | AiTranslationProvider;

export interface TranslationProviderSummary {
  id: string;
  name: string;
  providerType: string;
  isReady: boolean;
}
```

---

## 5. 后端实现

### 5.1 翻译服务调度器

```rust
// src-tauri/src/services/translation/mod.rs
use crate::db::Database;
use crate::models::translation::{TranslationProvider, CachedTranslation};
use crate::error::AeroError;

pub struct TranslationService {
    db: std::sync::Arc<Database>,
}

impl TranslationService {
    pub async fn translate_mail(
        &self,
        mail_id: &str,
        target_lang: &str,
        provider_id: Option<&str>,
    ) -> Result<String, AeroError> {
        let source_text = self.db.get_mail_body_text(mail_id)?;
        if source_text.trim().is_empty() {
            return Err(AeroError::TranslationNoText);
        }

        let provider_id = provider_id
            .map(|s| s.to_string())
            .or_else(|| self.db.get_default_translation_provider())
            .ok_or(AeroError::TranslationProviderNotFound)?;

        let source_hash = sha256_hex(&source_text);
        if let Some(cached) = self.db.get_translation(&source_hash, target_lang, &provider_id)? {
            return Ok(cached.translated_text);
        }

        let provider = self.db.get_translation_provider(&provider_id)?;
        let translated = match provider {
            TranslationProvider::Traditional { .. } => {
                traditional::translate(&provider, &source_text, target_lang).await?
            }
            TranslationProvider::Ai { ai_provider_id, .. } => {
                let ai_provider = self.db.get_ai_provider(&ai_provider_id)?;
                ai::translate(&ai_provider, &provider, &source_text, target_lang).await?
            }
        };

        self.db.save_translation(&source_hash,
            target_lang,
            &provider_id,
            &translated,
        )?;

        Ok(translated)
    }
}
```

### 5.2 缓存读写示例

```rust
// src-tauri/src/db/translations.rs
use crate::models::translation::CachedTranslation;
use crate::error::AeroError;
use rusqlite::Connection;

pub fn get_translation(
    conn: &Connection,
    source_hash: &str,
    target_lang: &str,
    provider_id: &str,
) -> Result<Option<CachedTranslation>, AeroError> {
    let mut stmt = conn.prepare(
        "SELECT id, source_hash, target_lang, provider_id, translated_text, created_at
         FROM translations
         WHERE source_hash = ?1 AND target_lang = ?2 AND provider_id = ?3"
    )?;
    let mut rows = stmt.query(rusqlite::params![source_hash, target_lang, provider_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(CachedTranslation {
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
```

### 5.3 AI 翻译客户端示例

```rust
// src-tauri/src/services/translation/ai.rs
use crate::models::ai::AiProvider;
use crate::models::translation::TranslationProvider;
use crate::error::AeroError;

pub async fn translate(
    ai_provider: &AiProvider,
    translation_provider: &TranslationProvider,
    source: &str,
    target_lang: &str,
) -> Result<String, AeroError> {
    let prompt = match translation_provider {
        TranslationProvider::Ai { prompt_template: Some(t), .. } => {
            t.replace("{source}", source).replace("{target_lang}", target_lang)
        }
        _ => format!(
            "Translate the following text to {}. Only output the translation, no explanation:\n\n{}",
            target_lang, source
        ),
    };

    crate::services::ai::complete(ai_provider, &prompt).await
        .map_err(|e| AeroError::TranslationApiError(e.to_string()))
}
```

### 5.4 错误类型扩展

```rust
// src-tauri/src/error.rs
pub enum AeroError {
    // ... existing variants
    TranslationProviderNotFound,
    TranslationApiError(String),
    TranslationNoText,
    AiProviderNotConfigured,
}

impl AeroError {
    pub fn to_payload(&self) -> ErrorPayload {
        match self {
            // ... existing mappings
            AeroError::TranslationProviderNotFound => ErrorPayload {
                code: "TRANSLATION_PROVIDER_NOT_FOUND".to_string(),
                args: vec![],
            },
            AeroError::TranslationApiError(msg) => ErrorPayload {
                code: "TRANSLATION_API_ERROR".to_string(),
                args: vec![msg.clone()],
            },
            AeroError::TranslationNoText => ErrorPayload {
                code: "TRANSLATION_NO_TEXT".to_string(),
                args: vec![],
            },
            AeroError::AiProviderNotConfigured => ErrorPayload {
                code: "AI_PROVIDER_NOT_CONFIGURED".to_string(),
                args: vec![],
            },
        }
    }
}
```

---

## 6. 前端实现

### 6.1 useTranslation composable

```typescript
// src/composables/useTranslation.ts
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { TranslationProviderSummary } from '@/types/translation';

export function useTranslation() {
  const isTranslating = ref(false);
  const error = ref<string | null>(null);

  async function translateMail(
    mailId: string,
    targetLang: string,
    providerId?: string
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
  };
}
```

### 6.2 TranslatePanel 组件

```vue
<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTranslation } from '@/composables/useTranslation';
import { useSettingsStore } from '@/stores/settings';

const props = defineProps<{ mailId: string }>();

const { t } = useI18n();
const { translateMail, isTranslating } = useTranslation();
const settings = useSettingsStore();

const showTranslated = ref(false);
const translatedText = ref('');
const targetLang = ref(settings.getDefaultTargetLang());

async function handleTranslate() {
  translatedText.value = await translateMail(props.mailId, targetLang.value);
  showTranslated.value = true;
}

function showOriginal() {
  showTranslated.value = false;
}
</script>

<template>
  <div class="flex items-center gap-2">
    <button v-if="!showTranslated" @click="handleTranslate" :disabled="isTranslating">
      {{ isTranslating ? t('translation.translating') : t('translation.translate') }}
    </button>
    <div v-else class="flex items-center gap-2">
      <span>{{ t('translation.translatedTo', { lang: targetLang }) }}</span>
      <button @click="showOriginal">{{ t('translation.showOriginal') }}</button>
    </div>
  </div>
</template>
```

---

## 7. 默认目标语言

```typescript
// src/stores/settings.ts
export function useSettingsStore() {
  // ... existing set/get

  function getDefaultTargetLang(): string {
    const saved = get('translation.default_target_language');
    if (saved) return saved;

    const { locale } = useI18n();
    return locale.value === 'zh-CN' ? 'en' : 'zh-CN';
  }

  return { set, get, getDefaultTargetLang };
}
```

---

## 8. UI 集成点

- **MailViewer Header**：右侧操作按钮组新增「翻译」入口，点击展开目标语言/引擎选择浮层。
- **MailViewer Body**：译文与原文切换显示，顶部显示翻译来源横幅。
- **SettingsView**：新增「翻译」卡片，包含默认目标语言、默认引擎、缓存清理。

---

## 9. 安全与隐私

- API Key 使用项目统一的 AES-256-GCM 加密后存入 SQLite。
- 翻译请求全部走后端 `reqwest`，不暴露 Key 到前端。
- 缓存文本仅本地存储，不上传。
- 用户删除提供商时同步清理相关缓存。

---

## 10. 测试策略

| 测试项 | 方法 |
|--------|------|
| 缓存命中 | 同一文本二次翻译返回缓存结果，不触发网络 |
| 引擎切换 | Traditional 与 AI 引擎都能返回有效译文 |
| 默认目标语言 | UI 中文默认 en，UI 英文默认 zh-CN |
| Key 加密 | 数据库中 api_key_encrypted 字段不可读 |
| 离线回看 | 断网时 `get_cached_translation` 返回已有译文 |
| 错误码 | 无提供商/无文本/API 失败均返回对应 ErrorPayload |

---

## 11. 依赖变更

### 后端

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
sha2 = "0.10"
```

### 前端

无需新增依赖。

---

## 12. 影响范围

- 新增数据库表 `translations`。
- 新增 `src-tauri/src/services/translation/` 目录。
- 新增 `src-tauri/src/commands/translation.rs`。
- 修改 `src-tauri/src/error.rs`，新增翻译相关错误码。
- 修改 `MailViewer.vue` Header，新增翻译入口。
- 依赖 AI 助手模块定义的 `AiProvider` 模型。
