use serde_json::json;
use sqlx::postgres::PgPool;

use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use crate::vault::{HostVault, SecretEntryContext};

use super::errors::AiControlCenterError;
use super::store::AiControlCenterStore;
use super::validation::validate_non_empty;

const SECRET_PURPOSE_API_KEY: &str = "api_key";

pub async fn store_api_key_in_host_vault(
    pool: &PgPool,
    vault: &HostVault,
    provider_id: &str,
    api_key: &str,
) -> Result<String, AiControlCenterError> {
    validate_non_empty("provider_id", provider_id)?;
    validate_non_empty("api_key", api_key)?;
    let secret_ref = format!("secret:ai-provider:{provider_id}:{SECRET_PURPOSE_API_KEY}");
    let metadata = json!({
        "provider_id": provider_id,
        "secret_purpose": SECRET_PURPOSE_API_KEY
    });
    let reference = NewSecretReference::new(
        &secret_ref,
        SecretKind::ApiToken,
        SecretStoreKind::HostVault,
        "AI provider API key",
    )
    .metadata(metadata.clone());

    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&reference)
        .await?;
    vault.store_secret(
        &secret_ref,
        api_key.trim(),
        SecretEntryContext {
            entry_kind: "ai_provider",
            account_id: provider_id,
            purpose: SECRET_PURPOSE_API_KEY,
            secret_kind: SecretKind::ApiToken.as_str(),
            label: "AI provider API key",
            metadata: &metadata,
        },
    )?;
    AiControlCenterStore::new(pool.clone())
        .bind_api_key_secret(provider_id, &secret_ref)
        .await?;

    Ok(secret_ref)
}
