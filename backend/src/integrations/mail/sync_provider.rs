use std::future::Future;
use std::pin::Pin;

use sqlx::postgres::PgPool;

use crate::domains::communications::core::{
    CommunicationProviderSecretBindingStore, ProviderCredentialReader,
};
use crate::integrations::mail::accounts::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::{
    EmailProviderNetworkError, GmailApiClient, GmailFetchOptions, GmailHistoryFetchOptions,
    ImapFetchOptions, ImapNetworkClient,
};
use crate::platform::communications::{
    EmailProviderSyncError, EmailProviderSyncPort, EmailSyncBatch, GmailHistoryFetchRequest,
    GmailMessageListFetchRequest, ImapMessageFetchRequest, ProviderAccountSecretPurpose,
};
use crate::platform::secrets::{ResolvedSecret, SecretReferenceStore};
use crate::vault::HostVault;

#[derive(Clone)]
pub struct LiveEmailProviderSyncPort {
    pool: PgPool,
    vault: HostVault,
    gmail_api_base_url: String,
}

impl LiveEmailProviderSyncPort {
    pub fn new(pool: PgPool, vault: HostVault, gmail_api_base_url: impl Into<String>) -> Self {
        Self {
            pool,
            vault,
            gmail_api_base_url: gmail_api_base_url.into(),
        }
    }

    async fn gmail_access_token(
        &self,
        account_id: &str,
    ) -> Result<ResolvedSecret, EmailProviderSyncError> {
        let binding = CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .get_for_account(account_id, ProviderAccountSecretPurpose::OauthToken)
            .await
            .map_err(EmailProviderSyncError::credential)?
            .ok_or_else(EmailProviderSyncError::missing_credential)?;
        EmailAccountSetupService::new_with_host_vault(
            self.pool.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            self.vault.clone(),
        )
        .refresh_gmail_access_token(&binding.secret_ref)
        .await
        .map_err(EmailProviderSyncError::account_setup)
    }
}

impl EmailProviderSyncPort for LiveEmailProviderSyncPort {
    fn fetch_gmail_message_list<'a>(
        &'a self,
        request: GmailMessageListFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>
    {
        Box::pin(async move {
            let access_token = self.gmail_access_token(&request.account_id).await?;
            let client = GmailApiClient::new(&self.gmail_api_base_url).user_id("me");
            let mut options = GmailFetchOptions::new(request.max_results);
            if let Some(token) = request.page_token {
                options = options.page_token(token);
            }
            client
                .fetch_raw_messages(&access_token, &options)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))
        })
    }

    fn fetch_gmail_history<'a>(
        &'a self,
        request: GmailHistoryFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>
    {
        Box::pin(async move {
            let access_token = self.gmail_access_token(&request.account_id).await?;
            let client = GmailApiClient::new(&self.gmail_api_base_url).user_id("me");
            let mut options =
                GmailHistoryFetchOptions::new(&request.start_history_id, request.max_results);
            if let Some(token) = request.page_token {
                options = options.page_token(token);
            }
            client
                .fetch_history_raw_messages(&access_token, &options)
                .await
                .map_err(|error| {
                    let history_expired = matches!(
                        &error,
                        EmailProviderNetworkError::Http(source)
                            if source.status().is_some_and(|status| status.as_u16() == 404)
                    );
                    EmailProviderSyncError::provider_network(error, history_expired)
                })
        })
    }

    fn fetch_imap_messages<'a>(
        &'a self,
        request: ImapMessageFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>
    {
        Box::pin(async move {
            let credential_reader = ProviderCredentialReader::new(
                CommunicationProviderSecretBindingStore::new(self.pool.clone()),
                SecretReferenceStore::new(self.pool.clone()),
                &self.vault,
            );
            let credential = credential_reader
                .read(
                    &request.account_id,
                    ProviderAccountSecretPurpose::ImapPassword,
                )
                .await
                .map_err(EmailProviderSyncError::credential)?;
            let mut options = ImapFetchOptions::new(
                &request.host,
                request.port,
                request.tls,
                &request.mailbox,
                &request.username,
            )
            .provider_kind(request.provider_kind)
            .max_messages(request.max_messages);
            if let Some(uid) = request.last_seen_uid {
                options = options.last_seen_uid(uid);
            }
            ImapNetworkClient::new()
                .fetch_raw_messages(&credential.secret, &options)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))
        })
    }
}
