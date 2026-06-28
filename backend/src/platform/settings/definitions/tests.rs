use serde_json::json;

use super::super::models::SettingValueKind;
use super::*;
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

#[test]
fn zoom_remote_transcript_download_setting_is_declared_as_opt_in_boolean() {
    let setting = declared_setting("privacy.zoom_remote_transcript_download_enabled")
        .expect("zoom transcript download policy setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Boolean);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(false));
    assert_eq!(setting.metadata["ui_control"], json!("checkbox"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_opt_in")
    );
}

#[test]
fn zoom_remote_recording_download_setting_is_declared_as_opt_in_boolean() {
    let setting = declared_setting("privacy.zoom_remote_recording_download_enabled")
        .expect("zoom recording download policy setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Boolean);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(false));
    assert_eq!(setting.metadata["ui_control"], json!("checkbox"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_opt_in")
    );
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn zoom_recording_import_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.zoom_recording_import_retention_days")
        .expect("zoom recording import retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn zoom_transcript_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.zoom_transcript_retention_days")
        .expect("zoom transcript retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("zoom"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn yandex_telemost_recording_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.yandex_telemost_recording_retention_days")
        .expect("yandex telemost recording retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("yandex_telemost"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}

#[test]
fn yandex_telemost_speaker_timeline_retention_setting_is_declared_as_editable_integer() {
    let setting = declared_setting("privacy.yandex_telemost_speaker_timeline_retention_days")
        .expect("yandex telemost speaker timeline retention setting");

    assert_eq!(setting.category, "privacy");
    assert_eq!(setting.value_kind, SettingValueKind::Integer);
    assert!(setting.is_editable);
    assert_eq!(setting.default_value, json!(0));
    assert_eq!(setting.metadata["ui_control"], json!("number"));
    assert_eq!(setting.metadata["scope"], json!("yandex_telemost"));
    assert_eq!(
        setting.metadata["policy_kind"],
        json!("owner_visible_retention")
    );
    assert_eq!(setting.metadata["min"], json!(0));
    assert_eq!(setting.metadata["max"], json!(3650));
    assert_eq!(setting.metadata["stores_private_content"], json!(true));
}
