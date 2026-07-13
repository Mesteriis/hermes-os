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
        WhatsappWebError::CommunicationMessagePort(error) => internal(
            error,
            "WhatsApp Web communication message port operation failed",
            "whatsapp_web_message_port_error",
            "WhatsApp Web message read model operation failed",
        ),
        WhatsappWebError::ObservationStore(error) => internal(
            error,
            "WhatsApp Web observation store operation failed",
            "whatsapp_web_observation_error",
            "WhatsApp Web store operation failed",
        ),
        WhatsappWebError::SecretReference(error) => internal(
            error,
            "WhatsApp Web secret reference operation failed",
            "whatsapp_web_secret_reference_error",
            "WhatsApp Web credential metadata operation failed",
        ),
        WhatsappWebError::SecretResolution(error) => internal(
            error,
            "WhatsApp Web secret resolution operation failed",
            "whatsapp_web_secret_resolution_error",
            "WhatsApp Web credential resolution failed",
        ),
        WhatsappWebError::HostVault(error) => internal(
            error,
            "WhatsApp Web host vault operation failed",
            "whatsapp_web_host_vault_error",
            "WhatsApp Web credential vault operation failed",
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
