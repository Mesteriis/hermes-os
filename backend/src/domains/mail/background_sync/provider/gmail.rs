mod history;
mod message_list;

use serde_json::Value;

use crate::domains::mail::accounts::EmailAccountSetupService;
use crate::domains::mail::core::ProviderAccountSecretPurpose;
use crate::integrations::gmail::client::GmailApiClient;
use crate::platform::secrets::SecretReferenceStore;

use super::super::errors::ProviderSyncError;
use super::super::service::MailBackgroundSyncService;
use super::{ProviderSyncContext, ProviderSyncSummary};

impl MailBackgroundSyncService {
    pub(in crate::domains::mail::background_sync::provider) async fn sync_gmail(
        &self,
        context: ProviderSyncContext<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        let binding = context
            .communication_store
            .provider_account_secret_binding(
                &context.account.account_id,
                ProviderAccountSecretPurpose::OauthToken,
            )
            .await?
            .ok_or(ProviderSyncError::MissingCredential)?;
        let account_setup = EmailAccountSetupService::new_with_host_vault(
            context.communication_store.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            self.vault.clone(),
        );
        let access_token = account_setup
            .refresh_gmail_access_token(&binding.secret_ref)
            .await?;
        let client = GmailApiClient::new(&self.gmail_api_base_url).user_id("me");
        let mut summary = ProviderSyncSummary::default();
        let checkpoint_next_page_token = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("next_page_token"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let checkpoint_page_kind = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("page_kind"))
            .and_then(Value::as_str);

        if checkpoint_next_page_token.is_some() && checkpoint_page_kind != Some("history") {
            self.sync_gmail_message_list_pages(
                &client,
                &access_token,
                &context,
                &mut summary,
                checkpoint_next_page_token,
            )
            .await?;
            return Ok(summary);
        }

        if let Some(history_id) = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("history_id"))
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            let start_history_id = context
                .checkpoint_before
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("start_history_id"))
                .and_then(Value::as_str)
                .unwrap_or(&history_id)
                .to_owned();
            let history_page_token = if checkpoint_page_kind == Some("history") {
                checkpoint_next_page_token
            } else {
                None
            };
            let history_expired = self
                .sync_gmail_history_pages(
                    &client,
                    &access_token,
                    &context,
                    &mut summary,
                    &start_history_id,
                    history_page_token,
                )
                .await?;
            if !history_expired {
                return Ok(summary);
            }
        }

        self.sync_gmail_message_list_pages(&client, &access_token, &context, &mut summary, None)
            .await?;

        Ok(summary)
    }
}
