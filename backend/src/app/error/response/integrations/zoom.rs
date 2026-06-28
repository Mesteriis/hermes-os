use axum::http::StatusCode;

use crate::application::provider_runtime_contracts::ZoomError;

use super::super::ErrorParts;

pub(super) fn zoom_error_parts(error: ZoomError) -> ErrorParts {
    match error {
        ZoomError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_zoom_request",
            message,
            false,
        ),
        ZoomError::ProviderAccountStore(error) => internal(
            error,
            "Zoom provider account store operation failed",
            "zoom_provider_account_store_error",
            "Zoom provider account store operation failed",
        ),
        ZoomError::ProviderSecretBindingStore(error) => internal(
            error,
            "Zoom provider secret binding operation failed",
            "zoom_provider_secret_binding_error",
            "Zoom provider credential metadata operation failed",
        ),
        ZoomError::Call(error) => internal(
            error,
            "Zoom call projection operation failed",
            "zoom_call_projection_error",
            "Zoom call projection operation failed",
        ),
        ZoomError::EventStore(error) => internal(
            error,
            "Zoom event store operation failed",
            "zoom_event_store_error",
            "Zoom event store operation failed",
        ),
        ZoomError::EventEnvelope(error) => internal(
            error,
            "Zoom event envelope operation failed",
            "zoom_event_envelope_error",
            "Zoom event envelope operation failed",
        ),
        ZoomError::SecretReference(error) => internal(
            error,
            "Zoom secret reference operation failed",
            "zoom_secret_reference_error",
            "Zoom credential metadata operation failed",
        ),
        ZoomError::SecretResolution(error) => internal(
            error,
            "Zoom secret resolution failed",
            "zoom_secret_resolution_error",
            "Zoom credential resolution failed",
        ),
        ZoomError::HostVault(error) => internal(
            error,
            "Zoom host vault operation failed",
            "zoom_host_vault_error",
            "Zoom credential storage operation failed",
        ),
        ZoomError::Http(error) => internal(
            error,
            "Zoom provider HTTP operation failed",
            "zoom_provider_http_error",
            "Zoom provider authorization request failed",
        ),
        ZoomError::Serialization(error) => internal(
            error,
            "Zoom credential serialization failed",
            "zoom_credential_serialization_error",
            "Zoom credential storage operation failed",
        ),
        ZoomError::Sqlx(error) => internal(
            error,
            "Zoom database operation failed",
            "zoom_store_error",
            "Zoom store operation failed",
        ),
        ZoomError::Storage(error) => internal(
            error,
            "Zoom recording storage operation failed",
            "zoom_recording_storage_error",
            "Zoom recording storage operation failed",
        ),
        ZoomError::Io(error) => internal(
            error,
            "Zoom file operation failed",
            "zoom_io_error",
            "Zoom file operation failed",
        ),
        ZoomError::Settings(error) => internal(
            error,
            "Zoom retention settings operation failed",
            "zoom_settings_error",
            "Zoom retention policy operation failed",
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
