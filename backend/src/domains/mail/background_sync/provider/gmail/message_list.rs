use serde_json::Value;

use crate::integrations::gmail::client::{GmailApiClient, GmailFetchOptions};
use crate::platform::secrets::ResolvedSecret;

use super::super::super::errors::ProviderSyncError;
use super::super::super::models::{MailSyncPhase, ProgressMode, ProgressUpdate};
use super::super::super::service::MailBackgroundSyncService;
use super::super::{ProviderSyncContext, ProviderSyncSummary};

impl MailBackgroundSyncService {
    pub(in crate::domains::mail::background_sync::provider::gmail) async fn sync_gmail_message_list_pages(
        &self,
        client: &GmailApiClient,
        access_token: &ResolvedSecret,
        context: &ProviderSyncContext<'_>,
        summary: &mut ProviderSyncSummary,
        mut page_token: Option<String>,
    ) -> Result<(), ProviderSyncError> {
        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Listing,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let mut options = GmailFetchOptions::new(context.settings.batch_size as u16);
            if let Some(token) = page_token {
                options = options.page_token(token);
            }
            let batch = client.fetch_raw_messages(access_token, &options).await?;
            page_token = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("next_page_token"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let fetched_count = batch.messages.len();
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if page_token.is_none() || fetched_count == 0 {
                break;
            }
        }

        Ok(())
    }
}
