use axum::http::StatusCode;

use super::super::types::ApiError;
use super::{ErrorParts, mail};

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::Messages(error) => internal_store(
            error,
            "communication message API store operation failed",
            "communication_message_store_error",
            "communication message store operation failed",
        ),
        ApiError::CommunicationIngestion(error) => internal_store(
            error,
            "communication account API store operation failed",
            "communication_account_store_error",
            "communication account store operation failed",
        ),
        ApiError::MailStorage(error) => internal_store(
            error,
            "communication attachment API store operation failed",
            "communication_attachment_store_error",
            "communication attachment store operation failed",
        ),
        ApiError::InvalidCommunicationQuery(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_communication_query",
            message.to_owned(),
            false,
        ),
        ApiError::EmailAccountDeleteConflict => (
            StatusCode::CONFLICT,
            "email_account_delete_conflict",
            "email account has retained communication evidence and cannot be deleted".to_owned(),
            false,
        ),
        ApiError::ProviderWriteConfirmationRequired => (
            StatusCode::BAD_REQUEST,
            "provider_write_confirmation_required",
            "explicit provider write confirmation is required".to_owned(),
            false,
        ),
        ApiError::CommunicationMessageNotFound => (
            StatusCode::NOT_FOUND,
            "communication_message_not_found",
            "communication message was not found".to_owned(),
            false,
        ),
        ApiError::AccountSetup(error) => mail::account_setup_error_parts(error),
        ApiError::AccountSetupState => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "account_setup_state_error",
            "account setup state is unavailable".to_owned(),
            false,
        ),
        ApiError::AccountSetupPendingGrantNotFound => (
            StatusCode::NOT_FOUND,
            "account_setup_pending_grant_not_found",
            "pending Gmail OAuth setup was not found".to_owned(),
            false,
        ),
        ApiError::AccountSetupStateMismatch => (
            StatusCode::BAD_REQUEST,
            "account_setup_state_mismatch",
            "Gmail OAuth state does not match the pending setup".to_owned(),
            false,
        ),
        _ => unreachable!("communication response mapper received non-communication ApiError"),
    }
}

fn internal_store(
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
