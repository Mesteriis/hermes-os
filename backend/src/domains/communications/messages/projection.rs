use serde_json::json;

use crate::domains::communications::core::StoredRawCommunicationRecord;
use crate::domains::communications::storage::LocalCommunicationBlobStore;
use crate::platform::communications::rfc822::{
    ParsedCommunicationSourceMessage, parse_rfc822_message,
};

use super::errors::MessageProjectionError;
use super::ids::message_id;
use super::models::{NewProjectedMessage, ProjectedMessage};
use super::payload::{required_payload_string, required_payload_string_array};
use super::store::MessageProjectionStore;

pub async fn project_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let subject = required_payload_string(&raw.payload, "subject")?;
    let sender = required_payload_string(&raw.payload, "from")?;
    let recipients = required_payload_string_array(&raw.payload, "to")?;
    let body_text = required_payload_string(&raw.payload, "body_text")?;
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject,
        sender: sender.clone(),
        recipients,
        body_text,
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some(sender.clone()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    store.upsert_message(&message).await
}

pub async fn project_raw_email_message_from_blob(
    store: &MessageProjectionStore,
    blob_store: &LocalCommunicationBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let parsed = parse_raw_email_message_from_blob(blob_store, raw).await?;
    project_parsed_raw_email_message(store, raw, &parsed).await
}

pub async fn parse_raw_email_message_from_blob(
    blob_store: &LocalCommunicationBlobStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ParsedCommunicationSourceMessage, MessageProjectionError> {
    let storage_kind = required_payload_string(&raw.payload, "raw_blob_storage_kind")?;
    if storage_kind != "local_fs" {
        return Err(MessageProjectionError::UnsupportedRawBlobStorageKind(
            storage_kind,
        ));
    }
    let storage_path = required_payload_string(&raw.payload, "raw_blob_storage_path")?;
    let bytes = blob_store.read_blob(&storage_path).await?;
    Ok(parse_rfc822_message(&bytes)?)
}

pub async fn project_parsed_raw_email_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
    parsed: &ParsedCommunicationSourceMessage,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let message = NewProjectedMessage {
        message_id: message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject: parsed.subject.clone(),
        sender: parsed.from.clone(),
        recipients: parsed.to.clone(),
        body_text: parsed.body_text.clone(),
        occurred_at: raw.occurred_at,
        channel_kind: "email".to_owned(),
        conversation_id: None,
        sender_display_name: Some(parsed.from.clone()),
        delivery_state: "received".to_owned(),
        message_metadata: json!({}),
    };

    store.upsert_message(&message).await
}
