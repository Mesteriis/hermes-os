use serde_json::{Map, Value, json};

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
        delivery_state: raw_email_delivery_state(raw),
        message_metadata: raw_email_message_metadata(raw),
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
        delivery_state: raw_email_delivery_state(raw),
        message_metadata: raw_email_message_metadata(raw),
    };

    store.upsert_message(&message).await
}

fn raw_email_message_metadata(raw: &StoredRawCommunicationRecord) -> Value {
    let mut metadata = Map::new();
    copy_raw_email_metadata_field(raw, &mut metadata, "provider");
    copy_raw_email_metadata_field(raw, &mut metadata, "transport");
    copy_raw_email_metadata_field(raw, &mut metadata, "mailbox");
    copy_raw_email_metadata_field(raw, &mut metadata, "uid");
    copy_raw_email_metadata_field(raw, &mut metadata, "uid_validity");
    Value::Object(metadata)
}

fn raw_email_delivery_state(raw: &StoredRawCommunicationRecord) -> String {
    if raw_email_mailbox(raw).is_some_and(|mailbox| mailbox_is_sent(&mailbox)) {
        return "sent".to_owned();
    }

    "received".to_owned()
}

fn raw_email_mailbox(raw: &StoredRawCommunicationRecord) -> Option<String> {
    raw.payload
        .get("mailbox")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|mailbox| !mailbox.is_empty())
        .map(str::to_owned)
}

fn mailbox_is_sent(mailbox: &str) -> bool {
    let normalized = mailbox.to_ascii_lowercase();
    normalized == "sent" || normalized.contains("sent messages") || normalized.contains("sent mail")
}

fn copy_raw_email_metadata_field(
    raw: &StoredRawCommunicationRecord,
    metadata: &mut Map<String, Value>,
    field: &'static str,
) {
    if let Some(value) = raw.payload.get(field) {
        metadata.insert(field.to_owned(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::{
        StoredRawCommunicationRecord, raw_email_delivery_state, raw_email_message_metadata,
    };

    #[test]
    fn raw_email_message_metadata_preserves_provider_mailbox_identity() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_mailbox_metadata".to_owned(),
            observation_id: "observation:v1:raw-mailbox-metadata".to_owned(),
            account_id: "acct_mailbox_metadata".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "imap:Junk:42".to_owned(),
            source_fingerprint: "sha256:raw-mailbox-metadata".to_owned(),
            import_batch_id: "batch_mailbox_metadata".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "provider": "icloud",
                "transport": "imap",
                "mailbox": "Junk",
                "uid": 42,
                "uid_validity": 7,
                "raw_blob_storage_path": "/private/mail/blob"
            }),
            provenance: json!({"source": "unit_test"}),
        };

        let metadata = raw_email_message_metadata(&raw);

        assert_eq!(metadata["provider"], json!("icloud"));
        assert_eq!(metadata["transport"], json!("imap"));
        assert_eq!(metadata["mailbox"], json!("Junk"));
        assert_eq!(metadata["uid"], json!(42));
        assert_eq!(metadata["uid_validity"], json!(7));
        assert!(metadata.get("raw_blob_storage_path").is_none());
    }

    #[test]
    fn raw_email_delivery_state_marks_sent_mailboxes_as_sent() {
        let raw = StoredRawCommunicationRecord {
            raw_record_id: "raw_sent_mailbox".to_owned(),
            observation_id: "observation:v1:raw-sent-mailbox".to_owned(),
            account_id: "acct_sent_mailbox".to_owned(),
            record_kind: "email_message".to_owned(),
            provider_record_id: "imap:Sent Messages:42".to_owned(),
            source_fingerprint: "sha256:raw-sent-mailbox".to_owned(),
            import_batch_id: "batch_sent_mailbox".to_owned(),
            occurred_at: Some(Utc::now()),
            captured_at: Utc::now(),
            payload: json!({
                "mailbox": "Sent Messages"
            }),
            provenance: json!({"source": "unit_test"}),
        };

        assert_eq!(raw_email_delivery_state(&raw), "sent");
    }
}
