use crate::platform::secrets::{DatabaseEncryptedSecretVault, SecretReferenceStore};
use crate::vault::HostVault;
use sqlx::postgres::PgPool;

use super::super::helpers::http_client;
use super::super::vault::AccountSecretVault;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub fn new(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: DatabaseEncryptedSecretVault,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_for_vault_only(vault: DatabaseEncryptedSecretVault) -> Self {
        Self {
            pool: None,
            secret_store: None,
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_with_host_vault(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: HostVault,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            vault: AccountSecretVault::Host(vault),
            http: http_client(),
        }
    }
}
