use serde_json::{Value, json};

use crate::domains::mail::core::EmailProviderKind;

pub(super) fn fallback_provider_account_config(
    provider_kind: EmailProviderKind,
    metadata: &Value,
    external_account_id: &str,
) -> Value {
    let connected_services = metadata
        .get("connected_services")
        .cloned()
        .unwrap_or_else(|| json!(["mail"]));
    match provider_kind {
        EmailProviderKind::Gmail => json!({
            "auth": "oauth",
            "api": "gmail",
            "connected_services": connected_services,
            "history_stream_id": "gmail:history"
        }),
        EmailProviderKind::Icloud => json!({
            "host": "imap.mail.me.com",
            "port": 993,
            "tls": true,
            "mailbox": "INBOX",
            "username": external_account_id,
            "connected_services": connected_services
        }),
        EmailProviderKind::Imap => json!({
            "username": external_account_id,
            "connected_services": connected_services
        }),
        _ => json!({}),
    }
}

pub(super) fn fallback_display_name(
    provider_kind: EmailProviderKind,
    label: &str,
    account_id: &str,
) -> String {
    let trimmed = label.trim();
    if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("IMAP password") {
        return trimmed.to_owned();
    }
    match provider_kind {
        EmailProviderKind::Gmail => "Google Workspace".to_owned(),
        EmailProviderKind::Icloud => "iCloud".to_owned(),
        EmailProviderKind::Imap => account_id.to_owned(),
        _ => account_id.to_owned(),
    }
}

pub(super) fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub(super) fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}
