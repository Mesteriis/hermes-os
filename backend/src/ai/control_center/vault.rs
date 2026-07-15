use crate::platform::secrets::store::SecretReferenceStore;
use serde_json::json;
use sqlx::postgres::PgPool;

use crate::platform::secrets::models::{NewSecretReference, SecretKind, SecretStoreKind};
use crate::vault::HostVault;
use crate::vault::models::SecretEntryContext;

use super::errors::AiControlCenterError;
use super::models::AiProviderAccount;
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
    let provider = AiControlCenterStore::new(pool.clone())
        .provider(provider_id)
        .await?
        .ok_or(AiControlCenterError::ProviderNotFound)?;
    let secret_ref = format!("secret:ai-provider:{provider_id}:{SECRET_PURPOSE_API_KEY}");
    let metadata = api_provider_secret_metadata(&provider);
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

fn api_provider_secret_metadata(provider: &AiProviderAccount) -> serde_json::Value {
    json!({
        "provider_id": provider.provider_id,
        "provider_kind": provider.provider_kind,
        "provider_key": provider.provider_key,
        "display_name": provider.display_name,
        "status": if provider.status == "disabled" { "disabled" } else { "ready" },
        "consent_state": provider.consent_state,
        "config": provider.config,
        "capabilities": provider.capabilities,
        "secret_purpose": SECRET_PURPOSE_API_KEY,
        "secret_material": "excluded"
    })
}
