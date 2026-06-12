use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::events::{CalendarAccountStore, CalendarError};
use crate::domains::mail::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind,
    NewProviderAccount, NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use crate::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceError, SecretReferenceStore, SecretStoreKind,
};
use crate::vault::{
    HostVault, HostVaultError, HostVaultManifestEntry, SecretEntryContext, VaultMode,
};

use super::AppState;

pub(crate) fn spawn_host_vault_manifest_reconciliation(state: &AppState) {
    if state.config.database_url().is_none() {
        return;
    }
    let Ok(status) = state.vault.status() else {
        tracing::warn!("host vault reconciliation skipped: vault status unavailable");
        return;
    };
    if status.state != VaultMode::Unlocked {
        return;
    }
    let Some(pool) = state.database.pool().cloned() else {
        return;
    };
    let vault = state.vault.clone();
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        tracing::warn!("host vault reconciliation skipped: no Tokio runtime");
        return;
    };

    handle.spawn(async move {
        match reconcile_host_vault_manifest(pool, vault).await {
            Ok(summary)
                if summary.restored_accounts > 0 || summary.restored_calendar_accounts > 0 =>
            {
                tracing::info!(
                    restored_accounts = summary.restored_accounts,
                    restored_calendar_accounts = summary.restored_calendar_accounts,
                    "host vault manifest reconciliation completed"
                );
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(error = %error, "host vault manifest reconciliation failed");
            }
        }
    });
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct HostVaultReconciliationSummary {
    restored_accounts: usize,
    restored_calendar_accounts: usize,
}

async fn reconcile_host_vault_manifest(
    pool: PgPool,
    vault: HostVault,
) -> Result<HostVaultReconciliationSummary, HostVaultReconciliationError> {
    let manifest = vault.account_secret_manifest()?;
    let secret_store = SecretReferenceStore::new(pool.clone());
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let calendar_store = CalendarAccountStore::new(pool.clone());
    let mut summary = HostVaultReconciliationSummary::default();

    for mut entry in manifest {
        enrich_manifest_entry_from_postgres(&pool, &vault, &mut entry).await?;
        let Some(recoverable) = RecoverableProviderSecret::from_manifest(entry) else {
            continue;
        };
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &recoverable.secret_ref,
                    recoverable.secret_kind,
                    recoverable.store_kind,
                    &recoverable.label,
                )
                .metadata(recoverable.secret_metadata.clone()),
            )
            .await?;

        if communication_store
            .provider_account(&recoverable.account_id)
            .await?
            .is_none()
        {
            communication_store
                .upsert_provider_account(
                    &NewProviderAccount::new(
                        &recoverable.account_id,
                        recoverable.provider_kind,
                        &recoverable.display_name,
                        &recoverable.external_account_id,
                    )
                    .config(recoverable.provider_account_config.clone()),
                )
                .await?;
            summary.restored_accounts += 1;
        }

        communication_store
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                &recoverable.account_id,
                recoverable.secret_purpose,
                &recoverable.secret_ref,
            ))
            .await?;

        if restore_linked_calendar_account(&calendar_store, &recoverable).await? {
            summary.restored_calendar_accounts += 1;
        }
    }

    Ok(summary)
}

