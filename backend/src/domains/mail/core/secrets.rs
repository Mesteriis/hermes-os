use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::errors::{CommunicationIngestionError, ProviderCredentialError};
use super::models::{
    NewProviderAccountSecretBinding, ProviderAccountSecretBinding, ProviderAccountSecretPurpose,
    ProviderCredential,
};
use super::rows::row_to_secret_binding;
use super::store::CommunicationIngestionStore;
use super::validation::validate_non_empty;

impl CommunicationIngestionStore {
    pub async fn bind_provider_account_secret(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        binding.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_account_secret_refs (
                account_id,
                secret_purpose,
                secret_ref,
                updated_at
            )
            VALUES ($1, $2, $3, now())
            ON CONFLICT (account_id, secret_purpose)
            DO UPDATE SET
                secret_ref = EXCLUDED.secret_ref,
                updated_at = now()
            RETURNING
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            "#,
        )
        .bind(binding.account_id.trim())
        .bind(binding.secret_purpose.as_str())
        .bind(binding.secret_ref.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_secret_binding(row)
    }

    pub async fn provider_account_secret_bindings(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            ORDER BY secret_purpose ASC
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_secret_binding).collect()
    }

    pub async fn provider_account_secret_binding(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
              AND secret_purpose = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(secret_purpose.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_secret_binding).transpose()
    }
}

pub struct ProviderCredentialReader<'a, R: SecretResolver + ?Sized> {
    communication_store: CommunicationIngestionStore,
    secret_store: SecretReferenceStore,
    resolver: &'a R,
}

impl<'a, R: SecretResolver + ?Sized> ProviderCredentialReader<'a, R> {
    pub fn new(
        communication_store: CommunicationIngestionStore,
        secret_store: SecretReferenceStore,
        resolver: &'a R,
    ) -> Self {
        Self {
            communication_store,
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
            .communication_store
            .provider_account_secret_binding(account_id, secret_purpose)
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
