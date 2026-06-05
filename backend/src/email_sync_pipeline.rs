use serde::Serialize;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::communications::{CommunicationIngestionStore, StoredRawCommunicationRecord};
use crate::contacts::{
    ContactProjectionError, ContactProjectionStore, upsert_contacts_from_message_participants,
};
use crate::email_sync::{
    EmailSyncBatch, EmailSyncRecordError, record_email_sync_batch_with_mail_blobs,
};
use crate::mail_storage::LocalMailBlobStore;
use crate::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage,
    project_raw_email_message_from_blob,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailSyncPipelineReport {
    pub imported_records: usize,
    pub raw_blobs_upserted: usize,
    pub projected_messages: usize,
    pub upserted_contacts: usize,
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
    let mail_store = crate::mail_storage::MailStorageStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let contact_store = ContactProjectionStore::new(pool);
    let import_report = record_email_sync_batch_with_mail_blobs(
        &communication_store,
        &mail_store,
        blob_store,
        account_id,
        import_batch_id.as_ref(),
        batch,
    )
    .await?;

    let projected_messages =
        project_raw_records(&message_store, blob_store, &import_report.raw_records).await?;
    let mut participants = Vec::new();
    for message in &projected_messages {
        participants.push(message.sender.clone());
        participants.extend(message.recipients.clone());
    }
    let contacts = upsert_contacts_from_message_participants(&contact_store, &participants).await?;

    Ok(EmailSyncPipelineReport {
        imported_records: import_report.inserted_or_existing_records,
        raw_blobs_upserted: import_report.blobs_upserted,
        projected_messages: projected_messages.len(),
        upserted_contacts: contacts.len(),
        checkpoint_saved: import_report.checkpoint_saved,
    })
}

async fn project_raw_records(
    message_store: &MessageProjectionStore,
    blob_store: &LocalMailBlobStore,
    raw_records: &[StoredRawCommunicationRecord],
) -> Result<Vec<ProjectedMessage>, MessageProjectionError> {
    let mut projected_messages = Vec::new();
    for raw_record in raw_records {
        projected_messages.push(
            project_raw_email_message_from_blob(message_store, blob_store, raw_record).await?,
        );
    }
    Ok(projected_messages)
}

#[derive(Debug, Error)]
pub enum EmailSyncPipelineError {
    #[error(transparent)]
    Sync(#[from] EmailSyncRecordError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    Contact(#[from] ContactProjectionError),
}
