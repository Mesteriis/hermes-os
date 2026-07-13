use axum::http::StatusCode;

use crate::platform::calls::errors::CallError;

use super::super::ErrorParts;

pub(super) fn call_error_parts(error: CallError) -> ErrorParts {
    match error {
        CallError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_call_request",
            message,
            false,
        ),
        CallError::Sqlx(error) => {
            tracing::error!(error = %error, "call intelligence database operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "call_store_error",
                "call intelligence operation failed".to_owned(),
                false,
            )
        }
    }
}
