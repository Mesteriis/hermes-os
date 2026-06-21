use serde_json::Value;

use crate::platform::secrets::{
    DatabaseEncryptedSecretVault, SecretKind, SecretReference, SecretResolutionFuture,
    SecretResolver, SecretStoreKind,
};
use crate::vault::{HostVault, SecretEntryContext};

use super::errors::EmailAccountSetupError;
use super::helpers::vault_secret_reference;

#[derive(Clone)]
pub(super) enum AccountSecretVault {
    Database(DatabaseEncryptedSecretVault),
    Host(HostVault),
}

impl AccountSecretVault {
    pub(super) fn store_kind(&self) -> SecretStoreKind {
        match self {
            Self::Database(_) => SecretStoreKind::DatabaseEncryptedVault,
            Self::Host(_) => SecretStoreKind::HostVault,
        }
    }

    pub(super) async fn store_secret(
        &self,
        secret_ref: &str,
        value: &str,
        context: SecretWriteContext<'_>,
    ) -> Result<(), EmailAccountSetupError> {
        match self {
            Self::Database(vault) => vault.store_secret(secret_ref, value).await?,
            Self::Host(vault) => vault.store_secret(
                secret_ref,
                value,
                SecretEntryContext {
                    entry_kind: context.entry_kind,
                    account_id: context.account_id,
                    purpose: context.purpose,
                    secret_kind: context.secret_kind.as_str(),
                    label: context.label,
                    metadata: context.metadata,
                },
            )?,
        }
        Ok(())
    }

    pub(super) fn secret_reference(
        &self,
        secret_ref: &str,
        secret_kind: SecretKind,
    ) -> SecretReference {
        vault_secret_reference(secret_ref, secret_kind, self.store_kind())
    }
}

pub(super) struct SecretWriteContext<'a> {
    pub(super) entry_kind: &'a str,
    pub(super) account_id: &'a str,
    pub(super) purpose: &'a str,
    pub(super) secret_kind: SecretKind,
    pub(super) label: &'a str,
    pub(super) metadata: &'a Value,
}

impl SecretResolver for AccountSecretVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        match self {
            Self::Database(vault) => vault.resolve(reference),
            Self::Host(vault) => vault.resolve(reference),
        }
    }
}
