use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::platform::events::EventBus;
use crate::vault::{HostVault, VaultMode};

static MAIL_BACKGROUND_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_OUTBOX_DELIVERY_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static TELEGRAM_COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_RUNTIME_RESTORE_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_TOKEN_MAINTENANCE_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_RECORDING_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_RETENTION_CLEANUP_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_RUNTIME_EVENT_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSON_DERIVED_EVIDENCE_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_SIGNAL_DETECTION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_CALENDAR_MATCHING_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_PARTICIPANT_IDENTITY_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSON_IDENTITY_REVIEW_INBOX_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PROJECT_LINK_REVIEW_EFFECTS_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_HUB_RAW_SIGNAL_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static EVENT_OUTBOX_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_REPLAY_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const MAIL_BACKGROUND_SYNC_RUNTIME: &str = "mail_background_sync";
const MAIL_OUTBOX_DELIVERY_RUNTIME: &str = "mail_outbox_delivery";
const TELEGRAM_COMMAND_EXECUTOR_RUNTIME: &str = "telegram_command_executor";
const WHATSAPP_COMMAND_EXECUTOR_RUNTIME: &str = "whatsapp_command_executor";
const WHATSAPP_RUNTIME_RESTORE_RECONCILIATION_RUNTIME: &str =
    "whatsapp_runtime_restore_reconciliation";
const WHATSAPP_NATIVE_MD_STARTUP_RESTORE_CONFIG_KEY: &str = "native_md_live_smoke_enabled";
const WHATSAPP_NATIVE_MD_STARTUP_RESTORE_ALIAS_CONFIG_KEY: &str =
    "whatsapp_native_md_live_smoke_enabled";
const WHATSAPP_NATIVE_MD_RUNTIME_FEATURE_DISABLED_BLOCKER: &str =
    "whatsapp_native_md_runtime_feature_disabled";
const WHATSAPP_STARTUP_RESTORE_FAILED_BLOCKER: &str = "whatsapp_startup_restore_failed";
const ZOOM_TOKEN_MAINTENANCE_RUNTIME: &str = "zoom_token_maintenance";
const ZOOM_TOKEN_MAINTENANCE_TICK_SECONDS: u64 = 60;
const ZOOM_TOKEN_MAINTENANCE_REFRESH_EXPIRING_WITHIN_SECONDS: i64 = 300;
const ZOOM_RECORDING_SYNC_RUNTIME: &str = "zoom_recording_sync";
const ZOOM_RECORDING_SYNC_TICK_SECONDS: u64 = 300;
const ZOOM_RECORDING_SYNC_LOOKBACK_DAYS: i64 = 7;
const ZOOM_RETENTION_CLEANUP_RUNTIME: &str = "zoom_retention_cleanup";
const ZOOM_RETENTION_CLEANUP_TICK_SECONDS: u64 = 3600;
const ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT: i64 = 100;
const WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME: &str = "whatsapp_runtime_event_projection";
const WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME: &str =
    "whatsapp_provider_observation_reconciliation";
const COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME: &str =
    "communication_provider_observation_projection";
const PERSON_DERIVED_EVIDENCE_RUNTIME: &str = "person_derived_evidence";
const ZOOM_SIGNAL_DETECTION_RUNTIME: &str = "zoom_signal_detection";
const ZOOM_CALENDAR_MATCHING_RUNTIME: &str = "zoom_calendar_matching";
const ZOOM_PARTICIPANT_IDENTITY_RUNTIME: &str = "zoom_participant_identity";
const PERSON_IDENTITY_REVIEW_INBOX_RUNTIME: &str = "person_identity_review_inbox";
const PROJECT_LINK_REVIEW_EFFECTS_RUNTIME: &str = "project_link_review_effects";
const SIGNAL_HUB_RAW_SIGNAL_RUNTIME: &str = "signal_hub_raw_signal_dispatcher";
const EVENT_OUTBOX_DISPATCHER_RUNTIME: &str = "event_outbox_dispatcher";
const SIGNAL_REPLAY_DISPATCHER_RUNTIME: &str = "signal_replay_dispatcher";

#[derive(Clone)]
pub(crate) struct ApplicationBootstrapContext {
    pub(crate) pool: Option<PgPool>,
    pub(crate) database_url: Option<String>,
    pub(crate) nats_server_url: Option<String>,
    pub(crate) zoom_token_maintenance_scheduler_enabled: bool,
    pub(crate) zoom_recording_sync_scheduler_enabled: bool,
    pub(crate) zoom_retention_cleanup_scheduler_enabled: bool,
    pub(crate) vault: HostVault,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: EventBus,
}

