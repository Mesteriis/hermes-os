use serde_json::{Value, json};

use super::super::models::GmailOAuthPendingGrant;

pub(in crate::domains::mail::accounts::service) fn gmail_account_config(
    pending: &GmailOAuthPendingGrant,
) -> Value {
    json!({
        "auth": "oauth",
        "api": "gmail",
        "oauth_client_id": pending.request.client_id,
        "requested_scopes": pending.request.scopes,
        "connected_services": ["mail", "calendar", "contacts"],
        "history_stream_id": "gmail:history"
    })
}

pub(in crate::domains::mail::accounts::service) fn gmail_secret_metadata(
    pending: &GmailOAuthPendingGrant,
    external_account_id: &str,
    account_config: &Value,
) -> Value {
    json!({
        "provider": "gmail",
        "account_id": pending.account_id,
        "display_name": pending.request.display_name,
        "external_account_id": external_account_id,
        "connected_services": ["mail", "calendar", "contacts"],
        "provider_account_config": account_config
    })
}
