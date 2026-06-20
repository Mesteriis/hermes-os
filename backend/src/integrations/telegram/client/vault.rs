use serde_json::json;

use crate::platform::communications::{CommunicationProviderKind, ProviderAccountSecretPurpose};
use crate::platform::secrets::{DatabaseEncryptedSecretVault, SecretKind, SecretStoreKind};
use crate::vault::{HostVault, SecretEntryContext};

use super::errors::TelegramError;

pub(super) struct TelegramCredentialWrite<'a> {
    pub(super) account_id: &'a str,
    pub(super) provider_kind: CommunicationProviderKind,
    pub(super) secret_purpose: ProviderAccountSecretPurpose,
    pub(super) secret_kind: SecretKind,
    pub(super) label: &'a str,
    pub(super) value: String,
}

pub enum TelegramSecretVault {
    Database(DatabaseEncryptedSecretVault),
    Host(HostVault),
}

impl TelegramSecretVault {
    pub fn database(vault: DatabaseEncryptedSecretVault) -> Self {
        Self::Database(vault)
    }

    pub fn host(vault: HostVault) -> Self {
        Self::Host(vault)
    }

    pub(super) fn store_kind(&self) -> SecretStoreKind {
        match self {
            Self::Database(_) => SecretStoreKind::DatabaseEncryptedVault,
            Self::Host(_) => SecretStoreKind::HostVault,
        }
    }

    pub(super) async fn store_secret(
        &self,
        secret_ref: &str,
        credential: &TelegramCredentialWrite<'_>,
    ) -> Result<(), TelegramError> {
        match self {
            Self::Database(vault) => vault.store_secret(secret_ref, &credential.value).await?,
            Self::Host(vault) => vault.store_secret(
                secret_ref,
                &credential.value,
                SecretEntryContext {
                    entry_kind: "provider_credential",
                    account_id: credential.account_id,
                    purpose: credential.secret_purpose.as_str(),
                    secret_kind: credential.secret_kind.as_str(),
                    label: credential.label,
                    metadata: &json!({
                        "provider": credential.provider_kind.as_str(),
                        "account_id": credential.account_id,
                        "secret_purpose": credential.secret_purpose.as_str()
                    }),
                },
            )?,
        }
        Ok(())
    }
}
