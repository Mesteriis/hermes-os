use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::vault::{HostVault, HostVaultManifestEntry, SecretEntryContext};

use super::errors::HostVaultReconciliationError;
use super::provider_recovery::is_recoverable_provider_entry_kind;

pub(super) async fn enrich_manifest_entry_from_postgres(
    pool: &PgPool,
    vault: &HostVault,
    entry: &mut HostVaultManifestEntry,
) -> Result<(), HostVaultReconciliationError> {
    if entry.entry_kind == "ai_provider" {
        enrich_ai_provider_entry_from_postgres(pool, vault, entry).await?;
        return Ok(());
    }
    if !is_recoverable_provider_entry_kind(&entry.entry_kind) {
        return Ok(());
    }

    let Some(row) = provider_account_metadata_row(pool, entry).await? else {
        return Ok(());
    };

    let metadata = manifest_metadata_from_row(&row, entry)?;
    vault.upsert_account_secret_manifest_entry(
        &entry.secret_ref,
        SecretEntryContext {
            entry_kind: &entry.entry_kind,
            account_id: &entry.account_id,
            purpose: &entry.purpose,
            secret_kind: &entry.secret_kind,
            label: &entry.label,
            metadata: &metadata,
        },
    )?;
    entry.metadata = metadata;
    Ok(())
}

async fn enrich_ai_provider_entry_from_postgres(
    pool: &PgPool,
    vault: &HostVault,
    entry: &mut HostVaultManifestEntry,
) -> Result<(), HostVaultReconciliationError> {
    let Some(row) = ai_provider_metadata_row(pool, entry).await? else {
        return Ok(());
    };

    let metadata = ai_manifest_metadata_from_row(&row, entry)?;
    vault.upsert_account_secret_manifest_entry(
        &entry.secret_ref,
        SecretEntryContext {
            entry_kind: &entry.entry_kind,
            account_id: &entry.account_id,
            purpose: &entry.purpose,
            secret_kind: &entry.secret_kind,
            label: &entry.label,
            metadata: &metadata,
        },
    )?;
    entry.metadata = metadata;
    Ok(())
}

async fn provider_account_metadata_row(
    pool: &PgPool,
    entry: &HostVaultManifestEntry,
) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT cpa.provider_kind, cpa.display_name, cpa.external_account_id, cpa.config
        FROM communication_provider_accounts cpa
        JOIN communication_provider_account_secret_refs refs
          ON refs.account_id = cpa.account_id
         AND refs.secret_purpose = $2
         AND refs.secret_ref = $3
        WHERE cpa.account_id = $1
        "#,
    )
    .bind(&entry.account_id)
    .bind(&entry.purpose)
    .bind(&entry.secret_ref)
    .fetch_optional(pool)
    .await
}

fn manifest_metadata_from_row(
    row: &sqlx::postgres::PgRow,
    entry: &HostVaultManifestEntry,
) -> Result<Value, sqlx::Error> {
    let provider_kind: String = row.try_get("provider_kind")?;
    let display_name: String = row.try_get("display_name")?;
    let external_account_id: String = row.try_get("external_account_id")?;
    let config: Value = row.try_get("config")?;
    let mut metadata = json!({
        "provider": provider_kind,
        "account_id": entry.account_id.clone(),
        "display_name": display_name,
        "external_account_id": external_account_id,
        "provider_account_config": config
    });
    if let Some(connected_services) = metadata
        .get("provider_account_config")
        .and_then(|config| config.get("connected_services"))
        .cloned()
    {
        metadata["connected_services"] = connected_services;
    }
    Ok(metadata)
}

async fn ai_provider_metadata_row(
    pool: &PgPool,
    entry: &HostVaultManifestEntry,
) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT
            provider.provider_id,
            provider.provider_kind,
            provider.provider_key,
            provider.display_name,
            provider.status,
            provider.consent_state,
            provider.config,
            provider.capabilities
        FROM ai_provider_accounts provider
        JOIN ai_provider_secret_refs refs
          ON refs.provider_id = provider.provider_id
         AND refs.secret_purpose = $2
         AND refs.secret_ref = $3
        WHERE provider.provider_id = $1
        "#,
    )
    .bind(&entry.account_id)
    .bind(&entry.purpose)
    .bind(&entry.secret_ref)
    .fetch_optional(pool)
    .await
}

fn ai_manifest_metadata_from_row(
    row: &sqlx::postgres::PgRow,
    entry: &HostVaultManifestEntry,
) -> Result<Value, sqlx::Error> {
    let provider_id: String = row.try_get("provider_id")?;
    let provider_kind: String = row.try_get("provider_kind")?;
    let provider_key: String = row.try_get("provider_key")?;
    let display_name: String = row.try_get("display_name")?;
    let status: String = row.try_get("status")?;
    let consent_state: String = row.try_get("consent_state")?;
    let config: Value = row.try_get("config")?;
    let capabilities: Value = row.try_get("capabilities")?;
    let mut metadata = json!({
        "provider_id": provider_id,
        "provider_kind": provider_kind,
        "provider_key": provider_key,
        "display_name": display_name,
        "status": status,
        "consent_state": consent_state,
        "config": config,
        "capabilities": capabilities,
        "secret_purpose": entry.purpose.clone()
    });
    if let Some(base_url) = metadata
        .get("config")
        .and_then(|config| config.get("base_url"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        metadata["base_url"] = json!(base_url);
    }
    Ok(metadata)
}
