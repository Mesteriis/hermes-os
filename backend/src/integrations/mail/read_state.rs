use std::sync::Arc;

use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::integrations::mail::accounts::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::GmailApiClient;
use crate::integrations::mail::imap_write::{ImapWriteClient, ImapWriteConfig, ImapWriteError};
use crate::platform::communications::{
    EmailProviderKind, ProviderAccount, ProviderAccountSecretPurpose,
    ProviderSecretBindingLookupPort,
};
use crate::platform::secrets::{ResolvedSecret, SecretReferenceStore};
use crate::vault::HostVault;

use super::sync_provider::read_provider_secret;

pub struct EmailReadStateRequest<'a> {
    pub account: &'a ProviderAccount,
    pub provider_record_id: &'a str,
    pub message_metadata: &'a Value,
}

#[derive(Clone)]
pub struct LiveEmailReadStateService {
    pool: PgPool,
    vault: HostVault,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>,
    gmail_api_base_url: String,
}

impl LiveEmailReadStateService {
    pub fn new(
        pool: PgPool,
        vault: HostVault,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>,
        gmail_api_base_url: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            vault,
            provider_secret_binding_store,
            gmail_api_base_url: gmail_api_base_url.into(),
        }
    }

    pub async fn mark_message_read(
        &self,
        request: EmailReadStateRequest<'_>,
    ) -> Result<(), EmailReadStateError> {
        match request.account.provider_kind {
            EmailProviderKind::Gmail => self.mark_gmail_message_read(request).await,
            EmailProviderKind::Icloud | EmailProviderKind::Imap => {
                self.mark_imap_message_read(request).await
            }
            provider_kind => Err(EmailReadStateError::UnsupportedProvider(
                provider_kind.as_str(),
            )),
        }
    }

    async fn mark_gmail_message_read(
        &self,
        request: EmailReadStateRequest<'_>,
    ) -> Result<(), EmailReadStateError> {
        let access_token = self.gmail_access_token(&request.account.account_id).await?;
        GmailApiClient::new(gmail_api_base_url(
            &request.account.config,
            &self.gmail_api_base_url,
        ))
        .user_id("me")
        .mark_message_read(&access_token, request.provider_record_id)
        .await
        .map_err(EmailReadStateError::Gmail)
    }

    async fn mark_imap_message_read(
        &self,
        request: EmailReadStateRequest<'_>,
    ) -> Result<(), EmailReadStateError> {
        let config = imap_write_config(request.account, request.message_metadata)?;
        let secret_store = SecretReferenceStore::new(self.pool.clone());
        let password = read_provider_secret(
            self.provider_secret_binding_store.as_ref(),
            &secret_store,
            &self.vault,
            &request.account.account_id,
            ProviderAccountSecretPurpose::ImapPassword,
        )
        .await
        .map_err(|error| EmailReadStateError::Credential(error.to_string()))?;
        let uid = imap_uid(request.message_metadata)?;
        ImapWriteClient::new()
            .mark_seen(
                &ImapWriteConfig {
                    host: &config.host,
                    port: config.port,
                    tls: config.tls,
                    username: &config.username,
                    password: &password,
                    mailbox: &config.mailbox,
                },
                &[uid],
            )
            .await
            .map_err(EmailReadStateError::Imap)
    }

    async fn gmail_access_token(
        &self,
        account_id: &str,
    ) -> Result<ResolvedSecret, EmailReadStateError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(account_id, ProviderAccountSecretPurpose::OauthToken)
            .await
            .map_err(|error| EmailReadStateError::Credential(error.to_string()))?
            .ok_or(EmailReadStateError::MissingCredential)?;
        EmailAccountSetupService::new_with_host_vault_for_token_refresh(
            self.pool.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            self.vault.clone(),
        )
        .refresh_gmail_access_token(&binding.secret_ref)
        .await
        .map_err(|error| EmailReadStateError::Credential(error.to_string()))
    }
}

struct ImapWriteAccountConfig {
    host: String,
    port: u16,
    tls: bool,
    username: String,
    mailbox: String,
}

fn imap_write_config(
    account: &ProviderAccount,
    message_metadata: &Value,
) -> Result<ImapWriteAccountConfig, EmailReadStateError> {
    let config = account
        .config
        .as_object()
        .ok_or(EmailReadStateError::InvalidConfig)?;
    let mailbox = message_metadata
        .get("mailbox")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| config.get("mailbox").and_then(Value::as_str))
        .ok_or(EmailReadStateError::InvalidConfig)?;
    Ok(ImapWriteAccountConfig {
        host: config_string(config, "host")?,
        port: config
            .get("port")
            .and_then(Value::as_u64)
            .filter(|value| *value > 0 && *value <= u64::from(u16::MAX))
            .map(|value| value as u16)
            .ok_or(EmailReadStateError::InvalidConfig)?,
        tls: config.get("tls").and_then(Value::as_bool).unwrap_or(true),
        username: config_string(config, "username")?,
        mailbox: mailbox.trim().to_owned(),
    })
}

fn imap_uid(message_metadata: &Value) -> Result<u32, EmailReadStateError> {
    message_metadata
        .get("uid")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0 && *value <= u64::from(u32::MAX))
        .map(|value| value as u32)
        .ok_or(EmailReadStateError::MissingImapUid)
}

fn config_string(
    config: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<String, EmailReadStateError> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or(EmailReadStateError::InvalidConfig)
}

fn gmail_api_base_url<'a>(config: &'a Value, fallback: &'a str) -> &'a str {
    config
        .get("gmail_api_base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
}

#[derive(Debug, Error)]
pub enum EmailReadStateError {
    #[error("provider does not support email read-state synchronization: {0}")]
    UnsupportedProvider(&'static str),
    #[error("provider account configuration is incomplete")]
    InvalidConfig,
    #[error("IMAP message metadata does not contain a UID")]
    MissingImapUid,
    #[error("provider credential is unavailable")]
    MissingCredential,
    #[error("provider credential could not be resolved: {0}")]
    Credential(String),
    #[error("Gmail read-state update failed: {0}")]
    Gmail(crate::integrations::mail::gmail::client::EmailProviderNetworkError),
    #[error("IMAP read-state update failed: {0}")]
    Imap(ImapWriteError),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{imap_uid, imap_write_config};
    use crate::platform::communications::{EmailProviderKind, ProviderAccount};

    #[test]
    fn imap_write_uses_the_message_mailbox_and_uid() {
        let account = ProviderAccount {
            account_id: "account-1".to_owned(),
            provider_kind: EmailProviderKind::Icloud,
            display_name: "iCloud".to_owned(),
            external_account_id: "owner@example.test".to_owned(),
            config: json!({
                "host": "imap.example.test",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "owner@example.test"
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let metadata = json!({ "mailbox": "Archive", "uid": 42 });

        let config = imap_write_config(&account, &metadata).expect("config");
        assert_eq!(config.mailbox, "Archive");
        assert_eq!(imap_uid(&metadata).expect("uid"), 42);
    }
}
