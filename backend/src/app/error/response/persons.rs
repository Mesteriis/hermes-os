use axum::http::StatusCode;

use crate::domains::persons::api::PersonProjectionError;

use super::ErrorParts;

pub(super) fn projection_error_parts(error: PersonProjectionError) -> ErrorParts {
    match error {
        PersonProjectionError::PersonNotFound(_) => (
            StatusCode::NOT_FOUND,
            "person_not_found",
            "person was not found".to_owned(),
            false,
        ),
        PersonProjectionError::EmptyEmailAddress
        | PersonProjectionError::InvalidEmailAddress(_)
        | PersonProjectionError::EmptyAiAgentId
        | PersonProjectionError::InvalidAiAgentId(_)
        | PersonProjectionError::EmptyDisplayName
        | PersonProjectionError::InvalidPersonaType(_) => (
            StatusCode::BAD_REQUEST,
            "invalid_person_projection",
            error.to_string(),
            false,
        ),
        PersonProjectionError::Graph(error) => {
            tracing::error!(error = %error, "person graph projection operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_projection_error",
                "person projection operation failed".to_owned(),
                false,
            )
        }
        PersonProjectionError::Sqlx(error) => {
            tracing::error!(error = %error, "person projection operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_projection_error",
                "person projection operation failed".to_owned(),
                false,
            )
        }
    }
}
