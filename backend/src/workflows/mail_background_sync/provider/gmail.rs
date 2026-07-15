mod history;
mod message_list;

use serde_json::Value;

use super::super::errors::ProviderSyncError;
use super::super::service::MailBackgroundSyncService;
use super::summary::ProviderSyncSummary;
use super::types::ProviderSyncContext;

impl MailBackgroundSyncService {
    pub(in crate::workflows::mail_background_sync::provider) async fn sync_gmail(
        &self,
        context: ProviderSyncContext<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
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
            self.sync_gmail_message_list_pages(&context, &mut summary, checkpoint_next_page_token)
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

        self.sync_gmail_message_list_pages(&context, &mut summary, None)
            .await?;

        Ok(summary)
    }
}
