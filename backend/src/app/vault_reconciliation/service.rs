use sqlx::postgres::PgPool;

use crate::ai::control_center::AiControlCenterStore;
use crate::domains::calendar::events::CalendarAccountStore;
use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore, NewProviderAccount,
    NewProviderAccountSecretBinding,
};
use crate::platform::secrets::{NewSecretReference, SecretReferenceStore};
use crate::vault::HostVault;

use super::ai_provider_recovery::RecoverableAiProviderSecret;
use super::calendar_restore::restore_linked_calendar_account;
use super::errors::HostVaultReconciliationError;
use super::manifest_enrichment::enrich_manifest_entry_from_postgres;
use super::provider_recovery::RecoverableProviderSecret;
use super::summary::HostVaultReconciliationSummary;

pub(super) async fn reconcile_host_vault_manifest(
    pool: PgPool,
    vault: HostVault,
) -> Result<HostVaultReconciliationSummary, HostVaultReconciliationError> {
    let manifest = vault.account_secret_manifest()?;
    let secret_store = SecretReferenceStore::new(pool.clone());
    let provider_account_store = CommunicationProviderAccountStore::new(pool.clone());
    let provider_secret_binding_store = CommunicationProviderSecretBindingStore::new(pool.clone());
    let calendar_store = CalendarAccountStore::new(pool.clone());
    let ai_store = AiControlCenterStore::new(pool.clone());
    let mut summary = HostVaultReconciliationSummary::default();

    for mut entry in manifest {
        enrich_manifest_entry_from_postgres(&pool, &vault, &mut entry).await?;
        if let Some(recoverable) = RecoverableAiProviderSecret::from_manifest(entry.clone()) {
            restore_ai_secret_reference(&secret_store, &recoverable).await?;
            let restore_missing = ai_store
                .provider(&recoverable.restore.provider_id)
                .await?
                .is_none()
                || ai_store
                    .api_key_secret_ref(&recoverable.restore.provider_id)
                    .await?
                    .is_none();
            if restore_missing {
                ai_store
                    .restore_provider_from_vault(&recoverable.restore)
                    .await?;
                summary.restored_ai_providers += 1;
            }
            continue;
        }
        let Some(recoverable) = RecoverableProviderSecret::from_manifest(entry) else {
            continue;
        };
        restore_secret_reference(&secret_store, &recoverable).await?;
        restore_provider_account(&provider_account_store, &recoverable, &mut summary).await?;
        restore_provider_account_secret_binding(&provider_secret_binding_store, &recoverable)
            .await?;

        if restore_linked_calendar_account(&calendar_store, &recoverable).await? {
            summary.restored_calendar_accounts += 1;
        }
    }

    Ok(summary)
}

async fn restore_ai_secret_reference(
    store: &SecretReferenceStore,
    secret: &RecoverableAiProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret.restore.secret_ref,
                secret.secret_kind,
                secret.store_kind,
                &secret.restore.secret_label,
            )
            .metadata(secret.restore.secret_metadata.clone()),
        )
        .await?;
    Ok(())
}

async fn restore_secret_reference(
    store: &SecretReferenceStore,
    secret: &RecoverableProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret.secret_ref,
                secret.secret_kind,
                secret.store_kind,
                &secret.label,
            )
            .metadata(secret.secret_metadata.clone()),
        )
        .await?;
    Ok(())
}

async fn restore_provider_account(
    store: &CommunicationProviderAccountStore,
    secret: &RecoverableProviderSecret,
    summary: &mut HostVaultReconciliationSummary,
) -> Result<(), HostVaultReconciliationError> {
    if store.get(&secret.account_id).await?.is_some() {
        return Ok(());
    }

    store
        .restore(
            &NewProviderAccount::new(
                &secret.account_id,
                secret.provider_kind,
                &secret.display_name,
                &secret.external_account_id,
            )
            .config(secret.provider_account_config.clone()),
        )
        .await?;
    summary.restored_accounts += 1;
    Ok(())
}

async fn restore_provider_account_secret_binding(
    store: &CommunicationProviderSecretBindingStore,
    secret: &RecoverableProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .restore(&NewProviderAccountSecretBinding::new(
            &secret.account_id,
            secret.secret_purpose,
            &secret.secret_ref,
        ))
        .await?;
    Ok(())
}
