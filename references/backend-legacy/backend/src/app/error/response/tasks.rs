use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::InvalidTaskQuery(message) => bad_request("invalid_task_query", message),
        _ => unreachable!("tasks response mapper received non-task ApiError"),
    }
}

fn bad_request(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::BAD_REQUEST, error, message.to_owned(), false)
}
