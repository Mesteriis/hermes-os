use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![DeclaredApplicationSetting {
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
    }]
}
