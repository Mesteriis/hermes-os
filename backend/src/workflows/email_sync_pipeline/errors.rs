use thiserror::Error;

use crate::domains::communications::core::CommunicationIngestionError;
use crate::domains::communications::messages::MessageProjectionError;
use crate::domains::communications::storage::{
    AttachmentSafetyScanError, CommunicationStorageError,
};
use crate::domains::decisions::DecisionReviewPortError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::core::OrgCoreError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::relationships::RelationshipReviewPortError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

#[derive(Debug, Error)]
pub enum EmailSyncRecordError {
    #[error("email sync record field must not be empty: {0}")]
    EmptyField(&'static str),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error("email sync payload must be a JSON object before raw blob projection")]
    InvalidRawPayloadObject,

    #[error("email sync payload missing provider raw field: {field}")]
    MissingRawPayloadField { field: &'static str },

    #[error("email sync payload field {field} is invalid base64: {source}")]
    InvalidRawPayloadBase64 {
        field: &'static str,
        #[source]
        source: base64::DecodeError,
    },

    #[error("email sync does not support provider kind: {0}")]
    UnsupportedProviderKind(String),
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
    PersonMemory(#[from] PersonMemoryError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),

    #[error(transparent)]
    Decision(#[from] DecisionReviewPortError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    OrganizationCore(#[from] OrgCoreError),

    #[error(transparent)]
    Relationship(#[from] RelationshipReviewPortError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid email participant address: {0}")]
    InvalidParticipantEmail(String),
}
