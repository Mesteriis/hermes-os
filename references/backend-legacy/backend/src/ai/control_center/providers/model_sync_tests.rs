use chrono::Utc;
use serde_json::json;

use super::*;

#[test]
fn openai_compatible_models_url_appends_models_path() {
    assert_eq!(
        openai_compatible_models_url("https://api.openai.com/v1")
            .expect("valid URL")
            .as_str(),
        "https://api.openai.com/v1/models"
    );
    assert_eq!(
        openai_compatible_models_url("http://127.0.0.1:11434/v1/models")
            .expect("valid URL")
            .as_str(),
        "http://127.0.0.1:11434/v1/models"
    );
}

#[test]
fn ollama_tags_url_uses_native_model_inventory_endpoint() {
    assert_eq!(
        ollama_tags_url("http://127.0.0.1:11434")
            .expect("valid URL")
            .as_str(),
        "http://127.0.0.1:11434/api/tags"
    );
    assert_eq!(
        ollama_tags_url("http://127.0.0.1:11434/v1")
            .expect("valid URL")
            .as_str(),
        "http://127.0.0.1:11434/api/tags"
    );
}

#[test]
fn ollama_show_url_uses_native_model_detail_endpoint() {
    assert_eq!(
        ollama_show_url("http://127.0.0.1:11434")
            .expect("valid URL")
            .as_str(),
        "http://127.0.0.1:11434/api/show"
    );
    assert_eq!(
        ollama_show_url("http://127.0.0.1:11434/v1")
            .expect("valid URL")
            .as_str(),
        "http://127.0.0.1:11434/api/show"
    );
}

#[test]
fn discovered_models_preserve_all_provider_records() {
    let provider = test_provider();
    let response: OpenAiCompatibleModelsResponse = serde_json::from_value(json!({
        "data": [
            {"id": "gpt-4.1", "object": "model", "owned_by": "openai"},
            {"id": "text-embedding-3-large", "object": "model", "owned_by": "openai"},
            {"id": "o3-mini", "object": "model", "owned_by": "openai"},
            {"id": "gpt-4.1", "object": "model", "owned_by": "duplicate"}
        ]
    }))
    .expect("OpenAI-compatible payload");

    let models = discovered_models_from_response(&provider, response);

    assert_eq!(models.len(), 3);
    assert!(models.iter().any(|model| model.model_key == "gpt-4.1"));
    assert!(models.iter().any(|model| {
        model.model_key == "text-embedding-3-large"
            && model.category == "embeddings"
            && model.capabilities == vec!["embeddings".to_owned()]
    }));
    assert!(models.iter().any(|model| {
        model.model_key == "o3-mini" && model.capabilities.contains(&"reasoning".to_owned())
    }));
}

#[test]
fn discovered_ollama_models_preserve_local_runtime_inventory() {
    let mut provider = test_provider();
    provider.provider_kind = "built_in".to_owned();
    provider.provider_key = "ollama".to_owned();
    provider.config = json!({"base_url": "http://127.0.0.1:11434"});
    provider.capabilities = vec!["chat".to_owned(), "embeddings".to_owned()];
    let response: OllamaTagsResponse = serde_json::from_value(json!({
        "models": [
            {
                "name": "qwen3:4b",
                "model": "qwen3:4b",
                "modified_at": "2026-07-06T10:00:00Z",
                "size": 123,
                "digest": "sha256:test",
                "details": {"family": "qwen3"}
            },
            {
                "name": "qwen3-embedding:4b",
                "model": "qwen3-embedding:4b",
                "details": {"family": "qwen3"}
            }
        ]
    }))
    .expect("Ollama tags payload");
    let show_by_model = HashMap::from([
        (
            "qwen3:4b".to_owned(),
            serde_json::from_value(json!({
                "capabilities": ["completion"],
                "model_info": {
                    "qwen3.context_length": 32768,
                    "qwen3.embedding_length": 2560,
                    "qwen3.vision.block_count": 1
                }
            }))
            .expect("Ollama show payload"),
        ),
        (
            "qwen3-embedding:4b".to_owned(),
            serde_json::from_value(json!({
                "capabilities": ["embedding"],
                "model_info": {
                    "qwen3.embedding_length": 2560
                }
            }))
            .expect("Ollama show payload"),
        ),
    ]);

    let models = discovered_ollama_models_from_response(&provider, response, &show_by_model);

    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|model| {
        model.model_key == "qwen3:4b"
            && model.category == "chat"
            && model.privacy == "local"
            && model.capabilities.contains(&"chat".to_owned())
            && model.capabilities.contains(&"vision".to_owned())
            && model.capabilities.contains(&"multimodal".to_owned())
            && model.context_window == Some(32768)
            && model.metadata["capability_source"] == "ollama_api_show"
    }));
    assert!(models.iter().any(|model| {
        model.model_key == "qwen3-embedding:4b"
            && model.category == "embeddings"
            && model.capabilities == vec!["embeddings".to_owned()]
            && model.embedding_dimension == Some(2560)
    }));
}

