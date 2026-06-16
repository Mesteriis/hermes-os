use super::super::errors::TelegramError;
use super::super::models::TelegramMessage;
use super::super::rows::row_to_telegram_message;
use super::super::store::TelegramStore;
use super::super::validation::validate_message_list_limit;

impl TelegramStore {
    pub async fn message_by_id(
        &self,
        message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE message_id = $1
              AND channel_kind IN ('telegram_user', 'telegram_bot')
            "#,
        )
        .bind(message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_message).transpose()
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = validate_message_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE channel_kind IN ('telegram_user', 'telegram_bot')
              AND ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR conversation_id = $2)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_message).collect()
    }
}
