use super::super::types::ApiError;
use crate::domains::signal_hub::store::SignalHubError;
use crate::platform::audit::ApiAuditError;
use crate::platform::settings::SettingsError;
use crate::vault::HostVaultError;
use hermes_events_api::EventEnvelopeError;
use hermes_events_postgres::errors::EventStoreError;

impl From<EventEnvelopeError> for ApiError {
    fn from(error: EventEnvelopeError) -> Self {
        Self::InvalidEnvelope(error)
    }
}

impl From<EventStoreError> for ApiError {
    fn from(error: EventStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<SettingsError> for ApiError {
    fn from(error: SettingsError) -> Self {
        match error {
            SettingsError::SettingNotFound { .. } => Self::SettingNotFound,
            _ => Self::Settings(error),
        }
    }
}

impl From<SignalHubError> for ApiError {
    fn from(error: SignalHubError) -> Self {
        Self::SignalHub(error)
    }
}

impl From<ApiAuditError> for ApiError {
    fn from(error: ApiAuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<HostVaultError> for ApiError {
    fn from(error: HostVaultError) -> Self {
        Self::HostVault(error)
    }
}
