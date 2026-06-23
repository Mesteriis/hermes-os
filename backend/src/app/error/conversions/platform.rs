use super::super::types::ApiError;
use crate::domains::signal_hub::SignalHubError;
use crate::platform::audit::ApiAuditError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::settings::SettingsError;
use crate::vault::HostVaultError;

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
