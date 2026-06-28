use axum::http::StatusCode;

use crate::application::provider_runtime_contracts::YandexTelemostError;

use super::super::ErrorParts;

pub(super) fn yandex_telemost_error_parts(error: YandexTelemostError) -> ErrorParts {
    match error {
        YandexTelemostError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_yandex_telemost_request",
            message,
            false,
        ),
        YandexTelemostError::ProviderAccountStore(error) => internal(
            error,
            "Yandex Telemost provider account store operation failed",
            "yandex_telemost_provider_account_store_error",
            "Yandex Telemost provider account store operation failed",
        ),
        YandexTelemostError::ProviderSecretBindingStore(error) => internal(
            error,
            "Yandex Telemost provider secret binding operation failed",
            "yandex_telemost_provider_secret_binding_error",
            "Yandex Telemost credential metadata operation failed",
        ),
        YandexTelemostError::EventStore(error) => internal(
            error,
            "Yandex Telemost event store operation failed",
            "yandex_telemost_event_store_error",
            "Yandex Telemost event store operation failed",
        ),
        YandexTelemostError::EventEnvelope(error) => internal(
            error,
            "Yandex Telemost event envelope operation failed",
            "yandex_telemost_event_envelope_error",
            "Yandex Telemost event envelope operation failed",
        ),
        YandexTelemostError::SecretReference(error) => internal(
            error,
            "Yandex Telemost secret reference operation failed",
            "yandex_telemost_secret_reference_error",
            "Yandex Telemost credential metadata operation failed",
        ),
        YandexTelemostError::SecretResolution(error) => internal(
            error,
            "Yandex Telemost secret resolution failed",
            "yandex_telemost_secret_resolution_error",
            "Yandex Telemost credential resolution failed",
        ),
        YandexTelemostError::HostVault(error) => internal(
            error,
            "Yandex Telemost host vault operation failed",
            "yandex_telemost_host_vault_error",
            "Yandex Telemost credential storage operation failed",
        ),
        YandexTelemostError::Http(error) => internal(
            error,
            "Yandex Telemost provider HTTP operation failed",
            "yandex_telemost_provider_http_error",
            "Yandex Telemost provider request failed",
        ),
        YandexTelemostError::Serialization(error) => internal(
            error,
            "Yandex Telemost serialization failed",
            "yandex_telemost_serialization_error",
            "Yandex Telemost serialization failed",
        ),
        YandexTelemostError::Io(error) => internal(
            error,
            "Yandex Telemost local recording bundle I/O failed",
            "yandex_telemost_local_bundle_io_error",
            "Yandex Telemost local recording bundle I/O failed",
        ),
        YandexTelemostError::ObservationStore(error) => internal(
            error,
            "Yandex Telemost observation capture failed",
            "yandex_telemost_observation_store_error",
            "Yandex Telemost observation capture failed",
        ),
        YandexTelemostError::ReviewInbox(error) => internal(
            error,
            "Yandex Telemost review inbox mirroring failed",
            "yandex_telemost_review_inbox_error",
            "Yandex Telemost review inbox mirroring failed",
        ),
        YandexTelemostError::Settings(error) if error.is_invalid_request() => (
            StatusCode::BAD_REQUEST,
            "invalid_yandex_telemost_setting",
            error.to_string(),
            false,
        ),
        YandexTelemostError::Settings(error) => internal(
            error,
            "Yandex Telemost settings operation failed",
            "yandex_telemost_settings_error",
            "Yandex Telemost settings operation failed",
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
