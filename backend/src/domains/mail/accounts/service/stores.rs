use crate::platform::secrets::SecretReferenceStore;
use crate::vault::{CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore};
use sqlx::postgres::PgPool;

use super::super::errors::EmailAccountSetupError;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub(in crate::domains::mail::accounts::service) fn pool(
        &self,
    ) -> Result<&PgPool, EmailAccountSetupError> {
        self.pool
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

    pub(in crate::domains::mail::accounts::service) fn provider_account_store(
        &self,
    ) -> Result<CommunicationProviderAccountStore, EmailAccountSetupError> {
        Ok(CommunicationProviderAccountStore::new(self.pool()?.clone()))
    }

    pub(in crate::domains::mail::accounts::service) fn provider_secret_binding_store(
        &self,
    ) -> Result<CommunicationProviderSecretBindingStore, EmailAccountSetupError> {
        Ok(CommunicationProviderSecretBindingStore::new(
            self.pool()?.clone(),
        ))
    }
}
