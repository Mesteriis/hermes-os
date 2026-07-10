use thiserror::Error;

use crate::ai::hub::AiHubError;
use crate::domains::communications::ai_state::CommunicationAiStateError;
use crate::domains::communications::messages::MessageProjectionError;
use crate::domains::communications::spam_reputation::SenderReputationError;
use crate::domains::review::ReviewInboxError;
use crate::domains::signal_hub::SignalHubError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

#[derive(Debug, Error)]
pub enum EmailIntelligenceError {
    #[error(transparent)]
    Hub(#[from] AiHubError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    CommunicationAiState(#[from] CommunicationAiStateError),

    #[error(transparent)]
    SenderReputation(#[from] SenderReputationError),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    ReviewInboxStorage(#[from] ReviewInboxError),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("failed to parse AI response: {0}")]
    ParseError(String),

    #[error("AI route is not configured: {0}")]
    RouteNotConfigured(String),
}
