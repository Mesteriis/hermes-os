use super::*;

pub(super) async fn project_message(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.mail.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;
    let message_store = MessageProjectionStore::new(pool);

    let message = if raw_record.payload.get("raw_blob_storage_path").is_some() {
        let blob_store = LocalCommunicationBlobStore::new(mail_blob_root_from_event(event));
        project_raw_email_message_from_blob(&message_store, &blob_store, &raw_record).await?
    } else {
        project_raw_email_message(&message_store, &raw_record).await?
    };

    Ok(Some(AcceptedSignalProjection {
        message,
        message_existed,
    }))
}
