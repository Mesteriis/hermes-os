use super::*;
use crate::domains::communications::messages::port::MessageProjectionPort;

pub(super) async fn project_telegram_observation(
    pool: PgPool,
    event: &StoredEventEnvelope,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    let payload = &event.event.payload;
    let event_kind = required_str(payload, "event_kind")?;
    let message_id = required_str(payload, "message_id").or_else(|_| {
        event
            .event
            .subject
            .get("message_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ProviderCommunicationMessagePortError::InvalidRequest(
                    "provider observation is missing message_id".to_owned(),
                )
            })
    })?;
    let observed_at = parse_observed_at(payload)?;
    let fact_payload = payload.get("payload").ok_or_else(|| {
        ProviderCommunicationMessagePortError::InvalidRequest(
            "provider observation is missing payload".to_owned(),
        )
    })?;
    let store = ProviderChannelMessageStore::new(pool);
    let context = telegram_projection_context(event_kind);

    match event_kind {
        "metadata_observed" => {
            let metadata = fact_payload.get("message_metadata").ok_or_else(|| {
                ProviderCommunicationMessagePortError::InvalidRequest(
                    "metadata observation is missing message_metadata".to_owned(),
                )
            })?;
            store.apply_metadata(message_id, metadata, context).await
        }
        "delivery_state_observed" => {
            let delivery_state = required_str(fact_payload, "delivery_state")?;
            store
                .set_delivery_state(message_id, delivery_state, observed_at, context)
                .await
        }
        "provider_identity_observed" => {
            let provider_record_id = required_str(fact_payload, "provider_record_id")?;
            store
                .rebind_provider_record_id(message_id, provider_record_id, observed_at, context)
                .await
        }
        "content_observed" => {
            let body_text = required_str(fact_payload, "body_text")?;
            let metadata = fact_payload.get("message_metadata").ok_or_else(|| {
                ProviderCommunicationMessagePortError::InvalidRequest(
                    "content observation is missing message_metadata".to_owned(),
                )
            })?;
            store
                .apply_content_update(message_id, body_text, metadata, observed_at, context)
                .await
        }
        "pinned_state_observed" => {
            let is_pinned = fact_payload
                .get("is_pinned")
                .and_then(Value::as_bool)
                .ok_or_else(|| {
                    ProviderCommunicationMessagePortError::InvalidRequest(
                        "pin observation is missing is_pinned".to_owned(),
                    )
                })?;
            store
                .apply_pinned_state(message_id, is_pinned, observed_at, context)
                .await
        }
        "attachment_download_state_observed" => {
            let update = ProviderAttachmentDownloadStateUpdate {
                message_id,
                provider_attachment_id: required_str(fact_payload, "provider_attachment_id")?,
                communication_attachment_id: optional_str(
                    fact_payload,
                    "communication_attachment_id",
                ),
                provider_file_id: required_i64(fact_payload, "provider_file_id")?,
                download_state: required_str(fact_payload, "download_state")?,
                local_path: optional_str(fact_payload, "local_path"),
                size_bytes: optional_i64(fact_payload, "size_bytes"),
                content_type: required_str(fact_payload, "content_type")?,
                filename: optional_str(fact_payload, "filename"),
                observed_at,
                context,
            };
            store.update_attachment_download_state(update).await
        }
        other => Err(ProviderCommunicationMessagePortError::InvalidRequest(
            format!("unsupported provider observation event kind `{other}`"),
        )),
    }
}
pub(super) async fn project_telegram_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.telegram.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let provider_chat_id = required_payload_str(&raw_record.payload, "provider_chat_id")?;
    let chat_title = required_payload_str(&raw_record.payload, "chat_title")?;
    let sender_display_name = required_payload_str(&raw_record.payload, "sender_display_name")?;
    let body_text = optional_payload_str(&raw_record.payload, "text").unwrap_or_default();
    let delivery_state = required_payload_str(&raw_record.payload, "delivery_state")?;
    let channel_kind = raw_record
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or("telegram_user")
        .trim()
        .to_owned();
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

    let message = NewProjectedMessage {
        message_id: telegram_message_id(&raw_record.account_id, &raw_record.provider_record_id),
        raw_record_id: raw_record.raw_record_id.clone(),
        account_id: raw_record.account_id.clone(),
        provider_record_id: raw_record.provider_record_id.clone(),
        subject: chat_title,
        sender: sender_display_name.clone(),
        recipients: vec![provider_chat_id.clone()],
        body_text,
        occurred_at: raw_record.occurred_at,
        channel_kind,
        conversation_id: Some(provider_chat_id),
        sender_display_name: Some(sender_display_name),
        delivery_state,
        message_metadata: raw_record.payload,
    };

    let projected = MessageProjectionPort::new(pool)
        .upsert_channel_message(&message)
        .await?;

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed,
    }))
}
