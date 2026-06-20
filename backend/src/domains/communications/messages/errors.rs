use thiserror::Error;

use crate::domains::communications::storage::MailStorageError;
use crate::platform::communications::rfc822::EmailRfc822ParseError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum MessageProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

    #[error(transparent)]
    Rfc822(#[from] EmailRfc822ParseError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error("raw email payload missing required field or wrong type: {0}")]
    MissingPayloadField(&'static str),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error(
        "raw communication record does not match projected message tuple: raw_record_id={raw_record_id}, account_id={account_id}, provider_record_id={provider_record_id}"
    )]
    RawRecordTupleMismatch {
        raw_record_id: String,
        account_id: String,
        provider_record_id: String,
    },

    #[error("stored communication message recipients must be a JSON array of strings")]
    InvalidStoredRecipients,

    #[error("communication message metadata must be a JSON object")]
    InvalidMessageMetadata,

    #[error("unsupported raw blob storage kind: {0}")]
    UnsupportedRawBlobStorageKind(String),

    #[error("message query limit must be between 1 and 5000: {0}")]
    InvalidLimit(i64),

    #[error("invalid communication message cursor")]
    InvalidCursor,

    #[error("communication message was not found")]
    MessageNotFound,

    #[error("invalid workflow state: {0}")]
    InvalidWorkflowState(String),

    #[error("invalid local message state: {0}")]
    InvalidLocalState(String),

    #[error("invalid importance score: {0}, must be 0-100")]
    InvalidImportanceScore(i16),
}
