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

pub(crate) mod mail;
mod mail_ai;
pub(crate) mod telegram;
pub(crate) mod telemost;
pub(crate) mod whatsapp;
pub(crate) mod zoom;
pub(crate) mod zulip;

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
static COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSONA_DERIVED_EVIDENCE_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

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
const COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME: &str =
    "communication_provider_observation_projection";
const PERSONA_DERIVED_EVIDENCE_RUNTIME: &str = "persona_derived_evidence";

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
mod tests {
    use super::*;

    #[test]
    fn telegram_runtime_reconciliation_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://telegram-runtime-reconciliation-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(telegram::register_telegram_runtime_reconciliation(
            &database_url
        ));
        assert!(!telegram::register_telegram_runtime_reconciliation(
            &database_url
        ));
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

        assert!(mail::register_mail_outbox_delivery_scheduler(&database_url));
        assert!(!mail::register_mail_outbox_delivery_scheduler(
            &database_url
        ));
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

        assert!(zoom::tasks::register_zoom_token_maintenance_scheduler(
            &database_url
        ));
        assert!(!zoom::tasks::register_zoom_token_maintenance_scheduler(
            &database_url
        ));
    }

    #[test]
    fn zoom_recording_sync_scheduler_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-recording-sync-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(zoom::tasks::register_zoom_recording_sync_scheduler(
            &database_url
        ));
        assert!(!zoom::tasks::register_zoom_recording_sync_scheduler(
            &database_url
        ));
    }

    #[test]
    fn zoom_retention_cleanup_scheduler_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-retention-cleanup-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(zoom::tasks::register_zoom_retention_cleanup_scheduler(
            &database_url
        ));
        assert!(!zoom::tasks::register_zoom_retention_cleanup_scheduler(
            &database_url
        ));
    }

    #[test]
    fn zoom_calendar_matching_consumer_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-calendar-matching-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(zoom::tasks::register_zoom_calendar_matching_consumer(
            &database_url
        ));
        assert!(!zoom::tasks::register_zoom_calendar_matching_consumer(
            &database_url
        ));
    }

    #[test]
    fn zoom_signal_detection_consumer_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-signal-detection-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(zoom::tasks::register_zoom_signal_detection_consumer(
            &database_url
        ));
        assert!(!zoom::tasks::register_zoom_signal_detection_consumer(
            &database_url
        ));
    }

    #[test]
    fn zoom_participant_identity_consumer_registration_is_once_per_database_url() {
        let database_url = format!(
            "postgres://zoom-participant-identity-test/{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        assert!(zoom::tasks::register_zoom_participant_identity_consumer(
            &database_url
        ));
        assert!(!zoom::tasks::register_zoom_participant_identity_consumer(
            &database_url
        ));
    }
}
