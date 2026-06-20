use chrono::Utc;

use super::super::errors::TelegramError;
use super::super::models::TelegramAttachmentAnchor;
use super::super::rows::provider_channel_message_to_telegram_message;
use super::super::store::TelegramStore;
use crate::platform::communications::{
    ProviderChannelMessageStore, ProviderMessageProjectionObservationContext,
};

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<TelegramAttachmentAnchor, TelegramError> {
        let anchor = ProviderChannelMessageStore::new(self.pool.clone())
            .attachment_anchor(
                account_id,
                provider_chat_id,
                provider_message_id,
                TELEGRAM_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram message `{}` is not projected for chat `{}` and account `{}`",
                    provider_message_id.trim(),
                    provider_chat_id.trim(),
                    account_id.trim()
                ))
            })?;

        Ok(TelegramAttachmentAnchor {
            message_id: anchor.message_id,
            raw_record_id: anchor.raw_record_id,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn update_message_attachment_download_state(
        &self,
        message_id: &str,
        provider_attachment_id: &str,
        tdlib_file_id: i64,
        download_state: &str,
        local_path: Option<&str>,
        size_bytes: Option<i64>,
        content_type: &str,
        filename: Option<&str>,
    ) -> Result<(), TelegramError> {
        let updated = ProviderChannelMessageStore::new(self.pool.clone())
            .update_attachment_download_state(
                message_id,
                provider_attachment_id,
                tdlib_file_id,
                download_state,
                local_path,
                size_bytes,
                content_type,
                filename,
                Utc::now(),
                ProviderMessageProjectionObservationContext {
                    channel_kinds: TELEGRAM_CHANNEL_KINDS,
                    relationship_kind: "telegram_attachment_download_state_update",
                    actor: "telegram.client.messages.attachments.update_message_attachment_download_state",
                },
            )
            .await?;
        if updated.is_none() {
            return Err(TelegramError::InvalidRequest(format!(
                "Telegram message `{message_id}` was not found"
            )));
        }
        let _ = updated.map(provider_channel_message_to_telegram_message);
        Ok(())
    }
}
