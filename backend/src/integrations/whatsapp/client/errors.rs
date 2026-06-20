use thiserror::Error;

use crate::platform::observations::ObservationStoreError;
use crate::workflows::provider_communication_projection::ProviderCommunicationProjectionError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

#[derive(Debug, Error)]
pub enum WhatsappWebError {
    #[error("invalid WhatsApp Web request: {0}")]
    InvalidRequest(String),

    #[error("WhatsApp Web provider account store operation failed: {0}")]
    ProviderAccountStore(String),

    #[error(transparent)]
    CommunicationProjection(#[from] ProviderCommunicationProjectionError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
