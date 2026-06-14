use crate::domains::mail::core::CommunicationIngestionStore;
use crate::platform::secrets::{DatabaseEncryptedSecretVault, SecretReferenceStore};
use crate::vault::HostVault;

use super::super::helpers::http_client;
use super::super::vault::AccountSecretVault;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub fn new(
        communication_store: CommunicationIngestionStore,
        secret_store: SecretReferenceStore,
        vault: DatabaseEncryptedSecretVault,
    ) -> Self {
        Self {
            communication_store: Some(communication_store),
            secret_store: Some(secret_store),
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_for_vault_only(vault: DatabaseEncryptedSecretVault) -> Self {
        Self {
            communication_store: None,
            secret_store: None,
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_with_host_vault(
        communication_store: CommunicationIngestionStore,
        secret_store: SecretReferenceStore,
        vault: HostVault,
    ) -> Self {
        Self {
            communication_store: Some(communication_store),
            secret_store: Some(secret_store),
            vault: AccountSecretVault::Host(vault),
            http: http_client(),
        }
    }
}