pub(crate) fn start_background_services(context: ApplicationBootstrapContext) {
    start_mail_background_sync(context.clone());
    start_mail_outbox_delivery(context.clone());
    start_telegram_command_executor(context.clone());
    start_whatsapp_command_executor(context.clone());
    start_whatsapp_runtime_restore_reconciliation(context.clone());
    start_zoom_token_maintenance(context.clone());
    start_zoom_recording_sync(context.clone());
    start_zoom_retention_cleanup(context.clone());
    start_whatsapp_runtime_event_projection(context.clone());
    start_whatsapp_provider_observation_reconciliation(context.clone());
    start_communication_provider_observation_projection(context.clone());
    start_person_derived_evidence_projection(context.clone());
    start_zoom_signal_detection_projection(context.clone());
    start_zoom_calendar_matching_projection(context.clone());
    start_zoom_participant_identity_projection(context.clone());
    start_person_identity_review_inbox_projection(context.clone());
    start_project_link_review_effects_projection(context.clone());
    start_signal_hub_raw_signal_dispatcher(context.clone());
    start_event_outbox_dispatcher(context.clone());
    start_signal_replay_dispatcher(context);
}

fn start_mail_background_sync(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_mail_background_sync_scheduler(&database_url) {
        return;
    }
    let vault = context.vault;

    tokio::spawn(async move {
        let store = crate::application::mail_background_sync::MailSyncStore::new(pool.clone());
        let service = crate::application::mail_background_sync::MailBackgroundSyncService::new(
            pool.clone(),
            vault.clone(),
            crate::application::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT,
            Arc::new(
                crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort::new(
                    pool.clone(),
                    vault,
                    Arc::new(
                        crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                            pool.clone(),
                        ),
                    ),
                    crate::application::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                ),
            ),
        );
        if let Err(error) = store.mark_orphaned_active_runs_failed(Utc::now()).await {
            tracing::warn!(error = %error, "mail background sync startup recovery failed");
        }
        let mut tick = tokio::time::interval(Duration::from_secs(30));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "mail",
                MAIL_BACKGROUND_SYNC_RUNTIME,
                json!({
                    "label": "Mail background sync",
                    "scope": "scheduler",
                }),
            )
            .await
            {
                continue;
            }
            if let Err(error) = service.run_due_accounts().await {
                tracing::warn!(error = %error, "mail background sync scheduler tick failed");
            }
        }
    });
}

fn start_mail_outbox_delivery(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_mail_outbox_delivery_scheduler(&database_url) {
        return;
    }
    let vault = context.vault;

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(10));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "mail",
                MAIL_OUTBOX_DELIVERY_RUNTIME,
                json!({
                    "label": "Mail outbox delivery",
                    "scope": "scheduler",
                }),
            )
            .await
            {
                continue;
            }
            if !host_vault_is_unlocked(&vault) {
                continue;
            }
            let store =
                crate::domains::communications::outbox::CommunicationOutboxStore::new(pool.clone());
            let sender =
                crate::domains::communications::outbox::CommunicationOutboxEmailSender::new(
                    pool.clone(),
                    vault.clone(),
                    crate::integrations::mail::send::smtp_outbox_transport(),
                    crate::integrations::mail::outbox::gmail_outbox_transport(
                        pool.clone(),
                        vault.clone(),
                    ),
                );
            let worker = crate::domains::communications::outbox::EmailOutboxDeliveryWorker::new(
                store, sender,
            );
            match worker.deliver_due(Utc::now(), 25).await {
                Ok(report) if report.claimed > 0 => {
                    tracing::info!(
                        claimed = report.claimed,
                        sent = report.sent,
                        failed = report.failed,
                        retried = report.retried,
                        "mail outbox delivery scheduler tick completed"
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(error = %error, "mail outbox delivery scheduler tick failed");
                }
            }
        }
    });
}

fn start_telegram_command_executor(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_telegram_command_executor(&database_url) {
        return;
    }
    let runtime_pool = pool.clone();
    let runtime = context.telegram_runtime;
    let event_bus = context.event_bus;
    let telegram_store = crate::integrations::telegram::client::TelegramStore::new(
        pool.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::messages::ProviderChannelMessageStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone()),
        ),
        Arc::new(
            crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool.clone(),
            ),
        ),
    );

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &runtime_pool,
                "telegram",
                TELEGRAM_COMMAND_EXECUTOR_RUNTIME,
                json!({
                    "label": "Telegram command executor",
                    "scope": "runtime",
                }),
            )
            .await
            {
                continue;
            }
            crate::integrations::telegram::runtime::execute_queued_commands(
                &telegram_store,
                &runtime,
                &event_bus,
                10,
            )
            .await;
        }
    });
}

