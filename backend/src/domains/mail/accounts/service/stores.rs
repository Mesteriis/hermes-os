use crate::domains::mail::core::CommunicationIngestionStore;
use crate::platform::secrets::SecretReferenceStore;

use super::super::errors::EmailAccountSetupError;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub(in crate::domains::mail::accounts::service) fn communication_store(
        &self,
    ) -> Result<&CommunicationIngestionStore, EmailAccountSetupError> {
        self.communication_store
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    pub(in crate::domains::mail::accounts::service) fn secret_store(
        &self,
    ) -> Result<&SecretReferenceStore, EmailAccountSetupError> {
        self.secret_store
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }
}