#[test]
fn discovered_cli_models_read_qwen_style_model_provider_settings() {
    let mut provider = test_provider();
    provider.provider_kind = "cli".to_owned();
    provider.provider_key = "qwen".to_owned();
    provider.capabilities = vec!["chat".to_owned(), "reasoning".to_owned()];
    let settings = json!({
        "modelProviders": {
            "openai": [
                {
                    "id": "qwen3-coder-plus",
                    "name": "Qwen3 Coder Plus",
                    "baseUrl": "https://dashscope.example.test/compatible-mode/v1",
                    "envKey": "BAILIAN_CODING_PLAN_API_KEY",
                    "generationConfig": {
                        "extra_body": {"enable_thinking": true},
                        "contextWindowSize": 262144
                    }
                },
                {
                    "id": "qwen3-embedding",
                    "name": "Qwen3 Embedding",
                    "capabilities": ["embedding"],
                    "embeddingDimension": 2560
                }
            ]
        }
    });

    let models = discovered_cli_models_from_settings(
        &provider,
        Path::new("/tmp/qwen-settings.json"),
        &settings,
    );

    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|model| {
        model.model_key == "qwen3-coder-plus"
            && model.display_name == "Qwen3 Coder Plus"
            && model.category == "reasoning"
            && model.capabilities.contains(&"reasoning".to_owned())
            && model.capabilities.contains(&"chat".to_owned())
            && model.context_window == Some(262144)
            && model.metadata["capability_source"] == "cli_settings_json"
            && model.metadata["reasoning"]["enabled"] == true
            && model.metadata.get("envKey").is_none()
    }));
    assert!(models.iter().any(|model| {
        model.model_key == "qwen3-embedding"
            && model.category == "embeddings"
            && model.capabilities == vec!["embeddings".to_owned()]
            && model.embedding_dimension == Some(2560)
    }));
}

#[test]
fn discovered_cli_models_read_claude_model_option_cache() {
    let mut provider = test_provider();
    provider.provider_kind = "cli".to_owned();
    provider.provider_key = "claude".to_owned();
    provider.capabilities = vec!["chat".to_owned(), "reasoning".to_owned()];
    let settings = json!({
        "additionalModelOptionsCache": [
            {
                "value": "claude-sonnet-5",
                "label": "Claude Sonnet 5",
                "description": "Fast reasoning model"
            },
            {
                "value": "claude-opus-5",
                "label": "Claude Opus 5",
                "reasoning": {"enabled": true, "effort": "high", "secret": "drop-me"}
            }
        ]
    });

    let models = discovered_cli_models_from_settings(
        &provider,
        Path::new("/tmp/claude-settings.json"),
        &settings,
    );

    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|model| {
        model.model_key == "claude-sonnet-5"
            && model.display_name == "Claude Sonnet 5"
            && model.privacy == "cli"
            && model.metadata["description"] == "Fast reasoning model"
    }));
    assert!(models.iter().any(|model| {
        model.model_key == "claude-opus-5"
            && model.capabilities.contains(&"reasoning".to_owned())
            && model.metadata["reasoning"]["effort"] == "high"
            && model.metadata["reasoning"].get("secret").is_none()
    }));
}

#[test]
fn discovered_cli_models_read_codex_model_cache_slugs() {
    let mut provider = test_provider();
    provider.provider_kind = "cli".to_owned();
    provider.provider_key = "codex".to_owned();
    provider.capabilities = vec!["chat".to_owned(), "reasoning".to_owned()];
    let settings = json!({
        "models": [
            {
                "slug": "gpt-5.5",
                "display_name": "GPT-5.5",
                "default_reasoning_level": "medium",
                "supported_reasoning_levels": ["low", "medium", "high"],
                "description": "Codex cache model"
            },
            {
                "slug": "codex-auto-review",
                "display_name": "Codex Auto Review",
                "supports_reasoning_summaries": true
            }
        ]
    });

    let models = discovered_cli_models_from_settings(
        &provider,
        Path::new("/tmp/codex-models-cache.json"),
        &settings,
    );

    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|model| {
        model.model_key == "gpt-5.5"
            && model.display_name == "GPT-5.5"
            && model.category == "reasoning"
            && model.capabilities.contains(&"chat".to_owned())
            && model.capabilities.contains(&"reasoning".to_owned())
            && model.metadata["description"] == "Codex cache model"
    }));
    assert!(models.iter().any(|model| {
        model.model_key == "codex-auto-review"
            && model.capabilities.contains(&"reasoning".to_owned())
    }));
}

#[test]
fn discovered_cli_models_read_nested_model_option_cache() {
    let mut provider = test_provider();
    provider.provider_kind = "cli".to_owned();
    provider.provider_key = "claude".to_owned();
    provider.capabilities = vec!["chat".to_owned()];
    let settings = json!({
        "profile": {
            "runtime": {
                "additionalModelOptionsCache": [
                    {"value": "claude-fable-5", "label": "Fable"}
                ]
            }
        }
    });

    let models = discovered_cli_models_from_settings(
        &provider,
        Path::new("/tmp/nested-cli-settings.json"),
        &settings,
    );

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].model_key, "claude-fable-5");
    assert_eq!(models[0].display_name, "Fable");
}

fn test_provider() -> AiProviderAccount {
    AiProviderAccount {
        provider_id: "provider:api:openai".to_owned(),
        provider_kind: "api".to_owned(),
        provider_key: "openai".to_owned(),
        display_name: "OpenAI".to_owned(),
        status: "ready".to_owned(),
        consent_state: "granted".to_owned(),
        consented_at: None,
        config: json!({"base_url": "https://api.openai.com/v1"}),
        capabilities: vec![
            "chat".to_owned(),
            "reasoning".to_owned(),
            "summarization".to_owned(),
            "embeddings".to_owned(),
        ],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
