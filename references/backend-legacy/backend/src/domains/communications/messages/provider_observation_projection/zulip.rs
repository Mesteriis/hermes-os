use super::*;

pub(super) async fn project_zulip_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    ensure_canonical_communication_account(&pool, &raw_record.account_id).await?;
    let provider_message_id = required_payload_str(&raw_record.payload, "provider_message_id")?;
    let body_text = required_payload_str(&raw_record.payload, "content")?;
    let sender_display_name = optional_payload_str(&raw_record.payload, "sender_full_name")
        .or_else(|| optional_payload_str(&raw_record.payload, "sender_email"))
        .unwrap_or_else(|| "Zulip sender".to_owned());
    let delivery_state = optional_payload_str(&raw_record.payload, "delivery_state")
        .unwrap_or_else(|| "received".to_owned());
    let target = zulip_message_target(&raw_record.account_id, &raw_record.payload);
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

    let message = MessageProjectionPort::new(pool)
        .upsert_channel_message(&NewProjectedMessage {
            message_id: zulip_message_id(&raw_record.account_id, &provider_message_id),
            raw_record_id: raw_record.raw_record_id.clone(),
            account_id: raw_record.account_id.clone(),
            provider_record_id: raw_record.provider_record_id.clone(),
            subject: target.subject,
            sender: sender_display_name.clone(),
            recipients: target.recipients,
            body_text,
            occurred_at: raw_record.occurred_at,
            channel_kind: ZULIP_CHANNEL_KIND.to_owned(),
            conversation_id: Some(target.conversation_id),
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

pub(super) async fn project_zulip_reaction_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.reaction" {
        return Ok(None);
    }

    let raw_record = raw_record_for_accepted_signal(pool.clone(), event).await?;
    let message = zulip_target_message(pool.clone(), &raw_record).await?;
    let reaction = optional_payload_str(&raw_record.payload, "emoji_name")
        .or_else(|| optional_payload_str(&raw_record.payload, "emoji_code"))
        .ok_or(CommunicationSignalProjectionError::Message(
            MessageProjectionError::MissingPayloadField("emoji_name"),
        ))?;
    let provider_actor_id = optional_payload_str(&raw_record.payload, "provider_actor_id")
        .or_else(|| optional_payload_str(&raw_record.payload, "sender_email"))
        .unwrap_or_else(|| "unknown".to_owned());
    let sender_display_name = optional_payload_str(&raw_record.payload, "sender_display_name")
        .or_else(|| optional_payload_str(&raw_record.payload, "sender_email"))
        .unwrap_or_else(|| provider_actor_id.clone());
    let reaction_op = optional_payload_str(&raw_record.payload, "reaction_op")
        .unwrap_or_else(|| "add".to_owned());
    let is_active = !matches!(reaction_op.as_str(), "remove" | "delete");
    let observed_at = zulip_observed_at(&raw_record, event);
    let reaction_id = zulip_reaction_id(
        &message.account_id,
        &message.provider_record_id,
        &provider_actor_id,
        &reaction,
    );

    sqlx::query(
        r#"
        INSERT INTO communication_message_reactions (
            reaction_id, message_id, account_id, provider_message_id,
            provider_conversation_id, sender_display_name, reaction, is_active,
            observed_at, source_event, provider_actor_id, metadata, provenance, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())
        ON CONFLICT (reaction_id)
        DO UPDATE SET
            sender_display_name = EXCLUDED.sender_display_name,
            reaction = EXCLUDED.reaction,
            is_active = EXCLUDED.is_active,
            observed_at = EXCLUDED.observed_at,
            source_event = EXCLUDED.source_event,
            provider_actor_id = EXCLUDED.provider_actor_id,
            metadata = EXCLUDED.metadata,
            provenance = EXCLUDED.provenance,
            updated_at = now()
        "#,
    )
    .bind(&reaction_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_record_id)
    .bind(&message.conversation_id)
    .bind(&sender_display_name)
    .bind(&reaction)
    .bind(is_active)
    .bind(observed_at)
    .bind(&event.event_id)
    .bind(&provider_actor_id)
    .bind(json!({
        "provider": "zulip",
        "provider_event_id": raw_record.payload.get("provider_event_id"),
        "provider_event_type": raw_record.payload.get("provider_event_type"),
        "reaction_type": raw_record.payload.get("reaction_type"),
        "reaction_op": reaction_op,
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .bind(json!({
        "provider": "zulip",
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .execute(&pool)
    .await?;

    let projected = projected_message_by_id(&pool, &message.message_id)
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip reaction target `{}` disappeared during projection",
                message.provider_record_id
            ))
        })?;

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed: true,
    }))
}

