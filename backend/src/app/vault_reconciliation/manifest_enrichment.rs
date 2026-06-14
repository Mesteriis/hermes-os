use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::vault::{HostVault, HostVaultManifestEntry, SecretEntryContext};

use super::errors::HostVaultReconciliationError;

pub(super) async fn enrich_manifest_entry_from_postgres(
    pool: &PgPool,
    vault: &HostVault,
    entry: &mut HostVaultManifestEntry,
) -> Result<(), HostVaultReconciliationError> {
    if entry.entry_kind != "provider_credential" {
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
