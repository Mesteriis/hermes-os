use sqlx::postgres::PgPool;

use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::platform::config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::vault::{HostVault, VaultMode};

pub(crate) mod core;
pub(crate) mod mail;
mod mail_ai;
pub(crate) mod telegram;
pub(crate) mod telemost;
pub(crate) mod whatsapp;
pub(crate) mod zoom;
pub(crate) mod zulip;

#[derive(Clone)]
pub(crate) struct ApplicationBootstrapContext {
    pub(crate) pool: Option<PgPool>,
    pub(crate) database_url: Option<String>,
    pub(crate) nats_server_url: Option<String>,
    pub(crate) config: AppConfig,
    pub(crate) zoom_token_maintenance_scheduler_enabled: bool,
    pub(crate) zoom_recording_sync_scheduler_enabled: bool,
    pub(crate) zoom_retention_cleanup_scheduler_enabled: bool,
    pub(crate) vault: HostVault,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: InMemoryEventBus,
}

pub(super) fn host_vault_is_unlocked(vault: &HostVault) -> bool {
    match vault.status() {
        Ok(status) => status.state == VaultMode::Unlocked,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "background scheduler could not read host vault status"
            );
            false
        }
    }
}

pub(super) async fn runtime_allows_processing(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    metadata: serde_json::Value,
) -> bool {
    let store = crate::domains::signal_hub::store::SignalHubStore::new(pool.clone());
    if let Err(error) = store.restore_system_sources().await {
        tracing::warn!(
            error = %error,
            source_code,
            runtime_kind,
            "signal hub system source restore failed during runtime gate check"
        );
        return true;
    }

    match crate::platform::events::runtime::runtime_allows_processing(
        pool,
        source_code,
        runtime_kind,
        &metadata,
    )
    .await
    {
        Ok(allowed) => allowed,
        Err(error) => {
            tracing::warn!(
                error = %error,
                source_code,
                runtime_kind,
                "signal hub runtime state ensure failed"
            );
            true
        }
    }
}

#[cfg(test)]
mod tests;
