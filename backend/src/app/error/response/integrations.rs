use axum::http::StatusCode;

use crate::engines::automation::AutomationError;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::whatsapp::client::WhatsappWebError;
use crate::platform::calls::CallError;

use super::ErrorParts;

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
        TelegramError::Communication(error) => {
            tracing::error!(error = %error, "Telegram communication store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_store_error",
                "Telegram store operation failed".to_owned(),
                false,
            )
        }
        TelegramError::SecretReference(error) => {
            tracing::error!(error = %error, "Telegram secret reference operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_secret_reference_error",
                "Telegram secret reference operation failed".to_owned(),
                false,
            )
        }
        TelegramError::DatabaseVault(error) => {
            tracing::error!(error = %error, "Telegram database vault operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_secret_vault_error",
                "Telegram secret vault operation failed".to_owned(),
                false,
            )
        }
        TelegramError::HostVault(error) => {
            tracing::warn!(error = %error, "Telegram host vault operation failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "telegram_host_vault_error",
                "Telegram host vault operation failed".to_owned(),
                false,
            )
        }
        TelegramError::MessageProjection(error) => {
            tracing::error!(error = %error, "Telegram message projection failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_projection_error",
                "Telegram message projection failed".to_owned(),
                false,
            )
        }
        TelegramError::Decision(error) => {
            tracing::error!(error = %error, "Telegram decision candidate refresh failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_decision_refresh_error",
                "Telegram decision candidate refresh failed".to_owned(),
                false,
            )
        }
        TelegramError::TaskCandidate(error) => {
            tracing::error!(error = %error, "Telegram task candidate refresh failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_task_candidate_refresh_error",
                "Telegram task candidate refresh failed".to_owned(),
                false,
            )
        }
        TelegramError::Sqlx(error) => {
            tracing::error!(error = %error, "Telegram database operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_store_error",
                "Telegram store operation failed".to_owned(),
                false,
            )
        }
    }
}

pub(super) fn whatsapp_web_error_parts(error: WhatsappWebError) -> ErrorParts {
    match error {
        WhatsappWebError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_whatsapp_web_request",
            message,
            false,
        ),
        WhatsappWebError::Communication(error) => {
            tracing::error!(error = %error, "WhatsApp Web communication store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "whatsapp_web_store_error",
                "WhatsApp Web store operation failed".to_owned(),
                false,
            )
        }
        WhatsappWebError::MessageProjection(error) => {
            tracing::error!(error = %error, "WhatsApp Web message projection failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "whatsapp_web_projection_error",
                "WhatsApp Web message projection failed".to_owned(),
                false,
            )
        }
        WhatsappWebError::Decision(error) => {
            tracing::error!(error = %error, "WhatsApp Web decision candidate refresh failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "whatsapp_web_decision_refresh_error",
                "WhatsApp Web decision candidate refresh failed".to_owned(),
                false,
            )
        }
        WhatsappWebError::TaskCandidate(error) => {
            tracing::error!(error = %error, "WhatsApp Web task candidate refresh failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "whatsapp_web_task_candidate_refresh_error",
                "WhatsApp Web task candidate refresh failed".to_owned(),
                false,
            )
        }
        WhatsappWebError::Sqlx(error) => {
            tracing::error!(error = %error, "WhatsApp Web database operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "whatsapp_web_store_error",
                "WhatsApp Web store operation failed".to_owned(),
                false,
            )
        }
    }
}

pub(super) fn automation_error_parts(error: AutomationError) -> ErrorParts {
    match error {
        AutomationError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_automation_request",
            message,
            false,
        ),
        AutomationError::PolicyNotFound => (
            StatusCode::NOT_FOUND,
            "automation_policy_not_found",
            "automation policy was not found".to_owned(),
            false,
        ),
        AutomationError::PolicyDisabled
        | AutomationError::ChatNotAllowed
        | AutomationError::MissingTemplateVariable(_)
        | AutomationError::UndeclaredTemplateVariable(_) => (
            StatusCode::FORBIDDEN,
            "automation_policy_denied",
            error.to_string(),
            false,
        ),
        AutomationError::EventEnvelope(error) => (
            StatusCode::BAD_REQUEST,
            "invalid_automation_event",
            error.to_string(),
            false,
        ),
        AutomationError::EventStore(error) => {
            tracing::error!(error = %error, "automation event store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "automation_store_error",
                "automation operation failed".to_owned(),
                false,
            )
        }
        AutomationError::Sqlx(error) => {
            tracing::error!(error = %error, "automation database operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "automation_store_error",
                "automation operation failed".to_owned(),
                false,
            )
        }
    }
}

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
