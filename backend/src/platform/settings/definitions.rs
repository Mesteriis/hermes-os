use serde_json::json;

use super::constants::{UI_STATE_FORBIDDEN_KEYS, UI_STATE_MAX_BYTES};
use super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(crate) fn declared_setting_keys() -> Vec<String> {
    declared_application_settings()
        .into_iter()
        .map(|setting| setting.setting_key.to_owned())
        .collect()
}

pub(crate) fn declared_setting(setting_key: &str) -> Option<DeclaredApplicationSetting> {
    declared_application_settings()
        .into_iter()
        .find(|setting| setting.setting_key == setting_key)
}

pub(crate) fn declared_application_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
        DeclaredApplicationSetting {
            setting_key: "server.http_addr",
            category: "server",
            value_kind: SettingValueKind::String,
            default_value: json!("127.0.0.1:8080"),
            label: "Backend HTTP bind",
            description: "Backend HTTP address used when the local server starts. Changes require a backend restart.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "127.0.0.1:8080",
                "restart_required": true,
                "bootstrap": true,
                "env_var": "HERMES_HTTP_ADDR"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.api_base_url",
            category: "frontend",
            value_kind: SettingValueKind::String,
            default_value: json!("http://127.0.0.1:8080"),
            label: "Frontend API base URL",
            description: "Backend URL used by the desktop shell after it has loaded local settings.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "http://127.0.0.1:8080",
                "bootstrap": true,
                "env_var": "VITE_HERMES_API_BASE_URL"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.layout",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 2,
                "views": {}
            }),
            label: "Frontend layout",
            description: "Desktop widget layout preset selections and user overrides. Stores layout metadata only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "json",
                "schema_version": 2,
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.sidebar",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 3,
                "rootItemIds": [
                    "home",
                    "group:communications",
                    "persons",
                    "projects",
                    "tasks",
                    "calendar",
                    "documents",
                    "notes",
                    "knowledge",
                    "agents"
                ],
                "groups": [
                    {
                        "id": "communications",
                        "label": "Communications",
                        "icon": "tabler:messages",
                        "itemIds": [
                            "communications.mail",
                            "communications.telegram",
                            "communications.whatsapp",
                            "communications.calls",
                            "communications.meetings",
                            "timeline"
                        ],
                        "separatorBeforeItemIds": []
                    }
                ],
                "hiddenItemIds": []
            }),
            label: "Frontend sidebar",
            description: "Desktop sidebar grouping, item order and hidden workspace metadata. Stores navigation preferences only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "json",
                "schema_version": 3,
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.theme",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 1,
                "shellBackground": "network-mesh",
                "backgroundBrightness": 70,
                "accentColor": "teal",
                "panelOpacity": 70,
                "panelBlur": 12
            }),
            label: "Frontend appearance",
            description: "Desktop shell background, image brightness, panel transparency, panel blur and accent color. Stores visual preferences only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "appearance",
                "schema_version": 1,
                "allowed_backgrounds": [
                    "none",
                    "network-mesh",
                    "data-stream",
                    "node-frame",
                    "eclipse-grid",
                    "dna-blueprint",
                    "forest-network",
                    "forest-stream",
                    "knowledge-map",
                    "rune-gold",
                    "rune-teal"
                ],
                "allowed_brightness": [30, 40, 50, 60, 70, 80, 90, 100],
                "allowed_accent_colors": ["teal", "cyan", "blue", "violet", "amber", "rose"],
                "allowed_panel_opacity": [40, 50, 60, 70, 80, 90, 100],
                "allowed_panel_blur": [0, 4, 8, 12, 16, 20, 24],
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.locale",
            category: "frontend",
            value_kind: SettingValueKind::String,
            default_value: json!("en"),
            label: "Frontend locale",
            description: "Desktop interface language preference. Stores only the selected locale code.",
            metadata: json!({
                "ui_control": "language",
                "allowed_values": ["en", "ru"],
                "stores_private_content": false,
                "restart_required": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "frontend.ui_state",
            category: "frontend",
            value_kind: SettingValueKind::Json,
            default_value: json!({
                "schemaVersion": 1
            }),
            label: "Frontend UI state",
            description: "Transient desktop UI state for restoring visible workspace context. Stores non-authoritative UI metadata only, never message bodies, document text or secrets.",
            metadata: json!({
                "ui_control": "hidden",
                "schema_version": 1,
                "stores_private_content": false,
                "restart_required": false,
                "max_bytes": UI_STATE_MAX_BYTES,
                "forbidden_keys": UI_STATE_FORBIDDEN_KEYS
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.provider",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("ollama"),
            label: "AI provider",
            description: "AI runtime provider. Ollama is local by default; OmniRoute is explicit opt-in and uses an env-backed API key.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["ollama", "omniroute"],
                "stores_secret": false
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.ollama_base_url",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("http://127.0.0.1:11434"),
            label: "Ollama base URL",
            description: "Local Ollama HTTP endpoint used by AI runtime requests.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "http://127.0.0.1:11434"
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ai.omniroute_base_url",
            category: "ai",
            value_kind: SettingValueKind::String,
            default_value: json!("https://ai.sh-inc.ru/v1"),
            label: "OmniRoute base URL",
            description: "OpenAI-compatible OmniRoute endpoint. API key is read from HERMES_OMNIROUTE_API_KEY, never from application settings.",
            metadata: json!({
                "ui_control": "text",
                "placeholder": "https://ai.sh-inc.ru/v1",
                "stores_secret": false,
                "key_env": "HERMES_OMNIROUTE_API_KEY"
            }),
            is_editable: true,
        },
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
        DeclaredApplicationSetting {
            setting_key: "ai.timeout_seconds",
            category: "ai",
            value_kind: SettingValueKind::Integer,
            default_value: json!(120),
            label: "AI request timeout",
            description: "Timeout in seconds for Ollama HTTP requests.",
            metadata: json!({
                "ui_control": "number",
                "min": 1,
                "max": 600,
                "step": 1
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ui.theme",
            category: "ui",
            value_kind: SettingValueKind::String,
            default_value: json!("system"),
            label: "Theme",
            description: "Desktop shell color theme preference.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["system", "dark", "light"]
            }),
            is_editable: true,
        },
        DeclaredApplicationSetting {
            setting_key: "ui.density",
            category: "ui",
            value_kind: SettingValueKind::String,
            default_value: json!("comfortable"),
            label: "UI density",
            description: "Desktop shell spacing density preference.",
            metadata: json!({
                "ui_control": "select",
                "allowed_values": ["comfortable", "compact"]
            }),
            is_editable: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    use crate::platform::settings::SettingsError;

    #[test]
    fn frontend_locale_setting_is_declared_as_editable_string() {
        let setting = declared_setting("frontend.locale").expect("frontend locale setting");

        assert_eq!(setting.category, "frontend");
        assert_eq!(setting.value_kind, SettingValueKind::String);
        assert!(setting.is_editable);
        assert_eq!(setting.default_value, json!("en"));
        assert_eq!(setting.metadata["ui_control"], json!("language"));
        assert_eq!(setting.metadata["allowed_values"], json!(["en", "ru"]));
        assert_eq!(setting.metadata["stores_private_content"], json!(false));
    }

    #[test]
    fn frontend_ui_state_setting_is_declared_as_hidden_json() {
        let setting = declared_setting("frontend.ui_state").expect("frontend ui state setting");

        assert_eq!(setting.category, "frontend");
        assert_eq!(setting.value_kind, SettingValueKind::Json);
        assert!(setting.is_editable);
        assert_eq!(setting.metadata["ui_control"], json!("hidden"));
        assert_eq!(setting.metadata["schema_version"], json!(1));
        assert_eq!(setting.metadata["stores_private_content"], json!(false));
        assert_eq!(setting.default_value["schemaVersion"], json!(1));
    }

    #[test]
    fn frontend_ui_state_rejects_private_content_keys() {
        let setting = declared_setting("frontend.ui_state").expect("frontend ui state setting");
        let value = json!({
            "schemaVersion": 1,
            "savedAt": "2026-06-11T12:00:00Z",
            "expiresAt": "2026-06-18T12:00:00Z",
            "communications": {
                "selectedMessageId": "msg-1",
                "compose": {
                    "draftId": "draft-1",
                    "body": "private draft body"
                }
            }
        });

        let error = setting
            .value_kind
            .validate_value(&value, &setting.metadata)
            .expect_err("private body key rejected");

        assert!(matches!(error, SettingsError::InvalidValue(_)));
    }

    #[test]
    fn frontend_ui_state_rejects_oversized_snapshots() {
        let setting = declared_setting("frontend.ui_state").expect("frontend ui state setting");
        let value = json!({
            "schemaVersion": 1,
            "savedAt": "2026-06-11T12:00:00Z",
            "expiresAt": "2026-06-18T12:00:00Z",
            "shell": {
                "expandedSidebarGroupIds": vec!["communications"; 10_000]
            }
        });

        let error = setting
            .value_kind
            .validate_value(&value, &setting.metadata)
            .expect_err("oversized snapshot rejected");

        assert!(matches!(error, SettingsError::InvalidValue(_)));
    }
}
