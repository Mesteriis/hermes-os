use thiserror::Error;

use crate::domains::graph::core::GraphStoreError;
use crate::domains::projects::core::ProjectStoreError;

#[derive(Debug, Error)]
pub enum GraphProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),

    #[error(transparent)]
    Project(#[from] ProjectStoreError),

    #[error("message recipients must be a JSON array of strings")]
    InvalidRecipients,
}