fn start_whatsapp_command_executor(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_whatsapp_command_executor(&database_url) {
        return;
    }
    let runtime_pool = pool.clone();
    let vault = context.vault;
    let event_bus = context.event_bus;
    let runtime = crate::application::whatsapp_provider_runtime(pool.clone());
    let event_store = crate::platform::events::EventStore::new(pool.clone());

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &runtime_pool,
                "whatsapp",
                WHATSAPP_COMMAND_EXECUTOR_RUNTIME,
                json!({
                    "label": "WhatsApp command executor",
                    "scope": "runtime",
                }),
            )
            .await
            {
                continue;
            }
            crate::application::execute_due_fixture_commands(
                pool.clone(),
                runtime.clone(),
                event_store.clone(),
                event_bus.clone(),
                10,
            )
            .await;
            crate::application::execute_due_live_native_md_commands(
                pool.clone(),
                runtime.clone(),
                vault.clone(),
                event_store.clone(),
                event_bus.clone(),
                10,
            )
            .await;
            crate::application::execute_due_live_business_cloud_commands(
                pool.clone(),
                runtime.clone(),
                vault.clone(),
                event_store.clone(),
                event_bus.clone(),
                10,
            )
            .await;
        }
    });
}

fn start_whatsapp_runtime_restore_reconciliation(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_whatsapp_runtime_restore_reconciliation(&database_url) {
        return;
    }
    let runtime_pool = pool.clone();
    let vault = context.vault;
    let event_bus = context.event_bus;
    let runtime = crate::application::whatsapp_provider_runtime(pool.clone());

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &runtime_pool,
                "whatsapp",
                WHATSAPP_RUNTIME_RESTORE_RECONCILIATION_RUNTIME,
                json!({
                    "label": "WhatsApp runtime restore reconciliation",
                    "scope": "runtime",
                }),
            )
            .await
            {
                continue;
            }
            if !host_vault_is_unlocked(&vault) {
                continue;
            }
            if let Err(error) =
                reconcile_whatsapp_runtime_restore_once(&pool, &vault, &event_bus, runtime.clone())
                    .await
            {
                tracing::warn!(
                    error = %error,
                    "whatsapp runtime restore reconciliation tick failed"
                );
            }
        }
    });
}

fn start_zoom_token_maintenance(context: ApplicationBootstrapContext) {
    if !context.zoom_token_maintenance_scheduler_enabled {
        return;
    }
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_zoom_token_maintenance_scheduler(&database_url) {
        return;
    }
    let vault = context.vault;
    let event_bus = context.event_bus;

    tokio::spawn(async move {
        let mut tick =
            tokio::time::interval(Duration::from_secs(ZOOM_TOKEN_MAINTENANCE_TICK_SECONDS));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "zoom",
                ZOOM_TOKEN_MAINTENANCE_RUNTIME,
                json!({
                    "label": "Zoom token maintenance",
                    "scope": "scheduler",
                }),
            )
            .await
            {
                continue;
            }
            if !host_vault_is_unlocked(&vault) {
                continue;
            }
            match run_zoom_token_maintenance_once(&pool, &vault, &event_bus).await {
                Ok(result)
                    if result.checked_count > 0
                        || result.refreshed_count > 0
                        || result.failed_count > 0 =>
                {
                    tracing::info!(
                        checked = result.checked_count,
                        refreshed = result.refreshed_count,
                        skipped = result.skipped_count,
                        failed = result.failed_count,
                        refresh_expiring_within_seconds = result.refresh_expiring_within_seconds,
                        "zoom token maintenance scheduler tick completed"
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        "zoom token maintenance scheduler tick failed"
                    );
                }
            }
        }
    });
}

fn start_zoom_recording_sync(context: ApplicationBootstrapContext) {
    if !context.zoom_recording_sync_scheduler_enabled {
        return;
    }
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_zoom_recording_sync_scheduler(&database_url) {
        return;
    }
    let vault = context.vault;
    let event_bus = context.event_bus;

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(ZOOM_RECORDING_SYNC_TICK_SECONDS));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "zoom",
                ZOOM_RECORDING_SYNC_RUNTIME,
                json!({
                    "label": "Zoom recording sync",
                    "scope": "scheduler",
                }),
            )
            .await
            {
                continue;
            }
            if !host_vault_is_unlocked(&vault) {
                continue;
            }
            match run_zoom_recording_sync_once(&pool, &vault, &event_bus).await {
                Ok(result)
                    if result.accounts_checked > 0
                        || result.accounts_synced > 0
                        || result.failed_count > 0
                        || result.meetings_recorded > 0
                        || result.recordings_recorded > 0
                        || result.media_downloads_recorded > 0
                        || result.transcripts_recorded > 0 =>
                {
                    tracing::info!(
                        accounts_checked = result.accounts_checked,
                        accounts_synced = result.accounts_synced,
                        accounts_skipped = result.accounts_skipped,
                        failed = result.failed_count,
                        meetings_recorded = result.meetings_recorded,
                        recordings_recorded = result.recordings_recorded,
                        media_downloads_recorded = result.media_downloads_recorded,
                        transcripts_recorded = result.transcripts_recorded,
                        lookback_days = result.lookback_days,
                        "zoom recording sync scheduler tick completed"
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(error = %error, "zoom recording sync scheduler tick failed");
                }
            }
        }
    });
}

