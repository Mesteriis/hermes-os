use serde_json::{Value, json};

use super::super::helpers::email_provider_connected_services;
use super::super::models::ImapAccountSetupRequest;

pub(in crate::integrations::mail::accounts::service) fn imap_account_config(
    request: &ImapAccountSetupRequest,
) -> Value {
    let smtp_config = request.smtp_config();
    let mut account_config = json!({
        "host": request.host,
        "port": request.port,
        "tls": request.tls,
        "mailbox": request.mailbox,
        "username": request.username,
        "smtp_host": smtp_config.host,
        "smtp_port": smtp_config.port,
        "smtp_tls": smtp_config.tls,
        "smtp_starttls": smtp_config.starttls,
        "smtp_username": smtp_config.username
    });
    if let Some(services) = email_provider_connected_services(request.provider_kind) {
        account_config["connected_services"] = json!(services);
    }
    account_config
}

pub(in crate::integrations::mail::accounts::service) fn imap_secret_metadata(
    request: &ImapAccountSetupRequest,
    account_config: &Value,
) -> Value {
    let mut secret_metadata = json!({
        "provider": request.provider_kind.as_str(),
        "account_id": request.account_id,
        "display_name": request.display_name,
        "external_account_id": request.external_account_id,
        "provider_account_config": account_config
    });
    if let Some(services) = email_provider_connected_services(request.provider_kind) {
        secret_metadata["connected_services"] = json!(services);
    }
    secret_metadata
}