async fn enrich_manifest_entry_from_postgres(
    pool: &PgPool,
    vault: &HostVault,
    entry: &mut HostVaultManifestEntry,
) -> Result<(), HostVaultReconciliationError> {
    if entry.entry_kind != "provider_credential" {
        return Ok(());
    }

    let Some(row) = sqlx::query(
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
    .await?
    else {
        return Ok(());
    };

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

async fn restore_linked_calendar_account(
    calendar_store: &CalendarAccountStore,
    secret: &RecoverableProviderSecret,
) -> Result<bool, HostVaultReconciliationError> {
    match secret.provider_kind {
        EmailProviderKind::Gmail => {
            let calendar_account_id = format!("google-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .upsert_google_workspace_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        EmailProviderKind::Icloud => {
            let calendar_account_id = format!("icloud-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .upsert_apple_icloud_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

struct RecoverableProviderSecret {
    account_id: String,
    provider_kind: EmailProviderKind,
    display_name: String,
    external_account_id: String,
    secret_ref: String,
    secret_kind: SecretKind,
    store_kind: SecretStoreKind,
    secret_purpose: ProviderAccountSecretPurpose,
    label: String,
    secret_metadata: Value,
    provider_account_config: Value,
}

impl RecoverableProviderSecret {
    fn from_manifest(entry: HostVaultManifestEntry) -> Option<Self> {
        if entry.entry_kind != "provider_credential" {
            return None;
        }
        let provider = metadata_string(&entry.metadata, "provider")?;
        let provider_kind = EmailProviderKind::try_from(provider.as_str()).ok()?;
        if !matches!(
            provider_kind,
            EmailProviderKind::Gmail | EmailProviderKind::Icloud | EmailProviderKind::Imap
        ) {
            return None;
        }
        let secret_kind = SecretKind::try_from(entry.secret_kind.as_str()).ok()?;
        let store_kind = SecretStoreKind::try_from(entry.store_kind.as_str()).ok()?;
        let secret_purpose = ProviderAccountSecretPurpose::try_from(entry.purpose.as_str()).ok()?;
        if !secret_purpose.accepts_secret_kind(secret_kind) {
            return None;
        }

        let account_id =
            non_empty(metadata_string(&entry.metadata, "account_id")).unwrap_or(entry.account_id);
        let display_name = non_empty(metadata_string(&entry.metadata, "display_name"))
            .unwrap_or_else(|| fallback_display_name(provider_kind, &entry.label, &account_id));
        let external_account_id =
            non_empty(metadata_string(&entry.metadata, "external_account_id"))
                .unwrap_or_else(|| account_id.clone());
        let provider_account_config = entry
            .metadata
            .get("provider_account_config")
            .filter(|value| value.is_object())
            .cloned()
            .unwrap_or_else(|| {
                fallback_provider_account_config(
                    provider_kind,
                    &entry.metadata,
                    &external_account_id,
                )
            });

        Some(Self {
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            secret_ref: entry.secret_ref,
            secret_kind,
            store_kind,
            secret_purpose,
            label: entry.label,
            secret_metadata: entry.metadata,
            provider_account_config,
        })
    }
}

fn fallback_provider_account_config(
    provider_kind: EmailProviderKind,
    metadata: &Value,
    external_account_id: &str,
) -> Value {
    let connected_services = metadata
        .get("connected_services")
        .cloned()
        .unwrap_or_else(|| json!(["mail"]));
    match provider_kind {
        EmailProviderKind::Gmail => json!({
            "auth": "oauth",
            "api": "gmail",
            "connected_services": connected_services,
            "history_stream_id": "gmail:history"
        }),
        EmailProviderKind::Icloud => json!({
            "host": "imap.mail.me.com",
            "port": 993,
            "tls": true,
            "mailbox": "INBOX",
            "username": external_account_id,
            "connected_services": connected_services
        }),
        EmailProviderKind::Imap => json!({
            "username": external_account_id,
            "connected_services": connected_services
        }),
        _ => json!({}),
    }
}

fn fallback_display_name(
    provider_kind: EmailProviderKind,
    label: &str,
    account_id: &str,
) -> String {
    let trimmed = label.trim();
    if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("IMAP password") {
        return trimmed.to_owned();
    }
    match provider_kind {
        EmailProviderKind::Gmail => "Google Workspace".to_owned(),
        EmailProviderKind::Icloud => "iCloud".to_owned(),
        EmailProviderKind::Imap => account_id.to_owned(),
        _ => account_id.to_owned(),
    }
}

fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

#[derive(Debug, Error)]
enum HostVaultReconciliationError {
    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
