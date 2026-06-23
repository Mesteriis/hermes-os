use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::platform::events::EventBus;
use crate::vault::{HostVault, VaultMode};

static MAIL_BACKGROUND_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_OUTBOX_DELIVERY_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static TELEGRAM_COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSON_DERIVED_EVIDENCE_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
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
const COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME: &str =
    "communication_provider_observation_projection";
const PERSON_DERIVED_EVIDENCE_RUNTIME: &str = "person_derived_evidence";
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
    pub(crate) vault: HostVault,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: EventBus,
}

pub(crate) fn start_background_services(context: ApplicationBootstrapContext) {
    start_mail_background_sync(context.clone());
    start_mail_outbox_delivery(context.clone());
    start_telegram_command_executor(context.clone());
    start_communication_provider_observation_projection(context.clone());
    start_person_derived_evidence_projection(context.clone());
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

fn host_vault_is_unlocked(vault: &HostVault) -> bool {
    match vault.status() {
        Ok(status) => status.state == VaultMode::Unlocked,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail outbox delivery scheduler could not read host vault status"
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
}
