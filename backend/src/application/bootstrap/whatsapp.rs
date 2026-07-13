use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::{
    WHATSAPP_NATIVE_MD_RUNTIME_FEATURE_DISABLED_BLOCKER,
    WHATSAPP_NATIVE_MD_STARTUP_RESTORE_ALIAS_CONFIG_KEY,
    WHATSAPP_NATIVE_MD_STARTUP_RESTORE_CONFIG_KEY, WHATSAPP_STARTUP_RESTORE_FAILED_BLOCKER,
};
use crate::platform::events::bus::InMemoryEventBus;
use crate::vault::HostVault;

pub(super) async fn reconcile_whatsapp_runtime_restore_once(
    pool: &PgPool,
    vault: &HostVault,
    event_bus: &InMemoryEventBus,
    runtime: crate::application::provider_runtime_services::WhatsAppProviderRuntimeRef,
) -> Result<(), String> {
    let account_store =
        hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            pool.clone(),
        );
    let secret_store = crate::platform::secrets::SecretReferenceStore::new(pool.clone());
    let signal_store = crate::domains::signal_hub::store::SignalHubStore::new(pool.clone());
    let event_store = hermes_events_postgres::store::EventStore::new(pool.clone());
    let fixture_ingest = crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService::new(
        pool.clone(),
        runtime.clone(),
        event_store.clone(),
        event_bus.clone(),
    );

    let accounts = account_store
        .list()
        .await
        .map_err(|error| error.to_string())?;
    for account in accounts
        .into_iter()
        .filter(|account| account.provider_kind.is_whatsapp())
    {
        let status = runtime
            .runtime_status(&secret_store, vault, &account.account_id)
            .await
            .map_err(|error| error.to_string())?;
        if !should_reconcile_whatsapp_runtime_restore(&status) {
            continue;
        }
        let (status, event_source) = restore_whatsapp_runtime_from_vault_session_if_enabled(
            runtime.clone(),
            &secret_store,
            vault,
            &account,
            status,
        )
        .await;

        let existing_connection = signal_store
            .find_connection_by_account("whatsapp", &account.account_id)
            .await
            .map_err(|error| error.to_string())?;
        let snapshot_changed =
            whatsapp_runtime_snapshot_changed(existing_connection.as_ref(), &status);

        crate::application::whatsapp_runtime_event_projection::sync_whatsapp_runtime_signal_connection_for_pool(
            pool,
            &account,
            &status,
            status.session_secret_ref.clone(),
        )
        .await
        .map_err(|error| error.to_string())?;

        if !snapshot_changed {
            continue;
        }

        capture_whatsapp_runtime_lifecycle_signal(&fixture_ingest, &status, event_source).await?;
        publish_whatsapp_runtime_status_event(&event_store, event_bus, &status, event_source)
            .await?;
        publish_whatsapp_session_link_state_event(
            &event_store,
            event_bus,
            &status.account_id,
            &status.provider_shape,
            &status.runtime_kind,
            &status.status,
            event_source,
            status.updated_at,
        )
        .await?;
    }

    Ok(())
}

async fn restore_whatsapp_runtime_from_vault_session_if_enabled(
    runtime: crate::application::provider_runtime_services::WhatsAppProviderRuntimeRef,
    secret_store: &crate::platform::secrets::SecretReferenceStore,
    vault: &HostVault,
    account: &hermes_communications_api::accounts::ProviderAccount,
    status: crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
) -> (
    crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
    &'static str,
) {
    if !should_start_whatsapp_runtime_from_restored_session(account, &status) {
        return (status, "startup_restore_reconcile");
    }
    let request = crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStartRequest {
        account_id: status.account_id.clone(),
    };
    match runtime.start_runtime(secret_store, vault, &request).await {
        Ok(started_status) => (started_status, "startup_restore_start"),
        Err(error) => {
            tracing::warn!(
                error = %error,
                account_id = %status.account_id,
                provider_shape = %status.provider_shape,
                "whatsapp startup restore failed to start provider runtime"
            );
            (
                whatsapp_startup_restore_failed_status(status),
                "startup_restore_start_failed",
            )
        }
    }
}

