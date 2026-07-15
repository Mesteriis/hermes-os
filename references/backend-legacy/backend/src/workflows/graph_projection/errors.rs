use thiserror::Error;

use crate::domains::graph::ports::GraphProjectionPortError;
use crate::domains::projects::ports::ProjectCommandPortError;

#[derive(Debug, Error)]
pub enum GraphProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphProjectionPortError),

    #[error(transparent)]
    Project(#[from] ProjectCommandPortError),

    #[error("message recipients must be a JSON array of strings")]
    InvalidRecipients,

    #[error("{domain} graph projection has invalid entity kind: {value}")]
    InvalidEntityKind { domain: &'static str, value: String },

    #[error("{domain} graph projection has invalid review state: {value}")]
    InvalidReviewState { domain: &'static str, value: String },
}
