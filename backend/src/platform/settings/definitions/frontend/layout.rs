use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![layout_setting(), sidebar_setting()]
}

fn layout_setting() -> DeclaredApplicationSetting {
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
    }
}

fn sidebar_setting() -> DeclaredApplicationSetting {
    DeclaredApplicationSetting {
        setting_key: "frontend.sidebar",
        category: "frontend",
        value_kind: SettingValueKind::Json,
        default_value: json!({
            "schemaVersion": 3,
            "rootItemIds": [
                "home",
                "group:communications",
                "personas",
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
    }
}
