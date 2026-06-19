use thiserror::Error;

use crate::domains::decisions::DecisionStoreError;
use crate::domains::mail::messages::MessageProjectionError;
use crate::domains::mail::storage::{AttachmentSafetyScanError, MailStorageError};
use crate::domains::mail::sync::EmailSyncRecordError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::core::OrgCoreError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

#[derive(Debug, Error)]
pub enum EmailSyncPipelineError {
    #[error(transparent)]
    Sync(#[from] EmailSyncRecordError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    Contact(#[from] PersonProjectionError),

    #[error(transparent)]
    PersonMemory(#[from] PersonMemoryError),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    OrganizationCore(#[from] OrgCoreError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid email participant address: {0}")]
    InvalidParticipantEmail(String),
}
