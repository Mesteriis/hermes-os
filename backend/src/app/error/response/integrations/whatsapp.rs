use axum::http::StatusCode;

use crate::integrations::whatsapp::client::WhatsappWebError;

use super::super::ErrorParts;

pub(super) fn whatsapp_web_error_parts(error: WhatsappWebError) -> ErrorParts {
    match error {
        WhatsappWebError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_whatsapp_web_request",
            message,
            false,
        ),
        WhatsappWebError::Communication(error) => internal(
            error,
            "WhatsApp Web communication store operation failed",
            "whatsapp_web_store_error",
            "WhatsApp Web store operation failed",
        ),
        WhatsappWebError::MessageProjection(error) => internal(
            error,
            "WhatsApp Web message projection failed",
            "whatsapp_web_projection_error",
            "WhatsApp Web message projection failed",
        ),
        WhatsappWebError::Decision(error) => internal(
            error,
            "WhatsApp Web decision candidate refresh failed",
            "whatsapp_web_decision_refresh_error",
            "WhatsApp Web decision candidate refresh failed",
        ),
        WhatsappWebError::TaskCandidate(error) => internal(
            error,
            "WhatsApp Web task candidate refresh failed",
            "whatsapp_web_task_candidate_refresh_error",
            "WhatsApp Web task candidate refresh failed",
        ),
        WhatsappWebError::Sqlx(error) => internal(
            error,
            "WhatsApp Web database operation failed",
            "whatsapp_web_store_error",
            "WhatsApp Web store operation failed",
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
