use serde::Serialize;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::communications::{CommunicationIngestionStore, StoredRawCommunicationRecord};
use crate::persons::{
    PersonProjectionError, PersonProjectionStore, upsert_persons_from_message_participants,
};
use crate::email_ingestion::analyze_ingested_message;
use crate::email_rfc822::{ParsedEmailAttachment, ParsedEmailAttachmentDisposition};
use crate::email_sync::{
    EmailSyncBatch, EmailSyncRecordError, record_email_sync_batch_with_mail_blobs,
};
use crate::mail_storage::{
    AttachmentSafetyScanError, AttachmentSafetyScanRequest, AttachmentSafetyScanStatus,
    AttachmentSafetyScanner, LocalMailBlobStore, MailAttachmentDisposition, MailStorageError,
    MailStorageStore, NewMailAttachment, NewMailBlob, NoopAttachmentSafetyScanner,
};
use crate::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage,
    parse_raw_email_message_from_blob, project_parsed_raw_email_message,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailSyncPipelineReport {
    pub imported_records: usize,
    pub raw_blobs_upserted: usize,
    pub projected_messages: usize,
    pub attachment_blobs_upserted: usize,
    pub attachments_extracted: usize,
    pub attachments_not_scanned: usize,
    pub upserted_persons: usize,
    pub checkpoint_saved: bool,
}

pub async fn project_email_sync_batch_with_mail_blobs(
    pool: PgPool,
    blob_store: &LocalMailBlobStore,
    account_id: &str,
    import_batch_id: impl AsRef<str>,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncPipelineReport, EmailSyncPipelineError> {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let mail_store = MailStorageStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let person_store = PersonProjectionStore::new(pool);
    let attachment_scanner = NoopAttachmentSafetyScanner;
    let import_report = record_email_sync_batch_with_mail_blobs(
        &communication_store,
        &mail_store,
        blob_store,
        account_id,
        import_batch_id.as_ref(),
        batch,
    )
    .await?;

    let projection_report = project_raw_records(
        &message_store,
        &mail_store,
        blob_store,
        &import_report.raw_records,
        &attachment_scanner,
    )
    .await?;
    let mut participants = Vec::new();
    for message in &projection_report.projected_messages {
        participants.push(message.sender.clone());
        participants.extend(message.recipients.clone());
    }
    let persons = upsert_persons_from_message_participants(&person_store, &participants).await?;

    Ok(EmailSyncPipelineReport {
        imported_records: import_report.inserted_or_existing_records,
        raw_blobs_upserted: import_report.blobs_upserted,
        projected_messages: projection_report.projected_messages.len(),
        attachment_blobs_upserted: projection_report.attachment_blobs_upserted,
        attachments_extracted: projection_report.attachments_extracted,
        attachments_not_scanned: projection_report.attachments_not_scanned,
        upserted_persons: persons.len(),
        checkpoint_saved: import_report.checkpoint_saved,
    })
}

#[derive(Default)]
struct RawRecordProjectionReport {
    projected_messages: Vec<ProjectedMessage>,
    attachment_blobs_upserted: usize,
    attachments_extracted: usize,
    attachments_not_scanned: usize,
}

async fn project_raw_records(
    message_store: &MessageProjectionStore,
    mail_store: &MailStorageStore,
    blob_store: &LocalMailBlobStore,
    raw_records: &[StoredRawCommunicationRecord],
    attachment_scanner: &impl AttachmentSafetyScanner,
) -> Result<RawRecordProjectionReport, EmailSyncPipelineError> {
    let mut report = RawRecordProjectionReport::default();
    for raw_record in raw_records {
        let parsed = parse_raw_email_message_from_blob(blob_store, raw_record).await?;
        let message = project_parsed_raw_email_message(message_store, raw_record, &parsed).await?;
        let _analysis = analyze_ingested_message(message_store, &message).await?;
        let attachment_report = project_attachments(
            mail_store,
            blob_store,
            raw_record,
            &message,
            &parsed.attachments,
            attachment_scanner,
        )
        .await?;

        report.attachment_blobs_upserted += attachment_report.attachment_blobs_upserted;
        report.attachments_extracted += attachment_report.attachments_extracted;
        report.attachments_not_scanned += attachment_report.attachments_not_scanned;
        report.projected_messages.push(message);
    }
    Ok(report)
}

#[derive(Default)]
struct AttachmentProjectionReport {
    attachment_blobs_upserted: usize,
    attachments_extracted: usize,
    attachments_not_scanned: usize,
}

async fn project_attachments(
    mail_store: &MailStorageStore,
    blob_store: &LocalMailBlobStore,
    raw_record: &StoredRawCommunicationRecord,
    message: &ProjectedMessage,
    attachments: &[ParsedEmailAttachment],
    attachment_scanner: &impl AttachmentSafetyScanner,
) -> Result<AttachmentProjectionReport, EmailSyncPipelineError> {
    let mut report = AttachmentProjectionReport::default();

    for parsed_attachment in attachments {
        let local_blob = blob_store.put_blob(&parsed_attachment.body_bytes).await?;
        let blob = mail_store
            .upsert_blob(
                &NewMailBlob::from_local_blob(&local_blob)
                    .content_type(&parsed_attachment.content_type),
            )
            .await?;
        let scan_report = attachment_scanner.scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: &parsed_attachment.provider_attachment_id,
            filename: parsed_attachment.filename.as_deref(),
            content_type: &parsed_attachment.content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &blob.sha256,
            storage_kind: &blob.storage_kind,
            storage_path: &blob.storage_path,
            bytes: &parsed_attachment.body_bytes,
        })?;
        let scan_status = scan_report.status;

        let mut attachment = NewMailAttachment::new(
            &message.message_id,
            &raw_record.raw_record_id,
            &blob.blob_id,
            &parsed_attachment.provider_attachment_id,
            &parsed_attachment.content_type,
            local_blob.size_bytes,
            &blob.sha256,
        )
        .disposition(mail_attachment_disposition(parsed_attachment.disposition))
        .scan_report(scan_report);

        if let Some(filename) = &parsed_attachment.filename {
            attachment = attachment.filename(filename);
        }

        mail_store.upsert_attachment(&attachment).await?;
        report.attachment_blobs_upserted += 1;
        report.attachments_extracted += 1;
        if scan_status == AttachmentSafetyScanStatus::NotScanned {
            report.attachments_not_scanned += 1;
        }
    }

    Ok(report)
}

fn mail_attachment_disposition(
    disposition: ParsedEmailAttachmentDisposition,
) -> MailAttachmentDisposition {
    match disposition {
        ParsedEmailAttachmentDisposition::Attachment => MailAttachmentDisposition::Attachment,
        ParsedEmailAttachmentDisposition::Inline => MailAttachmentDisposition::Inline,
        ParsedEmailAttachmentDisposition::Unknown => MailAttachmentDisposition::Unknown,
    }
}

#[derive(Debug, Error)]
pub enum EmailSyncPipelineError {
    #[error(transparent)]
    Sync(#[from] EmailSyncRecordError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    Contact(#[from] PersonProjectionError),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),
}
