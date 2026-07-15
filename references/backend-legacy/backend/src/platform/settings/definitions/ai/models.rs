use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "ai.chat_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("qwen3:4b"),
            label: "Chat model",
            description: "Ollama model used for chat and source-backed answers.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "qwen3:4b"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.omniroute_chat_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("codex/gpt-5.5"),
            label: "OmniRoute chat model",
            description: "OpenAI-compatible OmniRoute model used for chat and source-backed answers when ai.provider is omniroute.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "codex/gpt-5.5"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.embedding_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("qwen3-embedding:4b"),
            label: "Embedding model",
            description: "Ollama model used for semantic embeddings.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "qwen3-embedding:4b"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.omniroute_embedding_model",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("openai-compatible-chat-ollama-pve/qwen3-embedding:4b"),
            label: "OmniRoute embedding model",
            description: "OpenAI-compatible OmniRoute embedding model. It must return 2560 dimensions until the semantic index shape changes.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
                "required_dimension": 2560
            }),
            is_editable: true,
        },
    ]
}
