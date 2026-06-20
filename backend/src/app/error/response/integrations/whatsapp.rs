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
        WhatsappWebError::ProviderAccountStore(error) => internal(
            error,
            "WhatsApp Web provider account store operation failed",
            "whatsapp_web_provider_account_store_error",
            "WhatsApp Web provider account store operation failed",
        ),
        WhatsappWebError::CommunicationProjection(error) => internal(
            error,
            "WhatsApp Web communication projection failed",
            "whatsapp_web_projection_error",
            "WhatsApp Web communication projection failed",
        ),
        WhatsappWebError::ReviewInboxWorkflow(error) => internal(
            error,
            "WhatsApp Web review inbox mirroring failed",
            "whatsapp_web_review_inbox_error",
            "WhatsApp Web review inbox mirroring failed",
        ),
        WhatsappWebError::ObservationStore(error) => internal(
            error,
            "WhatsApp Web observation store operation failed",
            "whatsapp_web_observation_error",
            "WhatsApp Web store operation failed",
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