fn start_zoom_retention_cleanup(context: ApplicationBootstrapContext) {
    if !context.zoom_retention_cleanup_scheduler_enabled {
        return;
    }
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_zoom_retention_cleanup_scheduler(&database_url) {
        return;
    }
    let event_bus = context.event_bus;

    tokio::spawn(async move {
        let mut tick =
            tokio::time::interval(Duration::from_secs(ZOOM_RETENTION_CLEANUP_TICK_SECONDS));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "zoom",
                ZOOM_RETENTION_CLEANUP_RUNTIME,
                json!({
                    "label": "Zoom retention cleanup",
                    "scope": "scheduler",
                }),
            )
            .await
            {
                continue;
            }
            match run_zoom_retention_cleanup_once(&pool, &event_bus).await {
                Ok(result)
                    if result.accounts_checked > 0
                        || result.accounts_cleaned > 0
                        || result.recordings_removed > 0
                        || result.transcripts_removed > 0 =>
                {
                    tracing::info!(
                        accounts_checked = result.accounts_checked,
                        accounts_cleaned = result.accounts_cleaned,
                        recordings_removed = result.recordings_removed,
                        transcripts_removed = result.transcripts_removed,
                        limit_per_account = result.limit_per_account,
                        "zoom retention cleanup scheduler tick completed"
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        "zoom retention cleanup scheduler tick failed"
                    );
                }
            }
        }
    });
}

fn start_whatsapp_runtime_event_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_whatsapp_runtime_event_consumer(&database_url) {
        return;
    }
    let vault = context.vault;

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::WHATSAPP_RUNTIME_EVENT_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "whatsapp",
                WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME,
                json!({
                    "label": "WhatsApp runtime-event projection consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            let handler_vault = vault.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    let handler_pool = handler_pool.clone();
                    let handler_vault = handler_vault.clone();
                    async move {
                        crate::application::project_whatsapp_runtime_event(
                            handler_pool.clone(),
                            handler_vault.clone(),
                            event,
                        )
                        .await
                        .map_err(crate::platform::events::EventStoreError::ConsumerHandlerFailed)
                    }
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "whatsapp runtime-event projection consumer tick failed"
                );
            }
        }
    });
}

fn start_whatsapp_provider_observation_reconciliation(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_whatsapp_provider_observation_reconciliation_consumer(&database_url) {
        return;
    }
    let event_bus = context.event_bus;

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "whatsapp",
                WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME,
                json!({
                    "label": "WhatsApp provider observation reconciliation consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            let handler_event_bus = event_bus.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    let handler_pool = handler_pool.clone();
                    let handler_event_bus = handler_event_bus.clone();
                    async move {
                        crate::application::reconcile_whatsapp_provider_observation_event(
                            handler_pool.clone(),
                            handler_event_bus.clone(),
                            event,
                        )
                        .await
                        .map_err(crate::platform::events::EventStoreError::ConsumerHandlerFailed)
                    }
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "whatsapp provider observation reconciliation consumer tick failed"
                );
            }
        }
    });
}

fn start_communication_provider_observation_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_communication_provider_observation_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::domains::communications::messages::COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "system",
                COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME,
                json!({
                    "label": "Communications accepted-signal consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::domains::communications::messages::project_provider_observation_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "communication provider observation projection consumer tick failed"
                );
            }
        }
    });
}

fn start_person_derived_evidence_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_person_derived_evidence_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::PERSON_DERIVED_EVIDENCE_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "system",
                PERSON_DERIVED_EVIDENCE_RUNTIME,
                json!({
                    "label": "Person derived evidence consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::application::project_person_derived_evidence_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "person derived evidence projection consumer tick failed"
                );
            }
        }
    });
}

fn start_zoom_calendar_matching_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_zoom_calendar_matching_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::ZOOM_CALENDAR_MATCHING_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "zoom",
                ZOOM_CALENDAR_MATCHING_RUNTIME,
                json!({
                    "label": "Zoom calendar matching consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::application::project_zoom_calendar_matching_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "zoom calendar matching projection consumer tick failed"
                );
            }
        }
    });
}

fn start_zoom_signal_detection_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_zoom_signal_detection_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::ZOOM_SIGNAL_DETECTION_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "zoom",
                ZOOM_SIGNAL_DETECTION_RUNTIME,
                json!({
                    "label": "Zoom signal detection consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::application::project_zoom_signal_detection_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "zoom signal detection projection consumer tick failed"
                );
            }
        }
    });
}

