use sqlx::postgres::PgPool;

use crate::domains::communications::core::StoredRawCommunicationRecord;
use crate::domains::communications::ingestion::analyze_ingested_message;
use crate::domains::communications::messages::{
    CommunicationMessageProjectionPort, ProjectedMessage, parse_raw_email_message_from_blob,
    project_accepted_signal_if_runtime_allows,
};
use crate::domains::communications::storage::{
    CommunicationBlobMetadataPort, LocalCommunicationBlobPort,
};
use crate::domains::signal_hub::dispatch_mail_raw_signal;

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
    pool: &PgPool,
    mail_store: &CommunicationBlobMetadataPort,
    blob_store: &LocalCommunicationBlobPort,
    raw_records: &[StoredRawCommunicationRecord],
) -> Result<RawRecordProjectionReport, EmailSyncPipelineError> {
    let mut report = RawRecordProjectionReport::default();
    let message_store = CommunicationMessageProjectionPort::new(pool.clone());
    for raw_record in raw_records {
        let Some(accepted_event) =
            dispatch_mail_raw_signal(pool.clone(), raw_record, Some(blob_store.root())).await?
        else {
            continue;
        };
        let Some(message) =
            project_accepted_signal_if_runtime_allows(pool.clone(), &accepted_event).await?
        else {
            continue;
        };
        let parsed = parse_raw_email_message_from_blob(blob_store, raw_record).await?;
        let _analysis = analyze_ingested_message(&message_store, &message).await?;
        let attachment_report = project_attachments(
            mail_store,
            blob_store,
            raw_record,
            &message,
            &parsed.attachments,
        )
        .await?;

        report.attachment_blobs_upserted += attachment_report.attachment_blobs_upserted;
        report.attachments_extracted += attachment_report.attachments_extracted;
        report.attachments_not_scanned += attachment_report.attachments_not_scanned;
        report.projected_messages.push(message);
    }
    Ok(report)
}
