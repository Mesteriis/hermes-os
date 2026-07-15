use serde_json::json;

use super::super::super::models::{DeclaredApplicationSetting, SettingValueKind};

pub(super) fn declared_settings() -> Vec<DeclaredApplicationSetting> {
    vec![DeclaredApplicationSetting {
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
    }]
}
