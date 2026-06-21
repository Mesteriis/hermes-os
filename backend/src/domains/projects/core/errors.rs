use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("project progress_percent must be between 0 and 100: {0}")]
    InvalidProgress(i32),

    #[error("project must have at least one keyword")]
    NoKeywords,

    #[error(transparent)]
    ProjectLinkReview(#[from] crate::domains::projects::link_reviews::ProjectLinkReviewError),

    #[error("project limit must be positive")]
    InvalidLimit,

    #[error("project message recipients must be a JSON array of strings")]
    InvalidRecipients,
}
