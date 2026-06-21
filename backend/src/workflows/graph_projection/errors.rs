use thiserror::Error;

use crate::domains::graph::core::GraphProjectionPortError;
use crate::domains::projects::core::ProjectCommandPortError;

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
}
