use hermes_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use chrono::Utc;
use hermes_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskError, RuntimeTaskFactory, RuntimeTaskFuture,
    RuntimeTaskSpec,
};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tokio_util::sync::CancellationToken;

use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::platform::config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::vault::{HostVault, VaultMode};

mod mail_ai;
mod telegram;
mod zoom;
pub(crate) mod zulip;

static MAIL_BACKGROUND_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_ATTACHMENT_SCAN_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ADDRESS_BOOK_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_OUTBOX_DELIVERY_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_PROVIDER_COMMAND_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_AI_PIPELINE_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static TELEGRAM_COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static TELEGRAM_RUNTIME_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
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
static PERSONA_DERIVED_EVIDENCE_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_SIGNAL_DETECTION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_CALENDAR_MATCHING_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_PARTICIPANT_IDENTITY_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static YANDEX_TELEMOST_RETENTION_CLEANUP_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER_DATABASES: LazyLock<
    Mutex<HashSet<String>>,
> = LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PROJECT_LINK_REVIEW_EFFECTS_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER_DATABASES: LazyLock<
    Mutex<HashSet<String>>,
> = LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_HUB_RAW_SIGNAL_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static EVENT_OUTBOX_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_REPLAY_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const MAIL_BACKGROUND_SYNC_RUNTIME: &str = "mail_background_sync";
const MAIL_ATTACHMENT_SCAN_RUNTIME: &str = "mail_attachment_scan";
const ADDRESS_BOOK_SYNC_RUNTIME: &str = "address_book_sync";
const MAIL_OUTBOX_DELIVERY_RUNTIME: &str = "mail_outbox_delivery";
const MAIL_PROVIDER_COMMAND_RUNTIME: &str = "mail_provider_commands";
const MAIL_AI_PIPELINE_RUNTIME: &str = "mail_ai_pipeline";
const TELEGRAM_COMMAND_EXECUTOR_RUNTIME: &str = "telegram_command_executor";
const TELEGRAM_RUNTIME_RECONCILIATION_RUNTIME: &str = "telegram_runtime_reconciliation";
const TELEGRAM_RUNTIME_RECONCILIATION_TICK_SECONDS: u64 = 15;
const TELEGRAM_RUNTIME_CHAT_SYNC_LIMIT: i64 = 100;
const TELEGRAM_RUNTIME_RECONNECT_INITIAL_DELAY_SECONDS: u64 = 5;
const TELEGRAM_RUNTIME_RECONNECT_MAX_DELAY_SECONDS: u64 = 300;
const ZOOM_TOKEN_MAINTENANCE_RUNTIME: &str = "zoom_token_maintenance";
const ZOOM_TOKEN_MAINTENANCE_TICK_SECONDS: u64 = 60;
const ZOOM_TOKEN_MAINTENANCE_REFRESH_EXPIRING_WITHIN_SECONDS: i64 = 300;
const ZOOM_RECORDING_SYNC_RUNTIME: &str = "zoom_recording_sync";
const ZOOM_RECORDING_SYNC_TICK_SECONDS: u64 = 300;
const ZOOM_RECORDING_SYNC_LOOKBACK_DAYS: i64 = 7;
const ZOOM_RETENTION_CLEANUP_RUNTIME: &str = "zoom_retention_cleanup";
const ZOOM_RETENTION_CLEANUP_TICK_SECONDS: u64 = 3600;
const ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT: i64 = 100;
const YANDEX_TELEMOST_RETENTION_CLEANUP_RUNTIME: &str = "yandex_telemost_retention_cleanup";
const YANDEX_TELEMOST_RETENTION_CLEANUP_TICK_SECONDS: u64 = 3600;
const YANDEX_TELEMOST_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT: i64 = 100;
const WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME: &str = "whatsapp_runtime_event_projection";
const WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME: &str =
    "whatsapp_provider_observation_reconciliation";
const COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME: &str =
    "communication_provider_observation_projection";
