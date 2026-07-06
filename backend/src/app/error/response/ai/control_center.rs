use axum::http::StatusCode;

use crate::ai::control_center::AiControlCenterError;

use super::super::ErrorParts;

pub(super) fn control_center_error_parts(error: AiControlCenterError) -> ErrorParts {
    match error {
        AiControlCenterError::ProviderNotFound => {
            not_found("ai_provider_not_found", "AI provider was not found")
        }
        AiControlCenterError::ModelNotFound => {
            not_found("ai_model_not_found", "AI model was not found")
        }
        AiControlCenterError::PromptNotFound => {
            not_found("ai_prompt_not_found", "AI prompt was not found")
        }
        AiControlCenterError::PromptVersionNotFound => not_found(
            "ai_prompt_version_not_found",
            "AI prompt version was not found",
        ),
        AiControlCenterError::InvalidRequest(_)
        | AiControlCenterError::EmptyField { .. }
        | AiControlCenterError::SecretLikePayload => (
            StatusCode::BAD_REQUEST,
            "invalid_ai_control_center_request",
            error.to_string(),
            false,
        ),
        AiControlCenterError::ProviderModelSync(message) => (
            StatusCode::BAD_GATEWAY,
            "ai_provider_model_sync_failed",
            message,
            false,
        ),
        AiControlCenterError::HostVault(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "host_vault_error",
            error.to_string(),
            false,
        ),
        AiControlCenterError::ObservationStore(error) => {
            tracing::error!(error = %error, "AI control center observation trail operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ai_control_center_observation_error",
                "AI control center observation trail operation failed".to_owned(),
                false,
            )
        }
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

fn not_found(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::NOT_FOUND, error, message.to_owned(), false)
}
