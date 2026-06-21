use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionPort,
};
use crate::domains::communications::messages::{
    CommunicationMessageProjectionPort, MessageProjectionError, NewProjectedMessage,
    ProjectedMessage,
};
use crate::platform::communications::{NewRawCommunicationRecord, StoredRawCommunicationRecord};

const TELEGRAM_MESSAGE_RECORD_KIND: &str = "telegram_message";
const WHATSAPP_WEB_MESSAGE_RECORD_KIND: &str = "whatsapp_web_message";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMessageProjection {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Debug, Error)]
pub enum ProviderCommunicationProjectionError {
    #[error("invalid provider communication projection request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
}

pub async fn record_raw_provider_communication(
    pool: PgPool,
    raw: NewRawCommunicationRecord,
) -> Result<StoredRawCommunicationRecord, ProviderCommunicationProjectionError> {
    Ok(CommunicationIngestionPort::new(pool)
        .record_raw_source(&raw)
        .await?)
}

pub async fn record_and_project_telegram_message(
    pool: PgPool,
    raw: NewRawCommunicationRecord,
) -> Result<ProviderMessageProjection, ProviderCommunicationProjectionError> {
    let raw = record_raw_provider_communication(pool.clone(), raw).await?;
    let projected =
        project_raw_telegram_message(&CommunicationMessageProjectionPort::new(pool), &raw).await?;
    Ok(ProviderMessageProjection {
        raw_record_id: raw.raw_record_id,
        message_id: projected.message_id,
    })
}

pub async fn project_raw_telegram_message(
    store: &CommunicationMessageProjectionPort,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, ProviderCommunicationProjectionError> {
    if raw.record_kind != TELEGRAM_MESSAGE_RECORD_KIND {
        return Err(ProviderCommunicationProjectionError::InvalidRequest(
            "raw record kind must be telegram_message".to_owned(),
        ));
    }

    let provider_chat_id = required_payload_string(&raw.payload, "provider_chat_id")?;
    let chat_title = required_payload_string(&raw.payload, "chat_title")?;
    let sender_display_name = required_payload_string(&raw.payload, "sender_display_name")?;
    let text = optional_payload_string(&raw.payload, "text").unwrap_or_default();
    let allow_empty_body_text = text.is_empty() && is_tdlib_raw_payload(raw);
    if text.is_empty() && !allow_empty_body_text {
        return Err(ProviderCommunicationProjectionError::InvalidRequest(
            "payload field `text` is required".to_owned(),
        ));
    }
    let delivery_state = provider_delivery_state(
        &required_payload_string(&raw.payload, "delivery_state")?,
        "Telegram",
    )?;
    let channel_kind = raw
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or("telegram_user");

    let message = NewProjectedMessage {
        message_id: telegram_message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject: chat_title,
        sender: sender_display_name.clone(),
        recipients: vec![provider_chat_id.clone()],
        body_text: text,
        occurred_at: raw.occurred_at,
        channel_kind: channel_kind.to_owned(),
        conversation_id: Some(provider_chat_id),
        sender_display_name: Some(sender_display_name),
        delivery_state,
        message_metadata: raw.payload.clone(),
    };

    if allow_empty_body_text {
        Ok(store
            .upsert_channel_message_allowing_empty_body_text(&message)
            .await?)
    } else {
        Ok(store.upsert_channel_message(&message).await?)
    }
}

pub async fn record_and_project_whatsapp_web_message(
    pool: PgPool,
    raw: NewRawCommunicationRecord,
) -> Result<ProviderMessageProjection, ProviderCommunicationProjectionError> {
    let raw = record_raw_provider_communication(pool.clone(), raw).await?;
    let projected =
        project_raw_whatsapp_web_message(&CommunicationMessageProjectionPort::new(pool), &raw)
            .await?;
    Ok(ProviderMessageProjection {
        raw_record_id: raw.raw_record_id,
        message_id: projected.message_id,
    })
}

pub async fn project_raw_whatsapp_web_message(
    store: &CommunicationMessageProjectionPort,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, ProviderCommunicationProjectionError> {
    if raw.record_kind != WHATSAPP_WEB_MESSAGE_RECORD_KIND {
        return Err(ProviderCommunicationProjectionError::InvalidRequest(
            "raw record kind must be whatsapp_web_message".to_owned(),
        ));
    }

    let provider_chat_id = required_payload_string(&raw.payload, "provider_chat_id")?;
    let chat_title = required_payload_string(&raw.payload, "chat_title")?;
    let sender_display_name = required_payload_string(&raw.payload, "sender_display_name")?;
    let text = required_payload_string(&raw.payload, "text")?;
    let delivery_state = provider_delivery_state(
        &required_payload_string(&raw.payload, "delivery_state")?,
        "WhatsApp Web",
    )?;

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
            delivery_state,
            message_metadata: raw.payload.clone(),
        })
        .await?)
}

fn required_payload_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, ProviderCommunicationProjectionError> {
    optional_payload_string(payload, field)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ProviderCommunicationProjectionError::InvalidRequest(format!(
                "payload field `{field}` is required"
            ))
        })
}

fn optional_payload_string(payload: &Value, field: &'static str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .map(ToOwned::to_owned)
}

fn provider_delivery_state(
    value: &str,
    provider_label: &'static str,
) -> Result<String, ProviderCommunicationProjectionError> {
    match value {
        "received" | "sent" | "send_dry_run" | "send_blocked" => Ok(value.to_owned()),
        _ => Err(ProviderCommunicationProjectionError::InvalidRequest(
            format!("unsupported {provider_label} delivery_state `{value}`"),
        )),
    }
}

fn is_tdlib_raw_payload(raw: &StoredRawCommunicationRecord) -> bool {
    raw.provenance
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        == Some("tdlib")
        && raw.payload.get("tdlib_raw").is_some()
}

fn telegram_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v4:telegram:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

fn whatsapp_web_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v5:whatsapp_web:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

fn stable_hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
