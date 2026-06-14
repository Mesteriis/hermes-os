use thiserror::Error;

use crate::domains::decisions::DecisionStoreError;
use crate::domains::mail::messages::MessageProjectionError;
use crate::domains::mail::storage::{AttachmentSafetyScanError, MailStorageError};
use crate::domains::mail::sync::EmailSyncRecordError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::relationships::RelationshipStoreError;
use crate::domains::tasks::candidates::TaskCandidateError;

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

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid email participant address: {0}")]
    InvalidParticipantEmail(String),
}
