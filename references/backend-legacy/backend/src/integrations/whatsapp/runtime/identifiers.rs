use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{
    ProviderAccount, ProviderAccountSecretPurpose, WhatsAppProviderRuntimeShape, WhatsappWebError,
    account_provider_shape, account_runtime_kind,
};

pub(super) fn validated_or_generated_command_id(
    command_id: &Option<String>,
) -> Result<String, WhatsappWebError> {
    match command_id {
        Some(value) => super::validate_non_empty("command_id", value),
        None => Ok(new_whatsapp_command_id()),
    }
}

pub(super) fn new_whatsapp_command_id() -> String {
    let now = Utc::now();
    format!(
        "wacmd_{}_{}",
        now.timestamp_millis(),
        short_hash(&format!(
            "whatsapp-command-{}",
            now.timestamp_nanos_opt().unwrap_or(0)
        ))
    )
}

pub(super) fn whatsapp_text_preview_hash(text: &str) -> String {
    format!("sha256:{}", short_hash(text.trim()))
}

pub(super) fn short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

pub(super) fn whatsapp_session_secret_ref(account_id: &str) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        ProviderAccountSecretPurpose::WhatsappWebSessionKey.as_str()
    )
}

pub(super) fn session_secret_metadata(account: &ProviderAccount, extra: &Value) -> Value {
    let mut metadata = json!({
        "provider": account.provider_kind.as_str(),
        "provider_shape": account_provider_shape(account, WhatsAppProviderRuntimeShape::WebCompanion).as_str(),
        "account_id": account.account_id,
        "secret_purpose": ProviderAccountSecretPurpose::WhatsappWebSessionKey.as_str(),
        "runtime": account_runtime_kind(account),
    });
    if let (Some(metadata_object), Some(extra_object)) =
        (metadata.as_object_mut(), extra.as_object())
    {
        for (key, value) in extra_object {
            metadata_object.insert(key.clone(), value.clone());
        }
    }
    metadata
}
