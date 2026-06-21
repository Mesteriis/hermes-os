use chrono::Utc;

use super::super::errors::TelegramError;
use super::super::models::TelegramAttachmentAnchor;
use super::super::store::TelegramStore;
use crate::application::provider_message_state::observe_telegram_attachment_download;
use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderMessageProjectionObservationContext,
};

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

pub(crate) struct TelegramAttachmentDownloadStateUpdate<'a> {
    pub(crate) message_id: &'a str,
    pub(crate) provider_attachment_id: &'a str,
    pub(crate) tdlib_file_id: i64,
    pub(crate) download_state: &'a str,
    pub(crate) local_path: Option<&'a str>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) content_type: &'a str,
    pub(crate) filename: Option<&'a str>,
}

impl TelegramStore {
    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<TelegramAttachmentAnchor, TelegramError> {
        let anchor = self
            .provider_channel_message_store()
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

    pub(crate) async fn update_message_attachment_download_state(
        &self,
        update: TelegramAttachmentDownloadStateUpdate<'_>,
    ) -> Result<(), TelegramError> {
        let updated = observe_telegram_attachment_download(
            self.pool().clone(),
            ProviderAttachmentDownloadStateUpdate {
                message_id: update.message_id,
                provider_attachment_id: update.provider_attachment_id,
                provider_file_id: update.tdlib_file_id,
                download_state: update.download_state,
                local_path: update.local_path,
                size_bytes: update.size_bytes,
                content_type: update.content_type,
                filename: update.filename,
                observed_at: Utc::now(),
                context: ProviderMessageProjectionObservationContext {
                    channel_kinds: TELEGRAM_CHANNEL_KINDS,
                    relationship_kind: "telegram_attachment_download_state_update",
                    actor: "telegram.client.messages.attachments.update_message_attachment_download_state",
                },
            },
        )
        .await?;
        if updated.is_none() {
            return Err(TelegramError::InvalidRequest(format!(
                "Telegram message `{}` was not found",
                update.message_id
            )));
        }
        Ok(())
    }
}
