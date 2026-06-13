use crate::platform::config::{AiRuntimeProvider, AppConfig};

use super::models::ApplicationSetting;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiRuntimeSettings {
    pub provider: AiRuntimeProvider,
    pub base_url: String,
    pub chat_model: String,
    pub embedding_model: String,
    pub timeout_seconds: u64,
}

impl AiRuntimeSettings {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            provider: config.ai_provider(),
            base_url: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_base_url().to_owned(),
                AiRuntimeProvider::OmniRoute => config.omniroute_base_url().to_owned(),
            },
            chat_model: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_chat_model().to_owned(),
                AiRuntimeProvider::OmniRoute => config.omniroute_chat_model().to_owned(),
            },
            embedding_model: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_embed_model().to_owned(),
                AiRuntimeProvider::OmniRoute => config.omniroute_embed_model().to_owned(),
            },
            timeout_seconds: match config.ai_provider() {
                AiRuntimeProvider::Ollama => config.ollama_timeout_seconds(),
                AiRuntimeProvider::OmniRoute => config.omniroute_timeout_seconds(),
            },
        }
    }
}

pub(crate) fn runtime_settings_from_values(
    settings: &[ApplicationSetting],
    fallback: &AppConfig,
) -> AiRuntimeSettings {
    AiRuntimeSettings {
        provider: ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()),
        base_url: ai_base_url_value(settings, fallback),
        chat_model: ai_chat_model_value(settings, fallback),
        embedding_model: ai_embedding_model_value(settings, fallback),
        timeout_seconds: integer_value(settings, "ai.timeout_seconds")
            .and_then(|value| u64::try_from(value).ok())
            .filter(|value| *value > 0)
            .unwrap_or_else(|| {
                match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
                    AiRuntimeProvider::Ollama => fallback.ollama_timeout_seconds(),
                    AiRuntimeProvider::OmniRoute => fallback.omniroute_timeout_seconds(),
                }
            }),
    }
}

fn string_value(settings: &[ApplicationSetting], setting_key: &str) -> Option<String> {
    settings
        .iter()
        .find(|setting| setting.setting_key == setting_key)
        .and_then(|setting| setting.value.as_str())
        .map(str::to_owned)
}

fn ai_provider_value(settings: &[ApplicationSetting]) -> Option<AiRuntimeProvider> {
    string_value(settings, "ai.provider")
        .as_deref()
        .and_then(|value| AiRuntimeProvider::try_from(value).ok())
}

fn ai_base_url_value(settings: &[ApplicationSetting], fallback: &AppConfig) -> String {
    match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
        AiRuntimeProvider::Ollama => string_value(settings, "ai.ollama_base_url")
            .unwrap_or_else(|| fallback.ollama_base_url().to_owned()),
        AiRuntimeProvider::OmniRoute => string_value(settings, "ai.omniroute_base_url")
            .unwrap_or_else(|| fallback.omniroute_base_url().to_owned()),
    }
}

fn ai_chat_model_value(settings: &[ApplicationSetting], fallback: &AppConfig) -> String {
    match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
        AiRuntimeProvider::Ollama => string_value(settings, "ai.chat_model")
            .unwrap_or_else(|| fallback.ollama_chat_model().to_owned()),
        AiRuntimeProvider::OmniRoute => string_value(settings, "ai.omniroute_chat_model")
            .unwrap_or_else(|| fallback.omniroute_chat_model().to_owned()),
    }
}

fn ai_embedding_model_value(settings: &[ApplicationSetting], fallback: &AppConfig) -> String {
    match ai_provider_value(settings).unwrap_or_else(|| fallback.ai_provider()) {
        AiRuntimeProvider::Ollama => string_value(settings, "ai.embedding_model")
            .unwrap_or_else(|| fallback.ollama_embed_model().to_owned()),
        AiRuntimeProvider::OmniRoute => string_value(settings, "ai.omniroute_embedding_model")
            .unwrap_or_else(|| fallback.omniroute_embed_model().to_owned()),
    }
}

fn integer_value(settings: &[ApplicationSetting], setting_key: &str) -> Option<i64> {
    settings
        .iter()
        .find(|setting| setting.setting_key == setting_key)
        .and_then(|setting| setting.value.as_i64())
}
