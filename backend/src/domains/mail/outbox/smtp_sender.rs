use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::mail::core::{
    EmailProviderKind, ProviderAccount, ProviderAccountSecretPurpose, ProviderCredentialReader,
};
use crate::domains::mail::send::{
    EmailSendError, OutgoingEmail, SendResult, SmtpClient, SmtpConfig,
};
use crate::platform::secrets::{ResolvedSecret, SecretReferenceStore, SecretResolver};
use crate::vault::{CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore};

use super::{EmailOutboxItem, OutboxDeliveryError, OutboxEmailSender, OutboxSendReceipt};

pub trait SmtpTransport: Clone + Send + Sync {
    fn send<'a>(
        &'a self,
        config: &'a SmtpConfig,
        password: &'a ResolvedSecret,
        email: &'a OutgoingEmail,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>>;
}

#[derive(Clone, Default)]
pub struct LiveSmtpTransport;

impl SmtpTransport for LiveSmtpTransport {
    fn send<'a>(
        &'a self,
        config: &'a SmtpConfig,
        password: &'a ResolvedSecret,
        email: &'a OutgoingEmail,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>> {
        Box::pin(async move { SmtpClient::new().send(config, password, email).await })
    }
}

#[derive(Clone)]
pub struct SmtpOutboxEmailSender<R, T = LiveSmtpTransport> {
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    resolver: R,
    transport: T,
}

impl<R, T> SmtpOutboxEmailSender<R, T>
where
    R: SecretResolver,
    T: SmtpTransport,
{
    pub fn new(pool: PgPool, resolver: R, transport: T) -> Self {
        Self {
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            secret_store: SecretReferenceStore::new(pool),
            resolver,
            transport,
        }
    }
}

impl<R, T> OutboxEmailSender for SmtpOutboxEmailSender<R, T>
where
    R: SecretResolver + Send + Sync,
    T: SmtpTransport,
{
    fn send<'a>(
        &'a self,
        item: &'a EmailOutboxItem,
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
            let smtp_config = smtp_config_for_provider_account(&account)?;
            let credential_reader = ProviderCredentialReader::new(
                self.provider_secret_binding_store.clone(),
                self.secret_store.clone(),
                &self.resolver,
            );
            let credential = credential_reader
                .read(
                    &account.account_id,
                    ProviderAccountSecretPurpose::SmtpPassword,
                )
                .await
                .map_err(|error| {
                    delivery_error("SMTP credential is unavailable for this account", error)
                })?;
            let email = outgoing_email_from_outbox_item(item, &account);
            let result = self
                .transport
                .send(&smtp_config, &credential.secret, &email)
                .await
                .map_err(|error| delivery_error("SMTP send failed", error))?;

            Ok(OutboxSendReceipt {
                provider_message_id: result.message_id,
                accepted_recipients: result.accepted_recipients,
            })
        })
    }
}

pub fn smtp_config_for_provider_account(
    account: &ProviderAccount,
) -> Result<SmtpConfig, OutboxDeliveryError> {
    match account.provider_kind {
        EmailProviderKind::Icloud | EmailProviderKind::Imap => {}
        EmailProviderKind::Gmail => {
            return Err(OutboxDeliveryError::Transport(
                "Gmail send is unavailable until OAuth send scopes are configured".to_owned(),
            ));
        }
        _ => {
            return Err(OutboxDeliveryError::Transport(
                "provider does not support SMTP send".to_owned(),
            ));
        }
    }

    let config = account.config.as_object().ok_or_else(|| {
        OutboxDeliveryError::Transport("provider account config must be a JSON object".to_owned())
    })?;
    let host = config
        .get("smtp_host")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            OutboxDeliveryError::Transport("SMTP config is unavailable for this account".to_owned())
        })?;
    let port = config
        .get("smtp_port")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0 && *value <= u64::from(u16::MAX))
        .ok_or_else(|| {
            OutboxDeliveryError::Transport("SMTP port is unavailable for this account".to_owned())
        })? as u16;
    let username = config
        .get("smtp_username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(account.external_account_id.as_str());
    let tls = config
        .get("smtp_tls")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let starttls = config
        .get("smtp_starttls")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(SmtpConfig::new(host, port, tls, username).starttls(starttls))
}

pub fn outgoing_email_from_outbox_item(
    item: &EmailOutboxItem,
    account: &ProviderAccount,
) -> OutgoingEmail {
    OutgoingEmail {
        from: account.external_account_id.clone(),
        to: item.to_recipients.clone(),
        cc: item.cc_recipients.clone(),
        bcc: item.bcc_recipients.clone(),
        subject: item.subject.clone(),
        body_text: item.body_text.clone(),
        body_html: item.body_html.clone(),
        in_reply_to: optional_metadata_string(&item.metadata, "in_reply_to"),
        references: metadata_string_array(&item.metadata, "references"),
    }
}

fn optional_metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn metadata_string_array(metadata: &Value, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn delivery_error(
    public_message: &'static str,
    error: impl std::fmt::Display,
) -> OutboxDeliveryError {
    tracing::warn!(error = %error, "outbox SMTP delivery failed");
    OutboxDeliveryError::Transport(public_message.to_owned())
}
