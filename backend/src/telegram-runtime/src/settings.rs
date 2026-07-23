//! Telegram-owned decoding of one admitted generic settings snapshot.

use std::path::PathBuf;

use hermes_runtime_protocol::v1::{SettingsSnapshotV1, setting_value_v1::Value};

const ACCOUNT_ID: &str = "telegram.account_id";
const API_ID: &str = "telegram.api_id";
const TDJSON_ARTIFACT_PATH: &str = "telegram.tdjson_artifact_path";
const DATABASE_DIRECTORY: &str = "telegram.database_directory";
const API_HASH_REVISION: &str = "telegram.api_hash_revision";
const SESSION_KEY_REVISION: &str = "telegram.session_encryption_key_revision";

pub struct TelegramRuntimeSettingsV1 {
    pub account_id: String,
    pub api_id: i64,
    pub tdjson_artifact_path: PathBuf,
    pub database_directory: PathBuf,
    pub api_hash_revision: u64,
    pub session_encryption_key_revision: u64,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<TelegramRuntimeSettingsV1, String> {
    let account_id = required_string(snapshot, ACCOUNT_ID)?;
    let api_id = required_signed(snapshot, API_ID)?;
    let tdjson_artifact_path = PathBuf::from(required_string(snapshot, TDJSON_ARTIFACT_PATH)?);
    let database_directory = PathBuf::from(required_string(snapshot, DATABASE_DIRECTORY)?);
    let api_hash_revision = required_unsigned(snapshot, API_HASH_REVISION)?;
    let session_encryption_key_revision = required_unsigned(snapshot, SESSION_KEY_REVISION)?;
    if api_id <= 0
        || api_hash_revision == 0
        || session_encryption_key_revision == 0
        || !tdjson_artifact_path.is_absolute()
        || !database_directory.is_absolute()
    {
        return Err(invalid_settings());
    }
    Ok(TelegramRuntimeSettingsV1 {
        account_id,
        api_id,
        tdjson_artifact_path,
        database_directory,
        api_hash_revision,
        session_encryption_key_revision,
    })
}

fn required_string(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<String, String> {
    match value(snapshot, setting_id)? {
        Value::StringValue(value) if !value.trim().is_empty() => Ok(value.clone()),
        _ => Err(invalid_settings()),
    }
}

fn required_signed(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<i64, String> {
    match value(snapshot, setting_id)? {
        Value::SignedIntegerValue(value) => Ok(*value),
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
    "Telegram runtime settings are invalid".to_owned()
}
