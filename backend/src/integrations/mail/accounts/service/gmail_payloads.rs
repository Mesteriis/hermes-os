use serde_json::{Value, json};

use super::super::constants::GOOGLE_GMAIL_SEND_SCOPE;
use super::super::models::GmailOAuthPendingGrant;

pub(in crate::integrations::mail::accounts::service) fn gmail_account_config(
    pending: &GmailOAuthPendingGrant,
) -> Value {
    json!({
        "auth": "oauth",
        "api": "gmail",
        "oauth_client_id": pending.request.client_id,
        "requested_scopes": pending.request.scopes,
        "gmail_send_enabled": gmail_send_scope_requested(pending),
        "connected_services": ["mail", "calendar", "contacts"],
        "history_stream_id": "gmail:history"
    })
}

pub(in crate::integrations::mail::accounts::service) fn gmail_secret_metadata(
    pending: &GmailOAuthPendingGrant,
    account_id: &str,
    external_account_id: &str,
    account_config: &Value,
) -> Value {
    json!({
        "provider": "gmail",
        "account_id": account_id,
        "display_name": pending.request.display_name,
        "external_account_id": external_account_id,
        "connected_services": ["mail", "calendar", "contacts"],
        "provider_account_config": account_config
    })
}

fn gmail_send_scope_requested(pending: &GmailOAuthPendingGrant) -> bool {
    pending
        .request
        .scopes
        .iter()
        .any(|scope| scope.trim() == GOOGLE_GMAIL_SEND_SCOPE)
}
