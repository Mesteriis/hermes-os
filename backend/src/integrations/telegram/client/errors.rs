use crate::platform::observations::ObservationStoreError;
use crate::platform::secrets::{DatabaseEncryptedVaultError, SecretReferenceError};
use crate::vault::HostVaultError;
use crate::workflows::provider_communication_projection::ProviderCommunicationProjectionError;
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

    #[error("Telegram provider account store operation failed: {0}")]
    ProviderAccountStore(String),

    #[error("Telegram media storage operation failed: {0}")]
    MediaStorage(String),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    CommunicationProjection(#[from] ProviderCommunicationProjectionError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
