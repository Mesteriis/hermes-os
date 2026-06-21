use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonIdentityError {
    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("candidate kind is not supported: {0}")]
    InvalidCandidateKind(String),

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("candidate was not found")]
    IdentityCandidateNotFound,

    #[error("payload must be an object")]
    InvalidPayload(String),

    #[error("payload field was missing: {0}")]
    MissingPayloadField(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
