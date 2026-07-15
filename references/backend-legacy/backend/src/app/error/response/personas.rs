use axum::http::StatusCode;

use crate::domains::personas::api::errors::PersonaProjectionError;

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
        ApiError::InvalidPersonaIdentityReview(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_persona_identity_review",
            message.to_owned(),
            false,
        ),
        ApiError::PersonaIdentityNotFound => (
            StatusCode::NOT_FOUND,
            "persona_identity_candidate_not_found",
            "persona identity candidate was not found".to_owned(),
            false,
        ),
        ApiError::PersonaProjection(error) => projection_error_parts(error),
        ApiError::PersonaIdentity(error) => {
            tracing::error!(
                error = %error,
                "persona identity store operation failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "persona_identity_store_error",
                "persona identity store operation failed".to_owned(),
                false,
            )
        }
        _ => unreachable!("personas response mapper received non-person ApiError"),
    }
}

pub(super) fn projection_error_parts(error: PersonaProjectionError) -> ErrorParts {
    match error {
        PersonaProjectionError::PersonaNotFound(_) => (
            StatusCode::NOT_FOUND,
            "persona_not_found",
            "persona was not found".to_owned(),
            false,
        ),
        PersonaProjectionError::EmptyEmailAddress
        | PersonaProjectionError::InvalidEmailAddress(_)
        | PersonaProjectionError::EmptyAiAgentId
        | PersonaProjectionError::InvalidAiAgentId(_)
        | PersonaProjectionError::EmptyDisplayName
        | PersonaProjectionError::InvalidPersonaType(_) => (
            StatusCode::BAD_REQUEST,
            "invalid_persona_projection",
            error.to_string(),
            false,
        ),
        PersonaProjectionError::Sqlx(error) => {
            tracing::error!(error = %error, "persona projection operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "persona_projection_error",
                "persona projection operation failed".to_owned(),
                false,
            )
        }
        PersonaProjectionError::Observation(error) => {
            tracing::error!(error = %error, "persona projection observation operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "persona_projection_error",
                "persona projection operation failed".to_owned(),
                false,
            )
        }
    }
}
