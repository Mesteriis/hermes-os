use crate::domains::decisions::DecisionStoreError;
use crate::domains::mail::core::CommunicationIngestionError;
use crate::domains::mail::messages::MessageProjectionError;
use crate::domains::mail::storage::MailStorageError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::platform::observations::ObservationStoreError;
use crate::platform::secrets::{DatabaseEncryptedVaultError, SecretReferenceError};
use crate::vault::HostVaultError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("invalid Telegram request: {0}")]
    InvalidRequest(String),

    #[error("Telegram TDLib runtime is not available: {0}")]
    TdlibRuntimeUnavailable(String),

    #[error("Telegram TDLib runtime failed: {0}")]
    TdlibRuntime(String),

    #[error("Telegram QR generation failed: {0}")]
    QrGeneration(String),

    #[error("Telegram QR login setup was not found")]
    QrLoginNotFound,

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
