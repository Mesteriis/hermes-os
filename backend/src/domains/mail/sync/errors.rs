use thiserror::Error;

use crate::domains::mail::core::CommunicationIngestionError;
use crate::domains::mail::storage::MailStorageError;

#[derive(Debug, Error)]
pub enum EmailSyncPlanError {
    #[error("invalid provider config field {field}: {message}")]
    InvalidProviderConfig {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider account config must not contain secret-like key: {key}")]
    SecretLikeConfigKey { key: String },
}

#[derive(Debug, Error)]
pub enum EmailSyncRecordError {
    #[error(transparent)]
    Plan(#[from] EmailSyncPlanError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

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
