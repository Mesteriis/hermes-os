use serde_json::Value;

use crate::domains::mail::core::StoredRawCommunicationRecord;
use crate::domains::mail::messages::{
    MessageProjectionStore, NewProjectedMessage, ProjectedMessage,
};

use super::constants::WHATSAPP_WEB_MESSAGE_RECORD_KIND;
use super::errors::WhatsappWebError;
use super::ids::whatsapp_web_message_id;
use super::models::WhatsappWebDeliveryState;

pub async fn project_raw_whatsapp_web_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, WhatsappWebError> {
    if raw.record_kind != WHATSAPP_WEB_MESSAGE_RECORD_KIND {
        return Err(WhatsappWebError::InvalidRequest(
            "raw record kind must be whatsapp_web_message".to_owned(),
        ));
    }

    let provider_chat_id = required_payload_string(&raw.payload, "provider_chat_id")?;
    let chat_title = required_payload_string(&raw.payload, "chat_title")?;
    let sender_display_name = required_payload_string(&raw.payload, "sender_display_name")?;
    let text = required_payload_string(&raw.payload, "text")?;
    let delivery_state = WhatsappWebDeliveryState::try_from(required_payload_string(
        &raw.payload,
        "delivery_state",
    )?)?;

    Ok(store
        .upsert_channel_message(&NewProjectedMessage {
            message_id: whatsapp_web_message_id(&raw.account_id, &raw.provider_record_id),
            raw_record_id: raw.raw_record_id.clone(),
            account_id: raw.account_id.clone(),
            provider_record_id: raw.provider_record_id.clone(),
            subject: chat_title,
            sender: sender_display_name.clone(),
            recipients: vec![provider_chat_id.clone()],
            body_text: text,
            occurred_at: raw.occurred_at,
            channel_kind: "whatsapp_web".to_owned(),
            conversation_id: Some(provider_chat_id),
            sender_display_name: Some(sender_display_name),
            delivery_state: delivery_state.as_message_delivery_state().to_owned(),
            message_metadata: raw.payload.clone(),
        })
        .await?)
}

fn required_payload_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, WhatsappWebError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            WhatsappWebError::InvalidRequest(format!("payload field `{field}` is required"))
        })
}