fn start_person_identity_review_inbox_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_person_identity_review_inbox_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::PERSON_IDENTITY_REVIEW_INBOX_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "system",
                PERSON_IDENTITY_REVIEW_INBOX_RUNTIME,
                json!({
                    "label": "Person identity review inbox consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::application::project_person_identity_review_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "person identity review inbox projection consumer tick failed"
                );
            }
        }
    });
}

fn start_zoom_participant_identity_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_zoom_participant_identity_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::ZOOM_PARTICIPANT_IDENTITY_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "zoom",
                ZOOM_PARTICIPANT_IDENTITY_RUNTIME,
                json!({
                    "label": "Zoom participant identity consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::application::project_zoom_participant_identity_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "zoom participant identity projection consumer tick failed"
                );
            }
        }
    });
}

fn start_project_link_review_effects_projection(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_project_link_review_effects_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::application::PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "system",
                PROJECT_LINK_REVIEW_EFFECTS_RUNTIME,
                json!({
                    "label": "Project link review effects consumer",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::application::project_link_review_effect_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "project link review effects projection consumer tick failed"
                );
            }
        }
    });
}

fn start_signal_hub_raw_signal_dispatcher(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_signal_hub_raw_signal_consumer(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let runner = crate::platform::events::EventConsumerRunner::new(
            pool.clone(),
            crate::platform::events::EventConsumerConfig::new(
                crate::domains::signal_hub::SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
            ),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "system",
                SIGNAL_HUB_RAW_SIGNAL_RUNTIME,
                json!({
                    "label": "Signal Hub raw signal dispatcher",
                    "scope": "consumer",
                }),
            )
            .await
            {
                continue;
            }
            let handler_pool = pool.clone();
            if let Err(error) = runner
                .process_next_batch(|event| {
                    crate::domains::signal_hub::process_signal_hub_raw_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "signal hub raw signal dispatcher consumer tick failed"
                );
            }
        }
    });
}

fn start_event_outbox_dispatcher(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    let Some(nats_server_url) = context.nats_server_url else {
        tracing::info!(
            "event outbox dispatcher skipped because HERMES_NATS_SERVER_URL is not configured"
        );
        return;
    };
    if !register_event_outbox_dispatcher(&database_url) {
        return;
    }

    let realtime_bus = context.event_bus.clone();
    tokio::spawn(async move {
        let bus =
            match crate::platform::events::NatsJetStreamEventBus::connect(&nats_server_url).await {
                Ok(bus) => bus,
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        nats_server_url,
                        "event outbox dispatcher failed to initialize JetStream bus"
                    );
                    return;
                }
            };
        let dispatcher = crate::platform::events::EventOutboxDispatcher::new(
            crate::platform::events::EventStore::new(pool.clone()),
            bus,
        )
        .with_realtime_bus(realtime_bus);
        let mut tick = tokio::time::interval(Duration::from_secs(2));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "system",
                EVENT_OUTBOX_DISPATCHER_RUNTIME,
                json!({
                    "label": "Event outbox dispatcher",
                    "scope": "dispatcher",
                    "transport": "nats_jetstream",
                }),
            )
            .await
            {
                continue;
            }
            match dispatcher.dispatch_pending_once().await {
                Ok(report) if report.claimed > 0 || report.recovered > 0 => {
                    tracing::info!(
                        recovered = report.recovered,
                        claimed = report.claimed,
                        published = report.published,
                        retried = report.retried,
                        "event outbox dispatcher tick completed"
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(error = %error, "event outbox dispatcher tick failed");
                }
            }
        }
    });
}

fn start_signal_replay_dispatcher(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_signal_replay_dispatcher(&database_url) {
        return;
    }

    tokio::spawn(async move {
        let replay_service = crate::application::SignalHubReplayService::new(
            crate::domains::signal_hub::SignalHubStore::new(pool.clone()),
            crate::platform::events::EventStore::new(pool.clone()),
        );
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "system",
                SIGNAL_REPLAY_DISPATCHER_RUNTIME,
                json!({
                    "label": "Signal replay dispatcher",
                    "scope": "dispatcher",
                }),
            )
            .await
            {
                continue;
            }

            match replay_service.process_next_request().await {
                Ok(Some(report)) => {
                    tracing::info!(
                        request_id = %report.request_id,
                        replayed_count = report.replayed_count,
                        "signal replay dispatcher tick completed"
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(error = %error, "signal replay dispatcher tick failed");
                }
            }
        }
    });
}

fn register_telegram_command_executor(database_url: &str) -> bool {
    match TELEGRAM_COMMAND_EXECUTOR_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "telegram command executor registry is unavailable"
            );
            false
        }
    }
}

