use axum::http::StatusCode;

use crate::integrations::mail::accounts::EmailAccountSetupError;

use super::ErrorParts;

pub(super) fn account_setup_error_parts(error: EmailAccountSetupError) -> ErrorParts {
    let status = if matches!(
        error,
        EmailAccountSetupError::InvalidRequest { .. }
            | EmailAccountSetupError::MissingProviderField { .. }
    ) {
        StatusCode::BAD_REQUEST
    } else {
        tracing::error!(error = %error, "account setup failed");
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (
        status,
        "account_setup_error",
        if status == StatusCode::BAD_REQUEST {
            error.to_string()
        } else {
            "account setup failed".to_owned()
        },
        false,
    )
}
