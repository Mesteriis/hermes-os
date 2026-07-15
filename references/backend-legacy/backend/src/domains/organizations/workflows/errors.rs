use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrgWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::errors::TimelineEngineError),
}
