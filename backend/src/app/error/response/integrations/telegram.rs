use axum::http::StatusCode;

use crate::integrations::telegram::client::TelegramError;

use super::super::ErrorParts;

pub(super) fn telegram_error_parts(error: TelegramError) -> ErrorParts {
    match error {
        TelegramError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_telegram_request",
            message,
            false,
        ),
        TelegramError::TdlibRuntimeUnavailable(error) => {
            tracing::warn!(error = %error, "Telegram TDLib runtime is unavailable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "telegram_tdlib_runtime_unavailable",
                "Telegram TDLib runtime is not configured on this host".to_owned(),
                false,
            )
        }
        TelegramError::TdlibRuntime(error) => {
            tracing::warn!(error = %error, "Telegram TDLib runtime operation failed");
            (
                StatusCode::BAD_GATEWAY,
                "telegram_tdlib_runtime_error",
                "Telegram TDLib runtime operation failed".to_owned(),
                false,
            )
        }
        TelegramError::QrGeneration(error) => {
            tracing::warn!(error = %error, "Telegram QR generation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_qr_generation_error",
                "Telegram QR generation failed".to_owned(),
                false,
            )
        }
        TelegramError::QrLoginNotFound => (
            StatusCode::NOT_FOUND,
            "telegram_qr_login_not_found",
            "Telegram QR login setup was not found".to_owned(),
            false,
        ),
        TelegramError::Communication(error) => internal(
            error,
            "Telegram communication store operation failed",
            "telegram_store_error",
            "Telegram store operation failed",
        ),
        TelegramError::SecretReference(error) => internal(
            error,
            "Telegram secret reference operation failed",
            "telegram_secret_reference_error",
            "Telegram secret reference operation failed",
        ),
        TelegramError::DatabaseVault(error) => internal(
            error,
            "Telegram database vault operation failed",
            "telegram_secret_vault_error",
            "Telegram secret vault operation failed",
        ),
        TelegramError::HostVault(error) => {
            tracing::warn!(error = %error, "Telegram host vault operation failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "telegram_host_vault_error",
                "Telegram host vault operation failed".to_owned(),
                false,
            )
        }
        TelegramError::MessageProjection(error) => internal(
            error,
            "Telegram message projection failed",
            "telegram_projection_error",
            "Telegram message projection failed",
        ),
        TelegramError::MailStorage(error) => internal(
            error,
            "Telegram mail storage operation failed",
            "telegram_mail_storage_error",
            "Telegram attachment storage operation failed",
        ),
        TelegramError::Decision(error) => internal(
            error,
            "Telegram decision candidate refresh failed",
            "telegram_decision_refresh_error",
            "Telegram decision candidate refresh failed",
        ),
        TelegramError::TaskCandidate(error) => internal(
            error,
            "Telegram task candidate refresh failed",
            "telegram_task_candidate_refresh_error",
            "Telegram task candidate refresh failed",
        ),
        TelegramError::ReviewInboxWorkflow(error) => internal(
            error,
            "Telegram review inbox mirroring failed",
            "telegram_review_inbox_error",
            "Telegram review inbox mirroring failed",
        ),
        TelegramError::ObservationStore(error) => internal(
            error,
            "Telegram observation trail operation failed",
            "telegram_observation_error",
            "Telegram observation trail operation failed",
        ),
        TelegramError::Sqlx(error) => internal(
            error,
            "Telegram database operation failed",
            "telegram_store_error",
            "Telegram store operation failed",
        ),
    }
}

fn internal(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
    message: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        message.to_owned(),
        false,
    )
}
