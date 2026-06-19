use thiserror::Error;

use crate::domains::decisions::DecisionStoreError;
use crate::domains::relationships::RelationshipStoreError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;
use crate::workflows::review_mirror::ReviewMirrorError;

#[derive(Debug, Error)]
pub enum ProjectLinkReviewError {
    #[error("project_id does not exist")]
    ProjectNotFound,

    #[error("project link target does not exist")]
    TargetNotFound,

    #[error("target_kind must be one of message or document")]
    InvalidTargetKind(String),

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("field missing from payload: {0}")]
    MissingPayloadField(String),

    #[error("field must be a string: {0}")]
    InvalidPayload(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}
