use super::super::types::ApiError;
use crate::domains::documents::processing::DocumentProcessingError;

impl From<DocumentProcessingError> for ApiError {
    fn from(error: DocumentProcessingError) -> Self {
        Self::DocumentProcessing(error)
    }
}
