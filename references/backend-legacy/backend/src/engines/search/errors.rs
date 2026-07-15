use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Tantivy(#[from] tantivy::TantivyError),

    #[error(transparent)]
    OpenDirectory(#[from] tantivy::directory::error::OpenDirectoryError),

    #[error(transparent)]
    QueryParser(#[from] tantivy::query::QueryParserError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("search limit must be greater than zero")]
    InvalidLimit,

    #[error("search index writer lock was poisoned")]
    WriterLockPoisoned,

    #[error("search result missing stored field: {0}")]
    MissingStoredField(&'static str),
}
