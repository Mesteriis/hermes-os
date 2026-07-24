//! Zulip-owned decoding of an admitted generic settings snapshot.

use hermes_runtime_protocol::v1::{SettingsSnapshotV1, setting_value_v1::Value};
use hermes_zulip_api::{ZulipAccountV1, validate_account};

const ACCOUNT_ID: &str = "zulip.account_id";
const REALM_URL: &str = "zulip.realm_url";
const BOT_EMAIL: &str = "zulip.bot_email";
const API_KEY_REVISION: &str = "zulip.api_key_revision";

pub struct ZulipRuntimeSettingsV1 {
    pub account: ZulipAccountV1,
    pub api_key_revision: u64,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<ZulipRuntimeSettingsV1, String> {
    let account = ZulipAccountV1 {
        account_id: required_string(snapshot, ACCOUNT_ID)?,
        realm_url: required_string(snapshot, REALM_URL)?,
        bot_email: required_string(snapshot, BOT_EMAIL)?,
    };
    let api_key_revision = required_unsigned(snapshot, API_KEY_REVISION)?;
    if !validate_account(&account) || api_key_revision == 0 {
        return Err(invalid_settings());
    }
    Ok(ZulipRuntimeSettingsV1 {
        account,
        api_key_revision,
    })
}

fn required_string(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<String, String> {
    match value(snapshot, setting_id)? {
        Value::StringValue(value) if !value.trim().is_empty() => Ok(value.clone()),
        _ => Err(invalid_settings()),
    }
}

fn required_unsigned(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<u64, String> {
    match value(snapshot, setting_id)? {
        Value::UnsignedIntegerValue(value) => Ok(*value),
        _ => Err(invalid_settings()),
    }
}

fn value<'a>(snapshot: &'a SettingsSnapshotV1, setting_id: &str) -> Result<&'a Value, String> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id {
            let value = entry.value.as_ref().and_then(|value| value.value.as_ref());
            if selected.replace(value).is_some() {
                return Err(invalid_settings());
            }
        }
    }
    selected.flatten().ok_or_else(invalid_settings)
}

fn invalid_settings() -> String {
    "Zulip runtime settings are invalid".to_owned()
}
