use axum::http::StatusCode;

use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;

use super::ErrorParts;

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
        AiError::RunNotFound => (
            StatusCode::NOT_FOUND,
            "ai_run_not_found",
            "AI run was not found".to_owned(),
            false,
        ),
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_runtime_error",
                "AI runtime operation failed".to_owned(),
                false,
            )
        }
        AiError::EventEnvelope(error) => {
            tracing::error!(error = %error, "AI runtime operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_runtime_error",
                "AI runtime operation failed".to_owned(),
                false,
            )
        }
        AiError::EventStore(error) => {
            tracing::error!(error = %error, "AI event store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_runtime_error",
                "AI runtime operation failed".to_owned(),
                false,
            )
        }
        AiError::PersonProjection(error) => {
            tracing::error!(error = %error, "AI persona attribution operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_runtime_error",
                "AI runtime operation failed".to_owned(),
                false,
            )
        }
        AiError::Sqlx(error) => {
            tracing::error!(error = %error, "AI database operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_runtime_error",
                "AI runtime operation failed".to_owned(),
                false,
            )
        }
    }
}

pub(super) fn control_center_error_parts(error: AiControlCenterError) -> ErrorParts {
    match error {
        AiControlCenterError::ProviderNotFound => (
            StatusCode::NOT_FOUND,
            "ai_provider_not_found",
            "AI provider was not found".to_owned(),
            false,
        ),
        AiControlCenterError::ModelNotFound => (
            StatusCode::NOT_FOUND,
            "ai_model_not_found",
            "AI model was not found".to_owned(),
            false,
        ),
        AiControlCenterError::PromptNotFound => (
            StatusCode::NOT_FOUND,
            "ai_prompt_not_found",
            "AI prompt was not found".to_owned(),
            false,
        ),
        AiControlCenterError::PromptVersionNotFound => (
            StatusCode::NOT_FOUND,
            "ai_prompt_version_not_found",
            "AI prompt version was not found".to_owned(),
            false,
        ),
        AiControlCenterError::InvalidRequest(_)
        | AiControlCenterError::EmptyField { .. }
        | AiControlCenterError::SecretLikePayload => (
            StatusCode::BAD_REQUEST,
            "invalid_ai_control_center_request",
            error.to_string(),
            false,
        ),
        AiControlCenterError::HostVault(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "host_vault_error",
            error.to_string(),
            false,
        ),
        AiControlCenterError::SecretReference(error) => {
            tracing::error!(error = %error, "AI control center secret reference operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_secret_reference_error",
                "AI provider secret reference operation failed".to_owned(),
                false,
            )
        }
        AiControlCenterError::Sqlx(error) => {
            tracing::error!(error = %error, "AI control center store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_control_center_error",
                "AI control center operation failed".to_owned(),
                false,
            )
        }
    }
}
