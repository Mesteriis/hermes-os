use thiserror::Error;

use crate::platform::communications::ProviderCredential;
use crate::platform::secrets::{
    SecretKind, SecretReferenceError, SecretReferenceStore, SecretResolutionError, SecretResolver,
};
use hermes_communications_api::accounts::ProviderAccountSecretPurpose;
use hermes_communications_postgres::errors::CommunicationIngestionError;
use hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore;

#[derive(Debug, Error)]
pub enum ProviderCredentialError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

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
        if account_id.trim().is_empty() {
            return Err(CommunicationIngestionError::EmptyField("account_id").into());
        }

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