fn register_whatsapp_command_executor(database_url: &str) -> bool {
    match WHATSAPP_COMMAND_EXECUTOR_DATABASES.lock() {
        Ok(mut urls) => urls.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp command executor registry is unavailable"
            );
            false
        }
    }
}

fn register_whatsapp_runtime_restore_reconciliation(database_url: &str) -> bool {
    match WHATSAPP_RUNTIME_RESTORE_RECONCILIATION_DATABASES.lock() {
        Ok(mut urls) => urls.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp runtime restore reconciliation registry is unavailable"
            );
            false
        }
    }
}

fn register_whatsapp_runtime_event_consumer(database_url: &str) -> bool {
    match WHATSAPP_RUNTIME_EVENT_CONSUMER_DATABASES.lock() {
        Ok(mut urls) => urls.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp runtime-event projection consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_whatsapp_provider_observation_reconciliation_consumer(database_url: &str) -> bool {
    match WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_DATABASES.lock() {
        Ok(mut urls) => urls.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "whatsapp provider observation reconciliation consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_communication_provider_observation_consumer(database_url: &str) -> bool {
    match COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "communication provider observation consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_person_derived_evidence_consumer(database_url: &str) -> bool {
    match PERSON_DERIVED_EVIDENCE_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "person derived evidence consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_zoom_signal_detection_consumer(database_url: &str) -> bool {
    match ZOOM_SIGNAL_DETECTION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom signal detection consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_zoom_calendar_matching_consumer(database_url: &str) -> bool {
    match ZOOM_CALENDAR_MATCHING_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom calendar matching consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_zoom_participant_identity_consumer(database_url: &str) -> bool {
    match ZOOM_PARTICIPANT_IDENTITY_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom participant identity consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_person_identity_review_inbox_consumer(database_url: &str) -> bool {
    match PERSON_IDENTITY_REVIEW_INBOX_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "person identity review inbox consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_project_link_review_effects_consumer(database_url: &str) -> bool {
    match PROJECT_LINK_REVIEW_EFFECTS_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "project link review effects consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_signal_hub_raw_signal_consumer(database_url: &str) -> bool {
    match SIGNAL_HUB_RAW_SIGNAL_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "signal hub raw signal consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_event_outbox_dispatcher(database_url: &str) -> bool {
    match EVENT_OUTBOX_DISPATCHER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "event outbox dispatcher registry is unavailable"
            );
            false
        }
    }
}

fn register_signal_replay_dispatcher(database_url: &str) -> bool {
    match SIGNAL_REPLAY_DISPATCHER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "signal replay dispatcher registry is unavailable"
            );
            false
        }
    }
}

