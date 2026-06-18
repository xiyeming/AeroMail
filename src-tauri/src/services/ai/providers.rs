use crate::models::ai::AiProviderKind;

#[derive(Debug, Clone)]
pub struct AiProviderPreset {
    pub name: String,
    pub base_url: String,
    pub default_model: String,
}

#[must_use]
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
            name: "Qwen".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            default_model: "qwen-turbo".to_string(),
        },
        AiProviderKind::Zhipu => AiProviderPreset {
            name: "Zhipu GLM".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            default_model: "glm-4-flash".to_string(),
        },
        AiProviderKind::MiniMax => AiProviderPreset {
            name: "MiniMax".to_string(),
            base_url: "https://api.minimax.chat/v1".to_string(),
            default_model: "abab6.5s-chat".to_string(),
        },
        AiProviderKind::Baichuan => AiProviderPreset {
            name: "Baichuan".to_string(),
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
