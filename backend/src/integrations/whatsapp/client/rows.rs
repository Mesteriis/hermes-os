use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::WhatsappWebError;
use super::models::{WhatsappWebMessage, WhatsappWebSession};

pub(crate) fn row_to_whatsapp_web_session(
    row: PgRow,
) -> Result<WhatsappWebSession, WhatsappWebError> {
    Ok(WhatsappWebSession {
        session_id: row.try_get("session_id")?,
        account_id: row.try_get("account_id")?,
        device_name: row.try_get("device_name")?,
        companion_runtime: row.try_get("companion_runtime")?,
        link_state: row.try_get("link_state")?,
        local_state_path: row.try_get("local_state_path")?,
        last_sync_at: row.try_get("last_sync_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(crate) fn row_to_whatsapp_web_message(
    row: PgRow,
) -> Result<WhatsappWebMessage, WhatsappWebError> {
    Ok(WhatsappWebMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_record_id")?,
        provider_chat_id: row.try_get("conversation_id")?,
        chat_title: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        sender_display_name: row.try_get("sender_display_name")?,
        text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        delivery_state: row.try_get("delivery_state")?,
        metadata: row.try_get("message_metadata")?,
    })
}