fn register_mail_background_sync_scheduler(database_url: &str) -> bool {
    match MAIL_BACKGROUND_SYNC_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail background sync scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_mail_outbox_delivery_scheduler(database_url: &str) -> bool {
    match MAIL_OUTBOX_DELIVERY_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail outbox delivery scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_zoom_token_maintenance_scheduler(database_url: &str) -> bool {
    match ZOOM_TOKEN_MAINTENANCE_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom token maintenance scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_zoom_recording_sync_scheduler(database_url: &str) -> bool {
    match ZOOM_RECORDING_SYNC_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom recording sync scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_zoom_retention_cleanup_scheduler(database_url: &str) -> bool {
    match ZOOM_RETENTION_CLEANUP_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "zoom retention cleanup scheduler registry is unavailable"
            );
            false
        }
    }
}

fn host_vault_is_unlocked(vault: &HostVault) -> bool {
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

async fn runtime_allows_processing(
    pool: &PgPool,
    source_code: &str,
    runtime_kind: &str,
    metadata: serde_json::Value,
) -> bool {
    let store = crate::domains::signal_hub::SignalHubStore::new(pool.clone());
    if let Err(error) = store.restore_system_sources().await {
        tracing::warn!(
            error = %error,
            source_code,
            runtime_kind,
            "signal hub system source restore failed during runtime gate check"
        );
        return true;
    }

    match crate::platform::events::runtime_allows_processing(
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

async fn run_zoom_token_maintenance_once(
    pool: &PgPool,
    vault: &HostVault,
    event_bus: &EventBus,
) -> Result<crate::application::provider_runtime_contracts::ZoomTokenMaintenanceResult, String> {
    let secret_store = crate::platform::secrets::SecretReferenceStore::new(pool.clone());
    let service =
        crate::application::zoom_provider_runtime_service(pool.clone(), event_bus.clone());
    let request = crate::application::provider_runtime_contracts::ZoomTokenMaintenanceRequest {
        account_id: None,
        force: false,
        refresh_expiring_within_seconds: Some(
            ZOOM_TOKEN_MAINTENANCE_REFRESH_EXPIRING_WITHIN_SECONDS,
        ),
    };
    service
        .maintain_tokens(&secret_store, vault, &request)
        .await
        .map_err(|error| error.to_string())
}

struct ZoomRecordingSyncSchedulerResult {
    accounts_checked: usize,
    accounts_synced: usize,
    accounts_skipped: usize,
    failed_count: usize,
    meetings_recorded: usize,
    recordings_recorded: usize,
    media_downloads_recorded: usize,
    transcripts_recorded: usize,
    lookback_days: i64,
}

struct ZoomRetentionCleanupSchedulerResult {
    accounts_checked: usize,
    accounts_cleaned: usize,
    recordings_removed: usize,
    transcripts_removed: usize,
    limit_per_account: i64,
}

async fn run_zoom_recording_sync_once(
    pool: &PgPool,
    vault: &HostVault,
    event_bus: &EventBus,
) -> Result<ZoomRecordingSyncSchedulerResult, String> {
    let settings = crate::platform::settings::ApplicationSettingsStore::new(pool.clone());
    let allow_remote_transcript_downloads = settings
        .setting("privacy.zoom_remote_transcript_download_enabled")
        .await
        .map_err(|error| error.to_string())?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false);
    let allow_remote_recording_downloads = settings
        .setting("privacy.zoom_remote_recording_download_enabled")
        .await
        .map_err(|error| error.to_string())?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false);

    let secret_store = crate::platform::secrets::SecretReferenceStore::new(pool.clone());
    let service =
        crate::application::zoom_provider_runtime_service(pool.clone(), event_bus.clone());
    let accounts = service
        .list_accounts(false)
        .await
        .map_err(|error| error.to_string())?
        .items;
    let today = Utc::now().date_naive();
    let from = (today - chrono::TimeDelta::days(ZOOM_RECORDING_SYNC_LOOKBACK_DAYS))
        .format("%Y-%m-%d")
        .to_string();
    let to = today.format("%Y-%m-%d").to_string();
    let mut result = ZoomRecordingSyncSchedulerResult {
        accounts_checked: 0,
        accounts_synced: 0,
        accounts_skipped: 0,
        failed_count: 0,
        meetings_recorded: 0,
        recordings_recorded: 0,
        media_downloads_recorded: 0,
        transcripts_recorded: 0,
        lookback_days: ZOOM_RECORDING_SYNC_LOOKBACK_DAYS,
    };

    for account in accounts {
        if !account.provider_kind.starts_with("zoom_") {
            continue;
        }
        result.accounts_checked += 1;
        let status = service
            .runtime_status(&account.account_id)
            .await
            .map_err(|error| error.to_string())?;
        if !should_run_zoom_recording_sync(&status) {
            result.accounts_skipped += 1;
            continue;
        }
        let request = crate::application::provider_runtime_contracts::ZoomRecordingSyncRequest {
            account_id: account.account_id.clone(),
            user_id: None,
            from: from.clone(),
            to: to.clone(),
            page_size: Some(
                crate::integrations::zoom::client::ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE,
            ),
            max_meetings: Some(
                crate::integrations::zoom::client::ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS,
            ),
            api_base_url: None,
        };
        match service
            .sync_recordings(
                &secret_store,
                vault,
                &request,
                allow_remote_recording_downloads,
                allow_remote_transcript_downloads,
            )
            .await
        {
            Ok(sync) => {
                result.accounts_synced += 1;
                result.meetings_recorded += sync.meetings_recorded;
                result.recordings_recorded += sync.recordings_recorded;
                result.media_downloads_recorded += sync.media_downloads_recorded;
                result.transcripts_recorded += sync.transcripts_recorded;
                if !sync.failures.is_empty() {
                    result.failed_count += 1;
                }
            }
            Err(error) => {
                result.failed_count += 1;
                tracing::warn!(
                    error = %error,
                    account_id = %account.account_id,
                    "zoom recording sync failed for authorized runtime account"
                );
            }
        }
    }

    Ok(result)
}

async fn run_zoom_retention_cleanup_once(
    pool: &PgPool,
    event_bus: &EventBus,
) -> Result<ZoomRetentionCleanupSchedulerResult, String> {
    let service =
        crate::application::zoom_provider_runtime_service(pool.clone(), event_bus.clone());
    let accounts = service
        .list_accounts(false)
        .await
        .map_err(|error| error.to_string())?
        .items;
    let mut result = ZoomRetentionCleanupSchedulerResult {
        accounts_checked: 0,
        accounts_cleaned: 0,
        recordings_removed: 0,
        transcripts_removed: 0,
        limit_per_account: ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
    };

    for account in accounts {
        if !account.provider_kind.starts_with("zoom_") {
            continue;
        }
        result.accounts_checked += 1;
        let response = service
            .cleanup_retention(
                &account.account_id,
                &crate::application::provider_runtime_contracts::ZoomRetentionCleanupRequest {
                    remove_recordings: true,
                    remove_transcripts: true,
                    limit: ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        if response.recordings_removed > 0 || response.transcripts_removed > 0 {
            result.accounts_cleaned += 1;
        }
        result.recordings_removed += response.recordings_removed;
        result.transcripts_removed += response.transcripts_removed;
    }

    Ok(result)
}

async fn reconcile_whatsapp_runtime_restore_once(
    pool: &PgPool,
    vault: &HostVault,
    event_bus: &EventBus,
    runtime: crate::application::provider_runtime_contracts::WhatsAppProviderRuntimeRef,
) -> Result<(), String> {
    let account_store =
        crate::domains::communications::core::CommunicationProviderAccountStore::new(pool.clone());
    let secret_store = crate::platform::secrets::SecretReferenceStore::new(pool.clone());
    let signal_store = crate::domains::signal_hub::SignalHubStore::new(pool.clone());
    let event_store = crate::platform::events::EventStore::new(pool.clone());
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

        crate::application::sync_whatsapp_runtime_signal_connection_for_pool(
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
    runtime: crate::application::provider_runtime_contracts::WhatsAppProviderRuntimeRef,
    secret_store: &crate::platform::secrets::SecretReferenceStore,
    vault: &HostVault,
    account: &crate::platform::communications::ProviderAccount,
    status: crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
) -> (
    crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
    &'static str,
) {
    if !should_start_whatsapp_runtime_from_restored_session(account, &status) {
        return (status, "startup_restore_reconcile");
    }
    let request = crate::application::provider_runtime_contracts::WhatsAppRuntimeStartRequest {
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
    account: &crate::platform::communications::ProviderAccount,
    status: &crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
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
    mut status: crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
) -> crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus {
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
    status: &crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
) -> bool {
    status.session_restore_available || matches!(status.status.as_str(), "available" | "linked")
}

fn should_run_zoom_recording_sync(
    status: &crate::application::provider_runtime_contracts::ZoomRuntimeStatus,
) -> bool {
    status.live_runtime_available
        && matches!(status.status.as_str(), "running" | "degraded")
        && !status
            .runtime_blockers
            .iter()
            .any(|blocker| blocker == "zoom_token_rotation_required")
}

fn whatsapp_runtime_snapshot_changed(
    existing_connection: Option<&crate::domains::signal_hub::SignalConnection>,
    status: &crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
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
    event_store: &crate::platform::events::EventStore,
    event_bus: &EventBus,
    status: &crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
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
    let event = crate::platform::events::NewEventEnvelope::builder(
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
    event_store: &crate::platform::events::EventStore,
    event_bus: &EventBus,
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
    let event = crate::platform::events::NewEventEnvelope::builder(
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
    status: &crate::application::provider_runtime_contracts::WhatsAppRuntimeStatus,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbox_delivery_scheduler_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://outbox-scheduler-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_mail_outbox_delivery_scheduler(&database_url));
        assert!(!register_mail_outbox_delivery_scheduler(&database_url));
    }

    #[test]
    fn event_outbox_dispatcher_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://event-outbox-dispatcher-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_event_outbox_dispatcher(&database_url));
        assert!(!register_event_outbox_dispatcher(&database_url));
    }

    #[test]
    fn signal_replay_dispatcher_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://signal-replay-dispatcher-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_signal_replay_dispatcher(&database_url));
        assert!(!register_signal_replay_dispatcher(&database_url));
    }

    #[test]
    fn zoom_token_maintenance_scheduler_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-token-maintenance-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_zoom_token_maintenance_scheduler(&database_url));
        assert!(!register_zoom_token_maintenance_scheduler(&database_url));
    }

    #[test]
    fn zoom_recording_sync_scheduler_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-recording-sync-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_zoom_recording_sync_scheduler(&database_url));
        assert!(!register_zoom_recording_sync_scheduler(&database_url));
    }

    #[test]
    fn zoom_retention_cleanup_scheduler_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-retention-cleanup-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_zoom_retention_cleanup_scheduler(&database_url));
        assert!(!register_zoom_retention_cleanup_scheduler(&database_url));
    }

    #[test]
    fn zoom_calendar_matching_consumer_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-calendar-matching-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_zoom_calendar_matching_consumer(&database_url));
        assert!(!register_zoom_calendar_matching_consumer(&database_url));
    }

    #[test]
    fn zoom_signal_detection_consumer_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-signal-detection-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_zoom_signal_detection_consumer(&database_url));
        assert!(!register_zoom_signal_detection_consumer(&database_url));
    }

    #[test]
    fn zoom_participant_identity_consumer_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-participant-identity-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_zoom_participant_identity_consumer(&database_url));
        assert!(!register_zoom_participant_identity_consumer(&database_url));
    }
}
