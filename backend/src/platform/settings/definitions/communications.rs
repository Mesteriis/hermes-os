use serde_json::json;

use super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![DeclaredApplicationSetting {
        setting_key: "communications.mail.consecutive_failures_before_degraded",
        category: "communications",
        value_kind: SettingValueKind::Integer,
        default_value: json!(3),
        label: "Mail failures before degradation",
        description: "Number of consecutive provider sync failures before Hermes marks mail sync as degraded. Successful or skipped runs reset the counter.",
        metadata: json!({
            "ui_control": "number",
            "min": 1,
            "max": 10,
            "scope": "mail_sync",
            "stores_private_content": false
        }),
        is_editable: true,
    }]
}
