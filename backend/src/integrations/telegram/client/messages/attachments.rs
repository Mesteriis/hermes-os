use sqlx::Row;

use super::super::errors::TelegramError;
use super::super::models::TelegramAttachmentAnchor;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<TelegramAttachmentAnchor, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT message_id, raw_record_id
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND provider_record_id = $3
              AND channel_kind IN ('telegram_user', 'telegram_bot')
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT 1
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_chat_id.trim())
        .bind(provider_message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram message `{}` is not projected for chat `{}` and account `{}`",
                provider_message_id.trim(),
                provider_chat_id.trim(),
                account_id.trim()
            ))
        })?;

        Ok(TelegramAttachmentAnchor {
            message_id: row.try_get("message_id")?,
            raw_record_id: row.try_get("raw_record_id")?,
        })
    }
}
