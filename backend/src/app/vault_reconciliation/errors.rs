use thiserror::Error;

use crate::ai::control_center::AiControlCenterError;
use crate::domains::calendar::events::CalendarError;
use crate::platform::secrets::SecretReferenceError;
use crate::vault::HostVaultError;
use hermes_communications_postgres::errors::CommunicationIngestionError;

#[derive(Debug, Error)]
pub(super) enum HostVaultReconciliationError {
    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    AiControlCenter(#[from] AiControlCenterError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
