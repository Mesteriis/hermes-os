use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use super::service::signal_hub_raw_dispatcher_allows_processing;
use super::{SignalHubError, SignalHubSignalService, SignalHubStore, SignalProcessingOutcome};
use crate::platform::events::{EventEnvelope, EventStore, NewEventEnvelope};

pub async fn dispatch_ai_helper_signal(
    pool: PgPool,
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<Option<EventEnvelope>, SignalHubError> {
    let event_store = EventStore::new(pool.clone());
    let signal_store = SignalHubStore::new(pool);
    signal_store.restore_system_sources().await?;
    let raw_signal = build_ai_helper_signal(
        event_kind,
        source_id,
        subject,
        payload,
        provenance,
        correlation_id,
    )?;
    event_store
        .append_for_dispatch_idempotent(&raw_signal)
        .await?;

    let raw_event = event_store
        .get_by_id(&raw_signal.event_id)
        .await?
        .ok_or_else(|| SignalHubError::InvalidRawSignalEventType(raw_signal.event_type.clone()))?;

    if !signal_hub_raw_dispatcher_allows_processing(&signal_store).await? {
        return Ok(None);
    }

    let service = SignalHubSignalService::new(signal_store, event_store.clone());
    match service.process_raw_signal(&raw_event).await? {
        SignalProcessingOutcome::Accepted { event_id } => {
            Ok(event_store.get_by_id(&event_id).await?)
        }
        SignalProcessingOutcome::Rejected { .. }
        | SignalProcessingOutcome::Muted { .. }
        | SignalProcessingOutcome::Paused { .. } => Ok(None),
    }
}

fn build_ai_helper_signal(
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<NewEventEnvelope, SignalHubError> {
    let builder = NewEventEnvelope::builder(
        format!("evt_signal_raw_ai_{}", Uuid::now_v7()),
        format!("signal.raw.ai.{event_kind}.observed"),
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "ai",
            "source_id": source_id,
        }),
        subject,
    )
    .payload(payload)
    .provenance(provenance);

    let builder = match correlation_id {
        Some(value) if !value.trim().is_empty() => builder.correlation_id(value.to_owned()),
        _ => builder,
    };

    Ok(builder.build()?)
}

pub async fn dispatch_ai_helper_signal_best_effort(
    pool: PgPool,
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Option<EventEnvelope> {
    match dispatch_ai_helper_signal(
        pool,
        event_kind,
        source_id,
        subject,
        payload,
        provenance,
        correlation_id,
    )
    .await
    {
        Ok(event) => event,
        Err(error) => {
            tracing::warn!(
                error = %error,
                event_kind,
                source_id,
                "AI helper Signal Hub dispatch failed"
            );
            None
        }
    }
}
