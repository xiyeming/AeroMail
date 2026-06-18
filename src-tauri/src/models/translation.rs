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
