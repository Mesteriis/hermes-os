use super::*;

pub(super) async fn project_whatsapp_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.whatsapp.message" {
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
    let body_text = required_payload_str(&raw_record.payload, "text")?;
    let delivery_state = required_payload_str(&raw_record.payload, "delivery_state")?;
    let channel_kind = raw_record
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| matches!(*value, "whatsapp_web" | "whatsapp_business_cloud"))
        .unwrap_or("whatsapp_web")
        .to_owned();
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

    let message = MessageProjectionPort::new(pool)
        .upsert_channel_message(&NewProjectedMessage {
            message_id: whatsapp_web_message_id(
                &raw_record.account_id,
                &raw_record.provider_record_id,
            ),
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
        })
        .await?;

    Ok(Some(AcceptedSignalProjection {
        message,
        message_existed,
    }))
}

pub async fn project_whatsapp_content_observed(
    pool: PgPool,
    message_id: &str,
    body_text: &str,
    metadata: &Value,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    ProviderChannelMessageStore::new(pool)
        .apply_content_update(
            message_id,
            body_text,
            metadata,
            observed_at,
            whatsapp_projection_context("whatsapp_content_observed"),
        )
        .await
}

pub async fn project_whatsapp_delivery_state_observed(
    pool: PgPool,
    message_id: &str,
    delivery_state: &str,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    ProviderChannelMessageStore::new(pool)
        .set_delivery_state(
            message_id,
            delivery_state,
            observed_at,
            whatsapp_projection_context("whatsapp_delivery_state_observed"),
        )
        .await
}
