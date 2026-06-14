use thiserror::Error;

use crate::domains::decisions::DecisionStoreError;
use crate::domains::mail::core::CommunicationIngestionError;
use crate::domains::mail::messages::MessageProjectionError;
use crate::domains::tasks::candidates::TaskCandidateError;

#[derive(Debug, Error)]
pub enum WhatsappWebError {
    #[error("invalid WhatsApp Web request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
