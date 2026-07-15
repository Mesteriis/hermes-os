use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::DatabaseNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "database_not_configured",
            "DATABASE_URL is not configured".to_owned(),
            false,
        ),
        ApiError::SecretVaultNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "secret_vault_not_configured",
            "host vault must be initialized and unlocked for account setup".to_owned(),
            false,
        ),
        ApiError::HostVault(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "host_vault_error",
            error.to_string(),
            false,
        ),
        ApiError::InvalidEnvelope(error) => (
            StatusCode::BAD_REQUEST,
            "invalid_event_envelope",
            error.to_string(),
            false,
        ),
        ApiError::FailedPrecondition(message) => (
            StatusCode::PRECONDITION_FAILED,
            "failed_precondition",
            message,
            false,
        ),
        ApiError::Audit(error) => {
            tracing::error!(error = %error, "event API audit operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "api_audit_error",
                "API audit operation failed".to_owned(),
                false,
            )
        }
        ApiError::Store(error) if error.is_unique_violation() => (
            StatusCode::CONFLICT,
            "event_conflict",
            "event already exists or violates idempotency constraints".to_owned(),
            false,
        ),
        ApiError::Store(error) => {
            tracing::error!(error = %error, "event API store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "event_store_error",
                "event store operation failed".to_owned(),
                false,
            )
        }
        ApiError::SettingNotFound => (
            StatusCode::NOT_FOUND,
            "setting_not_found",
            "application setting was not found".to_owned(),
            false,
        ),
        ApiError::Settings(error) if error.is_invalid_request() => (
            StatusCode::BAD_REQUEST,
            "invalid_application_setting",
            error.to_string(),
            false,
        ),
        ApiError::Settings(error) => {
            tracing::error!(error = %error, "application settings operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "application_settings_error",
                "application settings operation failed".to_owned(),
                false,
            )
        }
        ApiError::SignalHub(error) if error.is_invalid_request() => (
            StatusCode::BAD_REQUEST,
            "invalid_signal_hub_request",
            error.to_string(),
            false,
        ),
        ApiError::SignalHub(error) if error.is_not_found() => (
            StatusCode::NOT_FOUND,
            "signal_hub_not_found",
            error.to_string(),
            false,
        ),
        ApiError::SignalHub(error) if error.is_failed_precondition() => (
            StatusCode::PRECONDITION_FAILED,
            "signal_hub_precondition_failed",
            error.to_string(),
            false,
        ),
        ApiError::SignalHub(error) => {
            tracing::error!(error = %error, "Signal Hub operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "signal_hub_error",
                "Signal Hub operation failed".to_owned(),
                false,
            )
        }
        _ => unreachable!("platform response mapper received non-platform ApiError"),
    }
}