fn should_start_whatsapp_runtime_from_restored_session(
    account: &hermes_communications_api::accounts::ProviderAccount,
    status: &crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
) -> bool {
    status.provider_shape == "whatsapp_native_md"
        && status.session_restore_available
        && native_md_startup_restore_enabled(&account.config)
        && !status
            .runtime_blockers
            .iter()
            .any(|blocker| blocker == WHATSAPP_NATIVE_MD_RUNTIME_FEATURE_DISABLED_BLOCKER)
}

fn native_md_startup_restore_enabled(config: &Value) -> bool {
    config
        .get(WHATSAPP_NATIVE_MD_STARTUP_RESTORE_CONFIG_KEY)
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || config
            .get(WHATSAPP_NATIVE_MD_STARTUP_RESTORE_ALIAS_CONFIG_KEY)
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

fn whatsapp_startup_restore_failed_status(
    mut status: crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
) -> crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus {
    status.status = "degraded".to_owned();
    status.live_runtime_available = false;
    status.live_send_available = false;
    status.media_download_available = false;
    status.media_upload_available = false;
    if !status
        .runtime_blockers
        .iter()
        .any(|blocker| blocker == WHATSAPP_STARTUP_RESTORE_FAILED_BLOCKER)
    {
        status
            .runtime_blockers
            .push(WHATSAPP_STARTUP_RESTORE_FAILED_BLOCKER.to_owned());
    }
    status.last_error = Some(WHATSAPP_STARTUP_RESTORE_FAILED_BLOCKER.to_owned());
    status.updated_at = Utc::now();
    status
}

fn should_reconcile_whatsapp_runtime_restore(
    status: &crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
) -> bool {
    status.session_restore_available || matches!(status.status.as_str(), "available" | "linked")
}

fn whatsapp_runtime_snapshot_changed(
    existing_connection: Option<&crate::domains::signal_hub::store::SignalConnection>,
    status: &crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
) -> bool {
    let Some(connection) = existing_connection else {
        return true;
    };
    let stored_last_error = connection
        .settings
        .get("whatsapp_last_error")
        .cloned()
        .unwrap_or(Value::Null);
    connection
        .settings
        .get("whatsapp_runtime_status")
        .and_then(Value::as_str)
        != Some(status.status.as_str())
        || connection
            .settings
            .get("whatsapp_provider_shape")
            .and_then(Value::as_str)
            != Some(status.provider_shape.as_str())
        || connection
            .settings
            .get("whatsapp_runtime_kind")
            .and_then(Value::as_str)
            != Some(status.runtime_kind.as_str())
        || connection
            .settings
            .get("whatsapp_session_restore_available")
            .and_then(Value::as_bool)
            != Some(status.session_restore_available)
        || stored_last_error != json!(status.last_error)
}

async fn publish_whatsapp_runtime_status_event(
    event_store: &hermes_events_postgres::store::EventStore,
    event_bus: &InMemoryEventBus,
    status: &crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
    source: &str,
) -> Result<(), String> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        status.account_id,
        source,
        status.status,
        status.updated_at.timestamp_micros()
    );
    let event = hermes_events_api::NewEventEnvelope::builder(
        whatsapp_event_id("runtime", &status.account_id, now),
        crate::platform::events::bus::whatsapp_event_types::RUNTIME_STATUS_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": status.account_id,
            "actor_id": "hermes-frontend",
            "kind": "whatsapp_runtime_status",
            "source_id": source_id,
        }),
        json!({
            "id": status.account_id,
            "entity_id": status.account_id,
            "kind": "whatsapp_runtime",
        }),
    )
    .payload(crate::platform::events::bus::sanitize_event_payload(
        json!({
            "account_id": status.account_id,
            "provider_kind": status.provider_kind,
            "provider_shape": status.provider_shape,
            "runtime_kind": status.runtime_kind,
            "status": status.status,
            "fixture_runtime": status.fixture_runtime,
            "live_runtime_available": status.live_runtime_available,
            "live_send_available": status.live_send_available,
            "qr_pairing_available": status.qr_pairing_available,
            "pair_code_available": status.pair_code_available,
            "media_download_available": status.media_download_available,
            "media_upload_available": status.media_upload_available,
            "session_restore_available": status.session_restore_available,
            "runtime_blockers": status.runtime_blockers,
            "last_error": status.last_error,
            "source": source,
        }),
    ))
    .build()
    .map_err(|error| error.to_string())?;
    event_store
        .append(&event)
        .await
        .map_err(|error| error.to_string())?;
    let _ = event_bus.broadcast(event);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn publish_whatsapp_session_link_state_event(
    event_store: &hermes_events_postgres::store::EventStore,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    provider_shape: &str,
    runtime_kind: &str,
    link_state: &str,
    source: &str,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), String> {
    let now = Utc::now();
    let source_id = format!(
        "{}:{}:{}:{}",
        account_id,
        source,
        link_state,
        observed_at.timestamp_micros()
    );
    let event = hermes_events_api::NewEventEnvelope::builder(
        whatsapp_event_id("session", account_id, now),
        crate::platform::events::bus::whatsapp_event_types::SESSION_LINK_STATE_CHANGED.to_owned(),
        now,
        json!({
            "channel": "whatsapp",
            "account_id": account_id,
            "actor_id": "hermes-frontend",
            "kind": "whatsapp_session_link_state",
            "source_id": source_id,
        }),
        json!({
            "id": account_id,
            "entity_id": account_id,
            "kind": "whatsapp_session",
        }),
    )
    .payload(crate::platform::events::bus::sanitize_event_payload(
        json!({
            "account_id": account_id,
            "provider_shape": provider_shape,
            "runtime_kind": runtime_kind,
            "link_state": link_state,
            "source": source,
        }),
    ))
    .build()
    .map_err(|error| error.to_string())?;
    event_store
        .append(&event)
        .await
        .map_err(|error| error.to_string())?;
    let _ = event_bus.broadcast(event);
    Ok(())
}

