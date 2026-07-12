use serde_json::{Value, json};

use crate::ai::core::AI_EMBEDDING_DIMENSION;

use super::models::{AiCapabilitySlot, AiProviderAccount, AiProviderPreset};
use super::validation::CAPABILITY_SLOTS;

pub const BUILT_IN_OLLAMA_PROVIDER_ID: &str = "provider:built_in:ollama";
pub const OLLAMA_CHAT_MODEL: &str = "qwen3:4b";
pub const OLLAMA_EMBEDDING_MODEL: &str = "qwen3-embedding:4b";

pub(super) fn capability_slots() -> Vec<AiCapabilitySlot> {
    CAPABILITY_SLOTS
        .iter()
        .map(|slot| AiCapabilitySlot {
            slot: (*slot).to_owned(),
            label: settings_label(slot),
            description: capability_description(slot),
            requires_embedding_dimension: if *slot == "embeddings" {
                Some(AI_EMBEDDING_DIMENSION as i32)
            } else {
                None
            },
        })
        .collect()
}

pub(super) fn provider_presets() -> Vec<AiProviderPreset> {
    let mut presets = vec![
        AiProviderPreset {
            provider_kind: "built_in".to_owned(),
            provider_key: "ollama".to_owned(),
            display_name: "Built-in Ollama".to_owned(),
            privacy: "local".to_owned(),
            base_url: Some("http://192.168.1.2:11434".to_owned()),
            command_preset: None,
            capabilities: vec![
                "chat".to_owned(),
                "embeddings".to_owned(),
                "local_runtime".to_owned(),
            ],
        },
        AiProviderPreset {
            provider_kind: "cli".to_owned(),
            provider_key: "codex".to_owned(),
            display_name: "Codex CLI".to_owned(),
            privacy: "cli".to_owned(),
            base_url: None,
            command_preset: Some("codex".to_owned()),
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
        AiProviderPreset {
            provider_kind: "cli".to_owned(),
            provider_key: "claude".to_owned(),
            display_name: "Claude CLI".to_owned(),
            privacy: "cli".to_owned(),
            base_url: None,
            command_preset: Some("claude".to_owned()),
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
    ];

    presets.extend([
        api_provider_preset(
            "raw",
            "Raw OpenAI-compatible API",
            None,
            &[
                "chat",
                "reasoning",
                "summarization",
                "embeddings",
                "extraction",
            ],
        ),
        api_provider_preset(
            "openai",
            "OpenAI",
            Some("https://api.openai.com/v1"),
            &["chat", "reasoning", "embeddings"],
        ),
        api_provider_preset(
            "deepseek",
            "DeepSeek",
            Some("https://api.deepseek.com/v1"),
            &["chat", "reasoning"],
        ),
        api_provider_preset(
            "openrouter",
            "OpenRouter",
            Some("https://openrouter.ai/api/v1"),
            &["chat", "reasoning", "routing"],
        ),
        api_provider_preset(
            "groq",
            "Groq",
            Some("https://api.groq.com/openai/v1"),
            &["chat", "reasoning"],
        ),
        api_provider_preset(
            "together",
            "Together AI",
            Some("https://api.together.xyz/v1"),
            &["chat", "reasoning", "embeddings"],
        ),
        api_provider_preset(
            "fireworks",
            "Fireworks AI",
            Some("https://api.fireworks.ai/inference/v1"),
            &["chat", "reasoning", "embeddings"],
        ),
        api_provider_preset(
            "mistral",
            "Mistral AI",
            Some("https://api.mistral.ai/v1"),
            &["chat", "reasoning", "embeddings"],
        ),
        api_provider_preset(
            "xai",
            "xAI",
            Some("https://api.x.ai/v1"),
            &["chat", "reasoning"],
        ),
        api_provider_preset(
            "gemini-openai",
            "Google Gemini OpenAI-compatible",
            Some("https://generativelanguage.googleapis.com/v1beta/openai"),
            &["chat", "reasoning", "embeddings"],
        ),
        api_provider_preset(
            "perplexity",
            "Perplexity",
            Some("https://api.perplexity.ai"),
            &["chat", "reasoning"],
        ),
        api_provider_preset(
            "nvidia-nim",
            "NVIDIA NIM",
            Some("https://integrate.api.nvidia.com/v1"),
            &["chat", "reasoning", "embeddings"],
        ),
        api_provider_preset(
            "cerebras",
            "Cerebras",
            Some("https://api.cerebras.ai/v1"),
            &["chat", "reasoning"],
        ),
        api_provider_preset(
            "lm-studio",
            "LM Studio",
            Some("http://127.0.0.1:1234/v1"),
            &["chat", "local_runtime"],
        ),
        api_provider_preset(
            "vllm-local",
            "vLLM local",
            Some("http://127.0.0.1:8000/v1"),
            &["chat", "local_runtime"],
        ),
        api_provider_preset(
            "omniroute",
            "OmniRoute",
            Some("https://ai.sh-inc.ru/v1"),
            &["chat", "embeddings", "routing"],
        ),
    ]);

    presets
}

fn api_provider_preset(
    provider_key: &str,
    display_name: &str,
    base_url: Option<&str>,
    capabilities: &[&str],
) -> AiProviderPreset {
    AiProviderPreset {
        provider_kind: "api".to_owned(),
        provider_key: provider_key.to_owned(),
        display_name: display_name.to_owned(),
        privacy: "remote".to_owned(),
        base_url: base_url.map(str::to_owned),
        command_preset: None,
        capabilities: capabilities
            .iter()
            .map(|capability| (*capability).to_owned())
            .collect(),
    }
}

pub(super) struct CuratedModel {
    pub(super) model_key: &'static str,
    pub(super) display_name: &'static str,
    pub(super) category: &'static str,
    pub(super) privacy: &'static str,
    pub(super) capabilities: Vec<&'static str>,
    pub(super) context_window: Option<i32>,
    pub(super) embedding_dimension: Option<i32>,
    pub(super) metadata: Value,
}

pub(super) fn curated_models_for(provider: &AiProviderAccount) -> Vec<CuratedModel> {
    match (
        provider.provider_kind.as_str(),
        provider.provider_key.as_str(),
    ) {
        ("built_in", "ollama") => vec![
            CuratedModel {
                model_key: OLLAMA_CHAT_MODEL,
                display_name: "Qwen3 4B",
                category: "chat",
                privacy: "local",
                capabilities: vec!["chat", "reasoning", "summarization", "extraction"],
                context_window: Some(32768),
                embedding_dimension: None,
                metadata: json!({"curated": true, "pull_required": true}),
            },
            CuratedModel {
                model_key: OLLAMA_EMBEDDING_MODEL,
                display_name: "Qwen3 Embedding 4B",
                category: "embeddings",
                privacy: "local",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(AI_EMBEDDING_DIMENSION as i32),
                metadata: json!({"curated": true, "pull_required": true}),
            },
        ],
        ("api", "openai") => vec![
            CuratedModel {
                model_key: "gpt-5.5",
                display_name: "GPT-5.5",
                category: "reasoning",
                privacy: "remote",
                capabilities: vec!["chat", "reasoning", "summarization"],
                context_window: Some(128000),
                embedding_dimension: None,
                metadata: json!({"curated": true}),
            },
            CuratedModel {
                model_key: "text-embedding-3-large",
                display_name: "Text Embedding 3 Large",
                category: "embeddings",
                privacy: "remote",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(3072),
                metadata: json!({"curated": true, "embedding_route_supported": false}),
            },
        ],
        ("api", "deepseek") => vec![CuratedModel {
            model_key: "deepseek-chat",
            display_name: "DeepSeek Chat",
            category: "chat",
            privacy: "remote",
            capabilities: vec!["chat", "reasoning", "summarization"],
            context_window: Some(64000),
            embedding_dimension: None,
            metadata: json!({"curated": true}),
        }],
        ("api", "omniroute") => vec![
            CuratedModel {
                model_key: "codex/gpt-5.5",
                display_name: "Codex GPT-5.5",
                category: "reasoning",
                privacy: "remote",
                capabilities: vec!["chat", "reasoning", "summarization"],
                context_window: Some(128000),
                embedding_dimension: None,
                metadata: json!({"curated": true}),
            },
            CuratedModel {
                model_key: "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
                display_name: "Qwen3 Embedding via OmniRoute",
                category: "embeddings",
                privacy: "remote",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(AI_EMBEDDING_DIMENSION as i32),
                metadata: json!({"curated": true}),
            },
        ],
        ("cli", "codex") => vec![CuratedModel {
            model_key: "codex-cli/default",
            display_name: "Codex CLI Default",
            category: "reasoning",
            privacy: "cli",
            capabilities: vec!["chat", "reasoning"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": true, "command_preset": "codex"}),
        }],
        ("cli", "claude") => vec![CuratedModel {
            model_key: "claude-cli/default",
            display_name: "Claude CLI Default",
            category: "reasoning",
            privacy: "cli",
            capabilities: vec!["chat", "reasoning"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": true, "command_preset": "claude"}),
        }],
        _ => vec![CuratedModel {
            model_key: "custom/default",
            display_name: "Custom default",
            category: "chat",
            privacy: if provider.provider_kind == "api" {
                "remote"
            } else {
                "cli"
            },
            capabilities: vec!["chat"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": false}),
        }],
    }
}

pub(super) fn default_capabilities(provider_kind: &str, provider_key: &str) -> Vec<String> {
    match provider_kind {
        "built_in" => vec!["chat", "embeddings", "local_runtime"],
        "cli" => vec!["chat", "reasoning"],
        "api" if provider_key == "omniroute" => vec!["chat", "embeddings", "routing"],
        "api" => vec!["chat", "reasoning"],
        _ => vec!["chat"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn settings_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capability_description(slot: &str) -> String {
    match slot {
        "default_chat" => "General source-backed answers and chat.".to_owned(),
        "reasoning" => "Higher-effort planning and synthesis.".to_owned(),
        "summarization" => "Short summaries over local context.".to_owned(),
        "mail_intelligence" => "Communication analysis and operational context.".to_owned(),
        "reply_draft" => "Drafting replies without sending provider messages.".to_owned(),
        "extraction" => "Structured extraction from untrusted source text.".to_owned(),
        "embeddings" => "Semantic index embeddings; dimension constrained.".to_owned(),
        "meeting_prep" => "Meeting brief generation from local context.".to_owned(),
        _ => "AI capability.".to_owned(),
    }
}
