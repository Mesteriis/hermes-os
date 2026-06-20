use thiserror::Error;

use crate::platform::communications::{ProviderAccountSecretPurpose, ProviderCredential};
use crate::platform::secrets::{
    SecretKind, SecretReferenceError, SecretReferenceStore, SecretResolutionError, SecretResolver,
};

use super::CommunicationProviderSecretBindingStore;

#[derive(Debug, Error)]
pub enum ProviderCredentialError {
    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error("provider account secret binding store failed: {0}")]
    BindingStore(String),

    #[error("account_id must not be empty")]
    EmptyAccountId,

    #[error(
        "provider account secret binding not found: account_id={account_id}, secret_purpose={secret_purpose:?}"
    )]
    MissingBinding {
        account_id: String,
        secret_purpose: ProviderAccountSecretPurpose,
    },

    #[error("provider account secret reference metadata was not found: {secret_ref}")]
    MissingSecretReference { secret_ref: String },

    #[error(
        "provider account secret kind is incompatible: secret_ref={secret_ref}, secret_purpose={secret_purpose:?}, secret_kind={secret_kind:?}"
    )]
    IncompatibleSecretKind {
        secret_ref: String,
        secret_purpose: ProviderAccountSecretPurpose,
        secret_kind: SecretKind,
    },
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
        let account_id = account_id.trim();
        if account_id.is_empty() {
            return Err(ProviderCredentialError::EmptyAccountId);
        }

        let binding = self
            .secret_binding_store
            .get_for_account(account_id, secret_purpose)
            .await
            .map_err(|error| ProviderCredentialError::BindingStore(error.to_string()))?
            .ok_or_else(|| ProviderCredentialError::MissingBinding {
                account_id: account_id.to_owned(),
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
