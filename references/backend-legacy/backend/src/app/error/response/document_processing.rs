use axum::http::StatusCode;

use crate::domains::documents::processing::errors::DocumentProcessingError;

use super::ErrorParts;

pub(super) fn invalid_query_parts(message: &'static str) -> ErrorParts {
    (
        StatusCode::BAD_REQUEST,
        "invalid_document_processing_query",
        message.to_owned(),
        false,
    )
}

pub(super) fn parts(error: DocumentProcessingError) -> ErrorParts {
    let (status, message) = match error {
        DocumentProcessingError::InvalidLimit => (
            StatusCode::BAD_REQUEST,
            "document processing limit must be between 1 and 100",
        ),
        DocumentProcessingError::EmptyField(_)
        | DocumentProcessingError::InvalidStep(_)
        | DocumentProcessingError::InvalidStatus(_)
        | DocumentProcessingError::InvalidArtifactKind(_) => (
            StatusCode::BAD_REQUEST,
            "invalid document processing request payload",
        ),
        DocumentProcessingError::DocumentNotFound | DocumentProcessingError::JobNotFound => (
            StatusCode::NOT_FOUND,
            "document processing job was not found",
        ),
        DocumentProcessingError::RetryRequiresFailedJob => (
            StatusCode::BAD_REQUEST,
            "document processing retry requires a failed job",
        ),
        DocumentProcessingError::RetryCommandConflict => (
            StatusCode::CONFLICT,
            "document processing retry command conflicts with existing event",
        ),
        DocumentProcessingError::EventStore(error) if error.is_unique_violation() => (
            StatusCode::CONFLICT,
            "document processing retry command conflicts with existing event",
        ),
        DocumentProcessingError::ObservationStore(error) => {
            tracing::error!(error = %error, "document processing observation trail failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "document processing observation trail failed",
            )
        }
        _ => {
            tracing::error!(error = %error, "document processing store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "document processing store operation failed",
            )
        }
    };

    (
        status,
        "document_processing_store_error",
        message.to_owned(),
        false,
    )
}
