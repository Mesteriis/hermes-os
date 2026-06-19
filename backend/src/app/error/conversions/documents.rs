use super::super::types::ApiError;
use crate::domains::documents::processing::{
    DocumentProcessingCommandServiceError, DocumentProcessingError,
};

impl From<DocumentProcessingError> for ApiError {
    fn from(error: DocumentProcessingError) -> Self {
        Self::DocumentProcessing(error)
    }
}

impl From<DocumentProcessingCommandServiceError> for ApiError {
    fn from(error: DocumentProcessingCommandServiceError) -> Self {
        match error {
            DocumentProcessingCommandServiceError::DocumentProcessing(error) => Self::from(error),
            DocumentProcessingCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "document processing retry observation capture failed");
                ApiError::InvalidCommunicationQuery(
                    "document processing retry observation capture failed",
                )
            }
        }
    }
}
