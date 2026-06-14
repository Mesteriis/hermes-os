use crate::domains::mail::core::StoredRawCommunicationRecord;
use crate::domains::mail::ingestion::analyze_ingested_message;
use crate::domains::mail::messages::{
    MessageProjectionStore, ProjectedMessage, parse_raw_email_message_from_blob,
    project_parsed_raw_email_message,
};
use crate::domains::mail::storage::{
    AttachmentSafetyScanner, LocalMailBlobStore, MailStorageStore,
};

use super::attachments::project_attachments;
use super::errors::EmailSyncPipelineError;

#[derive(Default)]
pub(crate) struct RawRecordProjectionReport {
    pub(crate) projected_messages: Vec<ProjectedMessage>,
    pub(crate) attachment_blobs_upserted: usize,
    pub(crate) attachments_extracted: usize,
    pub(crate) attachments_not_scanned: usize,
}

pub(crate) async fn project_raw_records(
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
