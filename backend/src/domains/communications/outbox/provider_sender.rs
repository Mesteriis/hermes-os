use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::communications::core::{
    EmailProviderKind, ProviderAccount, ProviderAccountSecretPurpose,
};
use crate::integrations::mail::accounts::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::GmailApiClient;
use crate::platform::secrets::SecretReferenceStore;
use crate::vault::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore, HostVault,
};

use super::smtp_sender::{LiveSmtpTransport, SmtpOutboxEmailSender, SmtpTransport};
use super::{
    CommunicationOutboxItem, OutboxDeliveryError, OutboxEmailSender, OutboxSendReceipt,
    outgoing_email_from_outbox_item,
};

#[derive(Clone)]
pub struct ProviderOutboxEmailSender<T = LiveSmtpTransport> {
    pool: PgPool,
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    vault: HostVault,
    smtp_sender: SmtpOutboxEmailSender<HostVault, T>,
}

impl<T> ProviderOutboxEmailSender<T>
where
    T: SmtpTransport,
{
    pub fn new(pool: PgPool, vault: HostVault, smtp_transport: T) -> Self {
        Self {
            pool: pool.clone(),
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            secret_store: SecretReferenceStore::new(pool.clone()),
            vault: vault.clone(),
            smtp_sender: SmtpOutboxEmailSender::new(pool, vault, smtp_transport),
        }
    }
}

impl<T> OutboxEmailSender for ProviderOutboxEmailSender<T>
where
    T: SmtpTransport,
{
    fn send<'a>(
        &'a self,
        item: &'a CommunicationOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let account = self
                .provider_account_store
                .get(&item.account_id)
                .await
                .map_err(|error| delivery_error("provider account lookup failed", error))?
                .ok_or_else(|| {
                    OutboxDeliveryError::Transport("provider account was not found".to_owned())
                })?;

            if matches!(account.provider_kind, EmailProviderKind::Gmail)
                && gmail_send_enabled(&account.config)
            {
                return self.send_gmail(item, &account).await;
            }

            self.smtp_sender.send(item).await
        })
    }
}

impl<T> ProviderOutboxEmailSender<T>
where
    T: SmtpTransport,
{
    async fn send_gmail(
        &self,
        item: &CommunicationOutboxItem,
        account: &ProviderAccount,
    ) -> Result<OutboxSendReceipt, OutboxDeliveryError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::OauthToken,
            )
            .await
            .map_err(|error| delivery_error("Gmail OAuth credential lookup failed", error))?
            .ok_or_else(|| {
                OutboxDeliveryError::Transport(
                    "Gmail OAuth credential is unavailable for this account".to_owned(),
                )
            })?;
        let account_setup = EmailAccountSetupService::new_with_host_vault(
            self.pool.clone(),
            self.secret_store.clone(),
            self.vault.clone(),
        );
        let access_token = account_setup
            .refresh_gmail_access_token(&binding.secret_ref)
            .await
            .map_err(|error| delivery_error("Gmail OAuth token refresh failed", error))?;
        let email = outgoing_email_from_outbox_item(item, account);
        let result = GmailApiClient::new(gmail_api_base_url(&account.config))
            .user_id("me")
            .send_message(&access_token, &email)
            .await
            .map_err(|error| delivery_error("Gmail API send failed", error))?;

        Ok(OutboxSendReceipt {
            provider_message_id: result.message_id,
            accepted_recipients: result.accepted_recipients,
        })
    }
}

fn gmail_send_enabled(config: &Value) -> bool {
    config
        .get("gmail_send_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn gmail_api_base_url(config: &Value) -> &str {
    config
        .get("gmail_api_base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("https://www.googleapis.com")
}

fn delivery_error(
    public_message: &'static str,
    error: impl std::fmt::Display,
) -> OutboxDeliveryError {
    tracing::warn!(error = %error, "provider outbox delivery failed");
    OutboxDeliveryError::Transport(public_message.to_owned())
}