async fn capture_whatsapp_runtime_lifecycle_signal(
    fixture_ingest: &crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService,
    status: &crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus,
    source: &str,
) -> Result<(), String> {
    let provider_event_id = format!(
        "{}:{}:{}",
        status.account_id,
        source,
        status.updated_at.timestamp_micros()
    );
    let metadata = json!({
        "source": source,
        "provider_kind": status.provider_kind,
        "provider_shape": status.provider_shape,
        "runtime_kind": status.runtime_kind,
        "fixture_runtime": status.fixture_runtime,
        "live_runtime_available": status.live_runtime_available,
        "live_send_available": status.live_send_available,
        "qr_pairing_available": status.qr_pairing_available,
        "pair_code_available": status.pair_code_available,
        "media_download_available": status.media_download_available,
        "media_upload_available": status.media_upload_available,
        "session_restore_available": status.session_restore_available,
        "runtime_blockers": status.runtime_blockers,
        "last_error": status.last_error,
    });
    fixture_ingest
        .capture_runtime_lifecycle_event(
            &status.account_id,
            &provider_event_id,
            "runtime.status_changed",
            Some(&status.status),
            Some(&status.status),
            Some(
                if status.status == "available" || status.status == "linked" {
                    "info"
                } else if status.status == "degraded" {
                    "warning"
                } else {
                    "blocked"
                },
            ),
            metadata,
            source,
            status.updated_at,
        )
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn whatsapp_event_id(scope: &str, entity: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    format!(
        "evt_whatsapp_{}_{}_{}_{}",
        scope,
        entity,
        now.timestamp_micros(),
        Uuid::now_v7()
    )
}
