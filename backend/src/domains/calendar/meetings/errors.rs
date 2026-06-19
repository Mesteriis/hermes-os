use thiserror::Error;

use crate::domains::decisions::DecisionStoreError;
use crate::domains::obligations::ObligationStoreError;
use crate::platform::observations::ObservationStoreError;
use crate::workflows::review_mirror::ReviewMirrorError;

#[derive(Debug, Error)]
pub enum MeetingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Decision(#[from] DecisionStoreError),
    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),
    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
    #[error("not found")]
    NotFound,
}
