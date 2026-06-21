use axum::http::StatusCode;

use crate::ai::core::AiError;

use super::super::ErrorParts;

pub(super) fn ai_run_not_found_parts() -> ErrorParts {
    (
        StatusCode::NOT_FOUND,
        "ai_run_not_found",
        "AI run was not found".to_owned(),
        false,
    )
}

pub(super) fn ai_error_parts(error: AiError) -> ErrorParts {
    match error {
        AiError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_ai_request",
            message.to_owned(),
            false,
        ),
        AiError::UnknownAgent(agent_id) => (
            StatusCode::BAD_REQUEST,
            "unknown_ai_agent",
            format!("unknown AI agent `{agent_id}`"),
            false,
        ),
        AiError::RunNotFound => ai_run_not_found_parts(),
        AiError::Runtime(error) => (
            StatusCode::BAD_GATEWAY,
            "ai_runtime_error",
            error.to_string(),
            false,
        ),
        AiError::InvalidEmbeddingDimension { .. } => (
            StatusCode::BAD_GATEWAY,
            "invalid_embedding_dimension",
            "embedding provider returned an unexpected vector dimension".to_owned(),
            false,
        ),
        AiError::Json(error) => (
            StatusCode::BAD_GATEWAY,
            "ai_provider_json_error",
            error.to_string(),
            false,
        ),
        AiError::InvalidSourceKind(value) => {
            tracing::error!(source_kind = %value, "AI runtime saw invalid semantic source kind");
            internal_runtime_error()
        }
        AiError::EventEnvelope(error) => {
            tracing::error!(error = %error, "AI runtime operation failed");
            internal_runtime_error()
        }
        AiError::EventStore(error) => {
            tracing::error!(error = %error, "AI event store operation failed");
            internal_runtime_error()
        }
        AiError::PersonaAttribution(error) => {
            tracing::error!(error = %error, "AI persona attribution operation failed");
            internal_runtime_error()
        }
        AiError::PersonaAttributionUnavailable => {
            tracing::error!("AI persona attribution port was not configured");
            internal_runtime_error()
        }
        AiError::ReviewInboxWorkflow(error) => {
            tracing::error!(error = %error, "AI review inbox mirroring failed");
            internal_runtime_error()
        }
        AiError::ObservationStore(error) => {
            tracing::error!(error = %error, "AI observation trail operation failed");
            internal_runtime_error()
        }
        AiError::Sqlx(error) => {
            tracing::error!(error = %error, "AI database operation failed");
            internal_runtime_error()
        }
    }
}

fn internal_runtime_error() -> ErrorParts {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "ai_runtime_error",
        "AI runtime operation failed".to_owned(),
        false,
    )
}