const PERSONA_DERIVED_EVIDENCE_RUNTIME: &str = "persona_derived_evidence";
const ZOOM_SIGNAL_DETECTION_RUNTIME: &str = "zoom_signal_detection";
const ZOOM_CALENDAR_MATCHING_RUNTIME: &str = "zoom_calendar_matching";
const ZOOM_PARTICIPANT_IDENTITY_RUNTIME: &str = "zoom_participant_identity";
const YANDEX_TELEMOST_CALENDAR_MATCHING_RUNTIME: &str = "yandex_telemost_calendar_matching";
const REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_RUNTIME: &str =
    "realtime_conversation_transcript_execution";
const PERSONA_IDENTITY_REVIEW_INBOX_RUNTIME: &str = "persona_identity_review_inbox";
const PROJECT_LINK_REVIEW_EFFECTS_RUNTIME: &str = "project_link_review_effects";
const REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_RUNTIME: &str =
    "realtime_conversation_transcript_projection";
const SIGNAL_HUB_RAW_SIGNAL_RUNTIME: &str = "signal_hub_raw_signal_dispatcher";
// Telegram and other inbound signals cross Signal Hub before they become
// canonical Communications messages. Keep that durable path responsive enough
// for an open conversation without tightening unrelated background workers.
const REALTIME_SIGNAL_PROJECTION_TICK_MILLIS: u64 = 250;
const EVENT_OUTBOX_DISPATCHER_RUNTIME: &str = "event_outbox_dispatcher";
const SIGNAL_REPLAY_DISPATCHER_RUNTIME: &str = "signal_replay_dispatcher";

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

pub(crate) fn start_background_services(context: ApplicationBootstrapContext) {
    let _ = context;
}

pub(crate) fn mail_runtime_task_specs(
    context: ApplicationBootstrapContext,
) -> Vec<RuntimeTaskSpec> {
    [
        mail_background_sync_task(context.clone()),
        mail_attachment_scan_task(context.clone()),
        crate::application::mail_imap_idle::runtime_task_spec(context.clone()),
        address_book_sync_task(context.clone()),
        mail_outbox_delivery_task(context.clone()),
        mail_provider_command_executor_task(context.clone()),
        mail_ai_pipeline_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("mail"))
    .collect()
}

