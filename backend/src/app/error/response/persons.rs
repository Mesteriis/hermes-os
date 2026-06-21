use axum::http::StatusCode;

use crate::domains::persons::api::PersonProjectionError;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::InvalidPersonaQuery(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_persona_query",
            message.to_owned(),
            false,
        ),
        ApiError::InvalidPersonIdentityReview(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_person_identity_review",
            message.to_owned(),
            false,
        ),
        ApiError::PersonIdentityNotFound => (
            StatusCode::NOT_FOUND,
            "person_identity_candidate_not_found",
            "person identity candidate was not found".to_owned(),
            false,
        ),
        ApiError::PersonProjection(error) => projection_error_parts(error),
        ApiError::PersonIdentity(error) => {
            tracing::error!(
                error = %error,
                "person identity store operation failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_identity_store_error",
                "person identity store operation failed".to_owned(),
                false,
            )
        }
        _ => unreachable!("persons response mapper received non-person ApiError"),
    }
}

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
        PersonProjectionError::Sqlx(error) => {
            tracing::error!(error = %error, "person projection operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_projection_error",
                "person projection operation failed".to_owned(),
                false,
            )
        }
        PersonProjectionError::Observation(error) => {
            tracing::error!(error = %error, "person projection observation operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_projection_error",
                "person projection operation failed".to_owned(),
                false,
            )
        }
    }
}
