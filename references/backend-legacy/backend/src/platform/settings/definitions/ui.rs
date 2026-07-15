use serde_json::json;

use super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![
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