fn mail_background_sync_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_background_sync_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let store = crate::workflows::mail_background_sync::MailSyncStore::new(pool.clone());
            let service = crate::workflows::mail_background_sync::MailBackgroundSyncService::new(
            pool.clone(),
            vault.clone(),
            crate::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT,
            Arc::new(
                crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort::new(
                    pool.clone(),
                    vault,
                    Arc::new(
                        hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                            pool.clone(),
                        ),
                    ),
                    crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                ),
            ),
            Arc::new(
                crate::domains::communications::provider_resources::MailProviderResourceStore::new(
                    pool.clone(),
                ),
            ),
            Arc::new(hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
            Arc::new(
                hermes_communications_postgres::store::CommunicationIngestionStore::new(
                    pool.clone(),
                ),
            ),
        );
            if let Err(error) = store.recover_interrupted_runs(Utc::now()).await {
                tracing::warn!(error = %error, "mail background sync startup recovery failed");
            }
            let mut tick = tokio::time::interval(Duration::from_secs(30));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_BACKGROUND_SYNC_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_attachment_scan_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_attachment_scan_worker(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let worker =
                crate::application::mail_attachment_scan_worker::MailAttachmentScanWorker::new(
                    pool.clone(),
                );
            let mut tick = tokio::time::interval(Duration::from_secs(60));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "mail",
                    MAIL_ATTACHMENT_SCAN_RUNTIME,
                    json!({
                        "label": "Mail attachment malware rescan",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                match worker.scan_due(50).await {
                    Ok(report)
                        if report.candidates_seen > 0
                            || report.verdicts_persisted > 0
                            || report.failures > 0 =>
                    {
                        tracing::info!(
                            candidates_seen = report.candidates_seen,
                            verdicts_persisted = report.verdicts_persisted,
                            retry_deferred = report.retry_deferred,
                            invalid_or_stale = report.invalid_or_stale,
                            failures = report.failures,
                            "mail attachment rescan tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "mail attachment rescan scheduler tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_ATTACHMENT_SCAN_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn address_book_sync_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_address_book_sync_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let service = crate::workflows::address_book_sync::AddressBookSyncService::new(
            pool.clone(),
            Arc::new(
                crate::integrations::mail::address_book_sync_provider::LiveAddressBookProviderSyncPort::new(
                    pool.clone(),
                    vault,
                    Arc::new(
                        hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                            pool.clone(),
                        ),
                    ),
                    crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                ),
            ),
            Arc::new(hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
        );
            let mut tick = tokio::time::interval(Duration::from_secs(300));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "mail",
                    ADDRESS_BOOK_SYNC_RUNTIME,
                    json!({
                        "label": "Address book sync",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                if let Err(error) = service.run_due_accounts().await {
                    tracing::warn!(error = %error, "address book sync scheduler tick failed");
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ADDRESS_BOOK_SYNC_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_outbox_delivery_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_outbox_delivery_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(10));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                let store = crate::domains::communications::outbox::CommunicationOutboxStore::new(
                    pool.clone(),
                );
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_OUTBOX_DELIVERY_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_provider_command_executor_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_provider_command_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;
    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let worker =
                crate::application::mail_provider_command_executor::MailProviderCommandWorker::new(
                    pool.clone(),
                    vault.clone(),
                    crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                );
            // Read-state intent is queued after the reader dwell threshold, so keep
            // provider reconciliation responsive without polling full mailbox sync.
            let mut tick = tokio::time::interval(Duration::from_secs(2));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                &pool,
                "mail",
                MAIL_PROVIDER_COMMAND_RUNTIME,
                json!({ "label": "Mail provider command reconciliation", "scope": "scheduler" }),
            )
            .await
                || !host_vault_is_unlocked(&vault)
            {
                continue;
            }
                if let Err(error) = worker.execute_due(Utc::now(), 25).await {
                    tracing::warn!(error = %error, "mail provider command scheduler tick failed");
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_PROVIDER_COMMAND_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_ai_pipeline_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_ai_pipeline(&database_url) {
        return None;
    }
    let config = context.config;
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let config = config.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(10));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "ai",
                    MAIL_AI_PIPELINE_RUNTIME,
                    json!({
                        "label": "Mail AI pipeline",
                        "scope": "worker",
                    }),
                )
                .await
                {
                    continue;
                }

                let mail_ai_runtime = mail_ai::mail_ai_hub_optional(&pool, &config, &vault).await;
                let (hub, external_body_egress_required) = match mail_ai_runtime {
                    Some((hub, external_body_egress_required)) => {
                        (Some(hub), external_body_egress_required)
                    }
                    None => (None, false),
                };
                let target_language = mail_ai::mail_ai_target_language(&pool).await;
                let service = crate::workflows::email_intelligence::pipeline::MailAiPipelineService::new(
                    pool.clone(),
                    hub,
                    target_language,
                    std::sync::Arc::new(
                        crate::domains::communications::sensitive_forwarding::SensitiveForwardingStore::new(
                            pool.clone(),
                        ),
                    ),
                )
                .requiring_external_body_egress(external_body_egress_required);
                match service.process_next_batch(10).await {
                    Ok(report)
                        if report.claimed > 0
                            || report.recovered > 0
                            || report.processed > 0
                            || report.failed > 0
                            || report.retrying > 0
                            || report.suppressed > 0 =>
                    {
                        tracing::info!(
                            claimed = report.claimed,
                            recovered = report.recovered,
                            processed = report.processed,
                            suppressed = report.suppressed,
                            failed = report.failed,
                            retrying = report.retrying,
                            review_candidates = report.review_candidates,
                            "mail AI pipeline tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "mail AI pipeline tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_AI_PIPELINE_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

pub(crate) fn telegram_runtime_task_specs(
    context: ApplicationBootstrapContext,
) -> Vec<RuntimeTaskSpec> {
    [
        telegram_command_executor_task(context.clone()),
        telegram_runtime_reconciliation_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("telegram"))
    .collect()
}

fn telegram_command_executor_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_telegram_command_executor(&database_url) {
        return None;
    }
    let runtime_pool = pool.clone();
    let runtime = context.telegram_runtime;
    let event_bus = context.event_bus;
    let telegram_store = crate::integrations::telegram::client::TelegramStore::new(
        pool.clone(),
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            hermes_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::messages::ProviderChannelMessageStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            hermes_communications_postgres::store::CommunicationIngestionStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool.clone(),
            ),
        ),
    );

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let runtime_pool = runtime_pool.clone();
        let runtime = runtime.clone();
        let event_bus = event_bus.clone();
        let telegram_store = telegram_store.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        TELEGRAM_COMMAND_EXECUTOR_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn telegram_runtime_reconciliation_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_telegram_runtime_reconciliation(&database_url) {
        return None;
    }

    let vault = context.vault;
    let config = context.config;
    let event_bus = context.event_bus;
    let runtime = context.telegram_runtime;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        let config = config.clone();
        let event_bus = event_bus.clone();
        let runtime = runtime.clone();
        Box::pin(async move {
            let mut reconnects = HashMap::<String, telegram::RuntimeReconnectState>::new();
            let mut tick = tokio::time::interval(Duration::from_secs(
                TELEGRAM_RUNTIME_RECONCILIATION_TICK_SECONDS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "telegram",
                    TELEGRAM_RUNTIME_RECONCILIATION_RUNTIME,
                    json!({
                        "label": "Telegram runtime reconciliation",
                        "scope": "runtime",
                    }),
                )
                .await
                {
                    continue;
                }

                if let Err(error) = telegram::reconcile_runtime_once(
                    &pool,
                    &vault,
                    &config,
                    &event_bus,
                    &runtime,
                    &mut reconnects,
                )
                .await
                {
                    tracing::warn!(
                        error = %error,
                        "telegram runtime reconciliation tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        TELEGRAM_RUNTIME_RECONCILIATION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

pub(crate) fn whatsapp_runtime_task_specs(
    context: ApplicationBootstrapContext,
) -> Vec<RuntimeTaskSpec> {
    [
        whatsapp_runtime_event_projection_task(context.clone()),
        whatsapp_provider_observation_reconciliation_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("whatsapp"))
    .collect()
}

pub(crate) fn zoom_runtime_task_specs(
    context: ApplicationBootstrapContext,
) -> Vec<RuntimeTaskSpec> {
    [
        zoom_token_maintenance_task(context.clone()),
        zoom_recording_sync_task(context.clone()),
        zoom_retention_cleanup_task(context.clone()),
        zoom_calendar_matching_projection_task(context.clone()),
        zoom_signal_detection_projection_task(context.clone()),
        zoom_participant_identity_projection_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("zoom"))
    .collect()
}

pub(crate) fn yandex_telemost_runtime_task_specs(
    context: ApplicationBootstrapContext,
) -> Vec<RuntimeTaskSpec> {
    [
        yandex_telemost_retention_cleanup_task(context.clone()),
        yandex_telemost_calendar_matching_projection_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("yandex_telemost"))
    .collect()
}

pub(crate) fn core_runtime_task_specs(
    context: ApplicationBootstrapContext,
) -> Vec<RuntimeTaskSpec> {
    [
        communication_provider_observation_projection_task(context.clone()),
        signal_hub_raw_signal_dispatcher_task(context.clone()),
        event_outbox_dispatcher_task(context.clone()),
        signal_replay_dispatcher_task(context.clone()),
        realtime_conversation_transcript_execution_task(context.clone()),
        realtime_conversation_transcript_projection_task(context.clone()),
        persona_derived_evidence_projection_task(context.clone()),
        persona_identity_review_inbox_projection_task(context.clone()),
        project_link_review_effects_projection_task(context),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn zoom_token_maintenance_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    if !context.zoom_token_maintenance_scheduler_enabled {
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_token_maintenance_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick =
                tokio::time::interval(Duration::from_secs(ZOOM_TOKEN_MAINTENANCE_TICK_SECONDS));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                match zoom::run_zoom_token_maintenance_once(&pool, &vault, &event_bus).await {
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
                            refresh_expiring_within_seconds =
                                result.refresh_expiring_within_seconds,
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_TOKEN_MAINTENANCE_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_recording_sync_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    if !context.zoom_recording_sync_scheduler_enabled {
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_recording_sync_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick =
                tokio::time::interval(Duration::from_secs(ZOOM_RECORDING_SYNC_TICK_SECONDS));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                match zoom::run_zoom_recording_sync_once(&pool, &vault, &event_bus).await {
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_RECORDING_SYNC_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_retention_cleanup_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    if !context.zoom_retention_cleanup_scheduler_enabled {
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_retention_cleanup_scheduler(&database_url) {
        return None;
    }
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick =
                tokio::time::interval(Duration::from_secs(ZOOM_RETENTION_CLEANUP_TICK_SECONDS));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                match zoom::run_zoom_retention_cleanup_once(&pool, &event_bus).await {
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_RETENTION_CLEANUP_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn yandex_telemost_retention_cleanup_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_yandex_telemost_retention_cleanup_scheduler(&database_url) {
        return None;
    }
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(
                YANDEX_TELEMOST_RETENTION_CLEANUP_TICK_SECONDS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "yandex_telemost",
                    YANDEX_TELEMOST_RETENTION_CLEANUP_RUNTIME,
                    json!({
                        "label": "Yandex Telemost retention cleanup",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                match run_yandex_telemost_retention_cleanup_once(&pool, &event_bus).await {
                    Ok(result)
                        if result.accounts_checked > 0
                            || result.accounts_cleaned > 0
                            || result.audio_files_removed > 0
                            || result.speaker_hint_files_removed > 0 =>
                    {
                        tracing::info!(
                            accounts_checked = result.accounts_checked,
                            accounts_cleaned = result.accounts_cleaned,
                            audio_files_removed = result.audio_files_removed,
                            speaker_hint_files_removed = result.speaker_hint_files_removed,
                            limit_per_account = result.limit_per_account,
                            "yandex telemost retention cleanup scheduler tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            "yandex telemost retention cleanup scheduler tick failed"
                        );
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        YANDEX_TELEMOST_RETENTION_CLEANUP_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn whatsapp_runtime_event_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_whatsapp_runtime_event_consumer(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::application::whatsapp_runtime_event_projection::WHATSAPP_RUNTIME_EVENT_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                            crate::application::whatsapp_runtime_event_projection::project_whatsapp_runtime_event(
                                handler_pool.clone(),
                                handler_vault.clone(),
                                event,
                            )
                            .await
                            .map_err(hermes_events_postgres::errors::EventStoreError::ConsumerHandlerFailed)
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn whatsapp_provider_observation_reconciliation_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_whatsapp_provider_observation_reconciliation_consumer(&database_url) {
        return None;
    }
    let event_bus = context.event_bus;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::application::whatsapp_provider_observation_reconciliation::WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                            crate::application::whatsapp_provider_observation_reconciliation::reconcile_whatsapp_provider_observation_event(
                                handler_pool.clone(),
                                handler_event_bus.clone(),
                                event,
                            )
                            .await
                            .map_err(hermes_events_postgres::errors::EventStoreError::ConsumerHandlerFailed)
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn communication_provider_observation_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_communication_provider_observation_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::domains::communications::messages::COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_millis(
                REALTIME_SIGNAL_PROJECTION_TICK_MILLIS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME,
        RuntimeTaskClass::Essential,
        RuntimeExitPolicy::ShutdownRuntime,
        task,
    ))
}

fn persona_derived_evidence_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_persona_derived_evidence_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::persona_derived_evidence::PERSONA_DERIVED_EVIDENCE_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "system",
                    PERSONA_DERIVED_EVIDENCE_RUNTIME,
                    json!({
                        "label": "Persona derived evidence consumer",
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
                    crate::workflows::persona_derived_evidence::project_persona_derived_evidence_event(
                        handler_pool.clone(),
                        event,
                    )
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    "persona derived evidence projection consumer tick failed"
                );
            }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        PERSONA_DERIVED_EVIDENCE_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_calendar_matching_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_calendar_matching_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::zoom_calendar_matching::ZOOM_CALENDAR_MATCHING_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                        crate::workflows::zoom_calendar_matching::project_zoom_calendar_matching_event(
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_CALENDAR_MATCHING_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_signal_detection_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_signal_detection_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::zoom_signal_detection::ZOOM_SIGNAL_DETECTION_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                        crate::workflows::zoom_signal_detection::project_zoom_signal_detection_event(
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_SIGNAL_DETECTION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn persona_identity_review_inbox_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_persona_identity_review_inbox_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::review_inbox::PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "system",
                    PERSONA_IDENTITY_REVIEW_INBOX_RUNTIME,
                    json!({
                        "label": "Persona identity review inbox consumer",
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
                        crate::workflows::review_inbox::project_persona_identity_review_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "persona identity review inbox projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        PERSONA_IDENTITY_REVIEW_INBOX_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn zoom_participant_identity_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_zoom_participant_identity_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::zoom_participant_identity::ZOOM_PARTICIPANT_IDENTITY_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                        crate::workflows::zoom_participant_identity::project_zoom_participant_identity_event(
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ZOOM_PARTICIPANT_IDENTITY_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn yandex_telemost_calendar_matching_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_yandex_telemost_calendar_matching_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::yandex_telemost_calendar_matching::YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "yandex_telemost",
                    YANDEX_TELEMOST_CALENDAR_MATCHING_RUNTIME,
                    json!({
                        "label": "Yandex Telemost calendar matching consumer",
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
                        crate::workflows::yandex_telemost_calendar_matching::project_yandex_telemost_calendar_matching_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "yandex telemost calendar matching projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        YANDEX_TELEMOST_CALENDAR_MATCHING_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn project_link_review_effects_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_project_link_review_effects_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
            pool.clone(),
            hermes_events_postgres::consumers::EventConsumerConfig::new(
                crate::workflows::project_link_review_effects::PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
            ),
        );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                    crate::workflows::project_link_review_effects::project_link_review_effect_event(
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        PROJECT_LINK_REVIEW_EFFECTS_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn realtime_conversation_transcript_execution_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    if !crate::workflows::realtime_conversation_transcript_execution::realtime_conversation_transcriber_is_configured() {
        tracing::info!(
            "HERMES_REALTIME_CONVERSATION_TRANSCRIBER is not configured; realtime conversation transcript execution consumer is disabled"
        );
        return None;
    }
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_realtime_conversation_transcript_execution_consumer(&database_url) {
        return None;
    }
    let event_bus = context.event_bus.clone();

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let event_bus = event_bus.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::realtime_conversation_transcript_execution::REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "system",
                    REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_RUNTIME,
                    json!({
                        "label": "Realtime conversation transcript execution consumer",
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
                        crate::workflows::realtime_conversation_transcript_execution::execute_realtime_conversation_transcript_request_event(
                            handler_pool.clone(),
                            handler_event_bus.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "realtime conversation transcript execution consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn realtime_conversation_transcript_projection_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_realtime_conversation_transcript_projection_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::workflows::realtime_conversation_transcript_projection::REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "system",
                    REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_RUNTIME,
                    json!({
                        "label": "Realtime conversation transcript projection consumer",
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
                        crate::workflows::realtime_conversation_transcript_projection::project_realtime_conversation_transcript_event(
                            handler_pool.clone(),
                            event,
                        )
                    })
                    .await
                {
                    tracing::warn!(
                        error = %error,
                        "realtime conversation transcript projection consumer tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn signal_hub_raw_signal_dispatcher_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_signal_hub_raw_signal_consumer(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let runner = hermes_events_postgres::consumers::EventConsumerRunner::new(
                pool.clone(),
                hermes_events_postgres::consumers::EventConsumerConfig::new(
                    crate::domains::signal_hub::service::SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
                ),
            );
            let mut tick = tokio::time::interval(Duration::from_millis(
                REALTIME_SIGNAL_PROJECTION_TICK_MILLIS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
                        crate::domains::signal_hub::service::process_signal_hub_raw_event(
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        SIGNAL_HUB_RAW_SIGNAL_RUNTIME,
        RuntimeTaskClass::Essential,
        RuntimeExitPolicy::ShutdownRuntime,
        task,
    ))
}

fn event_outbox_dispatcher_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    let Some(nats_server_url) = context.nats_server_url else {
        tracing::info!(
            "event outbox dispatcher skipped because HERMES_NATS_SERVER_URL is not configured"
        );
        return None;
    };
    if !register_event_outbox_dispatcher(&database_url) {
        return None;
    }

    let realtime_bus = context.event_bus.clone();
    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let realtime_bus = realtime_bus.clone();
        let nats_server_url = nats_server_url.clone();
        Box::pin(async move {
            let bus = tokio::select! {
                _ = cancellation.cancelled() => return Ok(()),
                result = hermes_events_nats::jetstream::NatsJetStreamEventBus::connect(&nats_server_url) => {
                    result.map_err(|error| {
                        tracing::warn!(
                            error = %error,
                            nats_server_url,
                            "event outbox dispatcher failed to initialize JetStream bus"
                        );
                        RuntimeTaskError::Coded {
                            code: "event_outbox_nats_connect_failed".to_owned(),
                        }
                    })?
                }
            };
            let dispatcher = crate::platform::events::dispatcher::EventOutboxDispatcher::new(
                hermes_events_postgres::store::EventStore::new(pool.clone()),
                bus,
            )
            .with_realtime_bus(realtime_bus);
            let mut tick = tokio::time::interval(Duration::from_secs(2));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        EVENT_OUTBOX_DISPATCHER_RUNTIME,
        RuntimeTaskClass::Essential,
        RuntimeExitPolicy::ShutdownRuntime,
        task,
    ))
}

fn signal_replay_dispatcher_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_signal_replay_dispatcher(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let replay_service = crate::application::signal_hub_replay::SignalHubReplayService::new(
                crate::domains::signal_hub::store::SignalHubStore::new(pool.clone()),
                hermes_events_postgres::store::EventStore::new(pool.clone()),
            );
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
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
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        SIGNAL_REPLAY_DISPATCHER_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
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

fn register_telegram_runtime_reconciliation(database_url: &str) -> bool {
    match TELEGRAM_RUNTIME_RECONCILIATION_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "telegram runtime reconciliation registry is unavailable"
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

fn register_persona_derived_evidence_consumer(database_url: &str) -> bool {
    match PERSONA_DERIVED_EVIDENCE_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "persona derived evidence consumer registry is unavailable"
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

fn register_yandex_telemost_retention_cleanup_scheduler(database_url: &str) -> bool {
    match YANDEX_TELEMOST_RETENTION_CLEANUP_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "yandex telemost retention cleanup scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_yandex_telemost_calendar_matching_consumer(database_url: &str) -> bool {
    match YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "yandex telemost calendar matching consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_persona_identity_review_inbox_consumer(database_url: &str) -> bool {
    match PERSONA_IDENTITY_REVIEW_INBOX_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "persona identity review inbox consumer registry is unavailable"
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

fn register_realtime_conversation_transcript_execution_consumer(database_url: &str) -> bool {
    match REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(_) => {
            tracing::warn!(
                "realtime conversation transcript execution consumer registry is unavailable"
            );
            false
        }
    }
}

fn register_realtime_conversation_transcript_projection_consumer(database_url: &str) -> bool {
    match REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "realtime conversation transcript projection consumer registry is unavailable"
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

fn register_mail_attachment_scan_worker(database_url: &str) -> bool {
    match MAIL_ATTACHMENT_SCAN_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(error = %error, "mail attachment rescan scheduler registry is unavailable");
            false
        }
    }
}

fn register_address_book_sync_scheduler(database_url: &str) -> bool {
    match ADDRESS_BOOK_SYNC_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "address book sync scheduler registry is unavailable"
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

fn register_mail_provider_command_scheduler(database_url: &str) -> bool {
    match MAIL_PROVIDER_COMMAND_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(error = %error, "mail provider command scheduler registry is unavailable");
            false
        }
    }
}

fn register_mail_ai_pipeline(database_url: &str) -> bool {
    match MAIL_AI_PIPELINE_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail AI pipeline registry is unavailable"
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

struct YandexTelemostRetentionCleanupSchedulerResult {
    accounts_checked: usize,
    accounts_cleaned: usize,
    audio_files_removed: usize,
    speaker_hint_files_removed: usize,
    limit_per_account: i64,
}

async fn run_yandex_telemost_retention_cleanup_once(
    pool: &PgPool,
    event_bus: &InMemoryEventBus,
) -> Result<YandexTelemostRetentionCleanupSchedulerResult, String> {
    let service =
        crate::application::provider_runtime_services::yandex_telemost_provider_runtime_service(
            pool.clone(),
            event_bus.clone(),
        );
    let accounts = service
        .list_accounts(false)
        .await
        .map_err(|error| error.to_string())?
        .items;
    let mut result = YandexTelemostRetentionCleanupSchedulerResult {
        accounts_checked: 0,
        accounts_cleaned: 0,
        audio_files_removed: 0,
        speaker_hint_files_removed: 0,
        limit_per_account: YANDEX_TELEMOST_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
    };

    for account in accounts {
        if account.provider_kind != "yandex_telemost_user" {
            continue;
        }
        result.accounts_checked += 1;
        let response = service
            .cleanup_retention(
                &account.account_id,
                &crate::integrations::yandex_telemost::client::models::YandexTelemostRetentionCleanupRequest {
                    remove_audio: true,
                    remove_speaker_hints: true,
                    limit: YANDEX_TELEMOST_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        if response.bundles_cleaned > 0 {
            result.accounts_cleaned += 1;
        }
        result.audio_files_removed += response.audio_files_removed;
        result.speaker_hint_files_removed += response.speaker_hint_files_removed;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telegram_runtime_reconciliation_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://telegram-runtime-reconciliation-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(register_telegram_runtime_reconciliation(&database_url));
        assert!(!register_telegram_runtime_reconciliation(&database_url));
    }

    #[test]
    fn telegram_runtime_reconciliation_only_runs_enabled_runnable_accounts() {
        let now = Utc::now();
        let account = |config| hermes_communications_api::accounts::ProviderAccount {
            account_id: "telegram-account".to_owned(),
            provider_kind:
                hermes_communications_api::accounts::CommunicationProviderKind::TelegramUser,
            display_name: "Telegram".to_owned(),
            external_account_id: "telegram:1".to_owned(),
            config,
            created_at: now,
            updated_at: now,
        };

        assert!(telegram::runtime_reconciliation_enabled(&account(json!({
            "runtime": "fixture"
        }))));
        assert!(telegram::runtime_reconciliation_enabled(&account(json!({
            "runtime": "tdlib_qr_authorized"
        }))));
        assert!(!telegram::runtime_reconciliation_enabled(&account(json!({
            "runtime": "tdlib_qr_authorized",
            "runtime_enabled": false
        }))));
        assert!(!telegram::runtime_reconciliation_enabled(&account(json!({
            "runtime": "fixture",
            "lifecycle_state": "logged_out"
        }))));
        assert!(!telegram::runtime_reconciliation_enabled(&account(json!({
            "runtime": "live_blocked"
        }))));
    }

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
