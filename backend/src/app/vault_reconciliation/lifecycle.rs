use crate::app::AppState;
use crate::vault::VaultMode;

use super::service::reconcile_host_vault_manifest;

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
                if summary.restored_accounts > 0
                    || summary.restored_calendar_accounts > 0
                    || summary.restored_ai_providers > 0
                    || summary.skipped_duplicate_provider_secrets > 0
                    || summary.purged_duplicate_provider_secrets > 0 =>
            {
                tracing::info!(
                    restored_accounts = summary.restored_accounts,
                    restored_calendar_accounts = summary.restored_calendar_accounts,
                    restored_ai_providers = summary.restored_ai_providers,
                    skipped_duplicate_provider_secrets = summary.skipped_duplicate_provider_secrets,
                    purged_duplicate_provider_secrets = summary.purged_duplicate_provider_secrets,
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
