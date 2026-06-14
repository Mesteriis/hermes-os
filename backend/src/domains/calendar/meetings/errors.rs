use thiserror::Error;

use crate::domains::decisions::DecisionStoreError;
use crate::domains::obligations::ObligationStoreError;

#[derive(Debug, Error)]
pub enum MeetingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Decision(#[from] DecisionStoreError),
    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),
    #[error("not found")]
    NotFound,
}
