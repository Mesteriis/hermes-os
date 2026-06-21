use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::errors::{CommunicationIngestionError, ProviderCredentialError};
use super::models::{
    NewProviderAccountSecretBinding, ProviderAccountSecretBinding, ProviderAccountSecretPurpose,
    ProviderCredential,
};
use super::provider_store::CommunicationProviderSecretBindingStore;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;

impl CommunicationIngestionStore {
    pub async fn bind_provider_account_secret(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .bind(binding)
            .await
    }

    pub async fn provider_account_secret_bindings(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .list_for_account(account_id)
            .await
    }

    pub async fn provider_account_secret_binding(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .get_for_account(account_id, secret_purpose)
            .await
    }
}

pub struct ProviderCredentialReader<'a, R: SecretResolver + ?Sized> {
    secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    resolver: &'a R,
}

impl<'a, R: SecretResolver + ?Sized> ProviderCredentialReader<'a, R> {
    pub fn new(
        secret_binding_store: CommunicationProviderSecretBindingStore,
        secret_store: SecretReferenceStore,
        resolver: &'a R,
    ) -> Self {
        Self {
            secret_binding_store,
            secret_store,
            resolver,
        }
    }

    pub async fn read(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<ProviderCredential, ProviderCredentialError> {
        validate_non_empty("account_id", account_id)?;

        let binding = self
            .secret_binding_store
            .get_for_account(account_id, secret_purpose)
            .await?
            .ok_or_else(|| ProviderCredentialError::MissingBinding {
                account_id: account_id.trim().to_owned(),
                secret_purpose,
            })?;
        let reference = self
            .secret_store
            .secret_reference(&binding.secret_ref)
            .await?
            .ok_or_else(|| ProviderCredentialError::MissingSecretReference {
                secret_ref: binding.secret_ref.clone(),
            })?;
        if !binding
            .secret_purpose
            .accepts_secret_kind(reference.secret_kind)
        {
            return Err(ProviderCredentialError::IncompatibleSecretKind {
                secret_ref: reference.secret_ref.clone(),
                secret_purpose: binding.secret_purpose,
                secret_kind: reference.secret_kind,
            });
        }

        let secret = self.resolver.resolve(&reference).await?;

        Ok(ProviderCredential {
            binding,
            reference,
            secret,
        })
    }
}
