use thiserror::Error;

use crate::domains::communications::core::CommunicationIngestionError;
use crate::platform::communications::EmailProviderSyncError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;
use crate::workflows::email_sync_pipeline::EmailSyncPipelineError;
use crate::workflows::graph_projection::GraphProjectionError;

#[derive(Debug, Error)]
pub enum MailSyncError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error("mail sync account was not found")]
    AccountNotFound,

    #[error("mail sync run is already active for account")]
    RunAlreadyActive,

    #[error("mail sync run was not found")]
    RunNotFound,

    #[error("invalid mail sync setting {field}: {message}")]
    InvalidSetting {
        field: &'static str,
        message: &'static str,
    },
}

#[derive(Debug, Error)]
pub(super) enum ProviderSyncError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    ProviderSync(#[from] EmailProviderSyncError),

    #[error(transparent)]
    Pipeline(#[from] EmailSyncPipelineError),

    #[error(transparent)]
    Graph(#[from] GraphProjectionError),

    #[error(transparent)]
    SyncStore(#[from] MailSyncError),
}