pub(super) async fn project_zulip_message_update_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.message_update" {
        return Ok(None);
    }

    let raw_record = raw_record_for_accepted_signal(pool.clone(), event).await?;
    let message = zulip_target_message(pool.clone(), &raw_record).await?;
    let body_text = required_payload_str(&raw_record.payload, "content")?;
    let observed_at = zulip_observed_at(&raw_record, event);
    let updated_metadata = merged_zulip_message_metadata(
        &message.message_metadata,
        json!({
            "edited": true,
            "provider": "zulip",
            "provider_event_id": raw_record.payload.get("provider_event_id"),
            "provider_event_type": raw_record.payload.get("provider_event_type"),
            "raw_record_id": &raw_record.raw_record_id,
            "accepted_signal_event_id": &event.event_id,
            "prev_content": raw_record.payload.get("prev_content"),
            "topic": raw_record.payload.get("topic"),
            "prev_topic": raw_record.payload.get("prev_topic"),
            "edit_timestamp": raw_record.payload.get("edit_timestamp"),
        }),
    )?;
    let updated_message = ProviderChannelMessageStore::new(pool.clone())
        .apply_content_update(
            &message.message_id,
            &body_text,
            &updated_metadata,
            observed_at,
            zulip_projection_context("zulip_content_observed"),
        )
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip update target `{}` disappeared during projection",
                message.provider_record_id
            ))
        })?;

    let version_id = zulip_message_version_id(&event.event_id);
    let next_version_number: i32 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(MAX(version_number), 0) + 1
        FROM communication_message_versions
        WHERE message_id = $1
        "#,
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO communication_message_versions (
            version_id, message_id, account_id, provider_message_id,
            provider_conversation_id, version_number, body_text, edited_at,
            source_event, diff_payload, provenance
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (version_id) DO NOTHING
        "#,
    )
    .bind(&version_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_record_id)
    .bind(&message.conversation_id)
    .bind(next_version_number)
    .bind(&body_text)
    .bind(observed_at)
    .bind(&event.event_id)
    .bind(zulip_content_diff(
        Some(message.body_text.as_str()),
        body_text.as_str(),
    ))
    .bind(json!({
        "provider": "zulip",
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .execute(&pool)
    .await?;

    let projected = projected_message_by_id(&pool, &updated_message.message_id)
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip update target `{}` disappeared after projection",
                message.provider_record_id
            ))
        })?;

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed: true,
    }))
}

pub(super) async fn project_zulip_message_delete_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.message_delete" {
        return Ok(None);
    }

    let raw_record = raw_record_for_accepted_signal(pool.clone(), event).await?;
    let message = zulip_target_message(pool.clone(), &raw_record).await?;
    let observed_at = zulip_observed_at(&raw_record, event);
    let tombstone_id = zulip_message_tombstone_id(&event.event_id);

    sqlx::query(
        r#"
        INSERT INTO communication_message_tombstones (
            tombstone_id, message_id, account_id, provider_message_id,
            provider_conversation_id, reason_class, actor_class, observed_at,
            source_event, is_provider_delete, is_local_visible, metadata, provenance
        )
        VALUES ($1, $2, $3, $4, $5, 'deleted_by_provider', 'provider', $6, $7, TRUE, FALSE, $8, $9)
        ON CONFLICT (tombstone_id) DO NOTHING
        "#,
    )
    .bind(&tombstone_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_record_id)
    .bind(&message.conversation_id)
    .bind(observed_at)
    .bind(&event.event_id)
    .bind(json!({
        "provider": "zulip",
        "provider_event_id": raw_record.payload.get("provider_event_id"),
        "provider_event_type": raw_record.payload.get("provider_event_type"),
        "message_type": raw_record.payload.get("message_type"),
        "stream_id": raw_record.payload.get("stream_id"),
        "topic": raw_record.payload.get("topic"),
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .bind(json!({
        "provider": "zulip",
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .execute(&pool)
    .await?;

    let projected = projected_message_by_id(&pool, &message.message_id)
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip delete target `{}` disappeared during projection",
                message.provider_record_id
            ))
        })?;

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed: true,
    }))
}
