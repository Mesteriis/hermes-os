use serde_json::json;

use super::super::super::constants::{UI_STATE_FORBIDDEN_KEYS, UI_STATE_MAX_BYTES};
use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![locale_setting(), ui_state_setting()]
}

fn locale_setting() -> DeclaredApplicationSetting {
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
    }
}

fn ui_state_setting() -> DeclaredApplicationSetting {
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
    }
}
