use hermes_events_api::{EventEnvelope, NewEventEnvelope, StoredEventEnvelope};
use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::Duration;

use axum::Json;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Sse};
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{Instant, sleep};

use crate::app::api_support::{
    automation_calls::*,
    communications::*,
    ensure_fixture_routes_enabled,
    messaging_integrations::*,
    platform_dtos::*,
    query_parsing::{communication::*, documents::*, graph::*, personas::*, projects::*, tasks::*},
    review_commands::*,
    review_lists::*,
    stores::{ai_runtime::*, domain_stores::*, integration_stores::*, settings_vault::*},
    telegram_capabilities::*,
    whatsapp_capabilities::*,
};
use crate::app::{ApiError, AppState};
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::bus::sanitize_event_payload;
use hermes_events_postgres::trace::EventTrace;

pub(crate) async fn post_event(
    State(state): State<AppState>,
    Json(request): Json<AppendEventRequest>,
) -> Result<(StatusCode, Json<AppendEventResponse>), ApiError> {
    let actor_id = "hermes-frontend".to_string();

    let store = event_store(&state)?;
    let event = request.into_new_event()?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_append(
            actor_id,
            event.event_id.clone(),
        ))
        .await?;
    let position = store.append(&event).await?;

    Ok((
        StatusCode::CREATED,
        Json(AppendEventResponse {
            event_id: event.event_id,
            position,
        }),
    ))
}

pub(crate) async fn get_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventEnvelope>, ApiError> {
    let actor_id = "hermes-frontend".to_string();

    let store = event_store(&state)?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_get(actor_id, event_id.clone()))
        .await?;
    let Some(event) = store.get_by_id(&event_id).await? else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(event))
}

#[derive(Deserialize)]
pub(crate) struct EventTraceQuery {
    limit: Option<u32>,
}

pub(crate) async fn get_event_trace(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(query): Query<EventTraceQuery>,
) -> Result<Json<EventTrace>, ApiError> {
    let store = event_store(&state)?;
    let Some(trace) = store
        .trace_by_event_id(&event_id, query.limit.unwrap_or(1000))
        .await?
    else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(sanitize_trace_payloads(trace)))
}

pub(crate) async fn get_event_trace_by_correlation(
    State(state): State<AppState>,
    Path(correlation_id): Path<String>,
    Query(query): Query<EventTraceQuery>,
) -> Result<Json<EventTrace>, ApiError> {
    let store = event_store(&state)?;
    let trace = store
        .trace_by_correlation_id(&correlation_id, query.limit.unwrap_or(1000))
        .await?;

    Ok(Json(sanitize_trace_payloads(trace)))
}

pub(crate) async fn get_event_children(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Query(query): Query<EventTraceQuery>,
) -> Result<Json<Vec<StoredEventEnvelope>>, ApiError> {
    let store = event_store(&state)?;
    let children = store
        .list_children(&event_id, query.limit.unwrap_or(1000))
        .await?
        .into_iter()
        .map(sanitize_stored_event_payload)
        .collect();

    Ok(Json(children))
}

#[derive(Deserialize)]
pub(crate) struct EventListQuery {
    after_position: Option<i64>,
    limit: Option<u32>,
    wait_seconds: Option<u64>,
}

#[derive(Serialize)]
pub(crate) struct EventListResponse {
    items: Vec<StoredEventEnvelope>,
    next_after_position: i64,
    has_more: bool,
}

pub(crate) async fn get_events(
    State(state): State<AppState>,
    Query(query): Query<EventListQuery>,
) -> Result<Json<EventListResponse>, ApiError> {
    let after_position = query.after_position.unwrap_or(0);
    if after_position < 0 {
        return Err(ApiError::InvalidCommunicationQuery(
            "after_position must be non-negative",
        ));
    }
    let limit = query.limit.unwrap_or(100).clamp(1, 1000);
    let wait_seconds = query.wait_seconds.unwrap_or(0).clamp(0, 30);

    let store = event_store(&state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::event_list(
            "hermes-frontend",
            after_position,
            limit,
            wait_seconds,
        ))
        .await?;

    let deadline = Instant::now() + Duration::from_secs(wait_seconds);
    loop {
        let fetch_limit = limit.saturating_add(1).min(1000);
        let events = store
            .list_after_position(after_position, fetch_limit)
            .await?;
        if !events.is_empty() || Instant::now() >= deadline {
            return Ok(Json(event_list_response(after_position, limit, events)));
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        sleep(remaining.min(Duration::from_millis(500))).await;
    }
}

#[derive(Deserialize)]
pub(crate) struct EventStreamQuery {
    after_position: Option<i64>,
    batch_size: Option<u32>,
    heartbeat_seconds: Option<u64>,
}

pub(crate) async fn get_events_stream(
    State(state): State<AppState>,
    Query(query): Query<EventStreamQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let store = event_store(&state)?;
    let after_position = stream_start_position(&store, query.after_position).await?;
    let stream_state = EventStreamState {
        store,
        after_position,
        batch_size: query.batch_size.unwrap_or(100).clamp(1, 1000),
        heartbeat: Duration::from_secs(query.heartbeat_seconds.unwrap_or(15).clamp(1, 60)),
        pending: VecDeque::new(),
    };

    let stream = futures::stream::unfold(stream_state, |mut state| async move {
        loop {
            if let Some(envelope) = state.pending.pop_front() {
                state.after_position = envelope.position;
                return Some((Ok(stored_event_to_sse(envelope)), state));
            }

            match state
                .store
                .list_after_position(state.after_position, state.batch_size)
                .await
            {
                Ok(events) if !events.is_empty() => {
                    state.pending = events.into();
                }
                Ok(_) => {
                    sleep(state.heartbeat).await;
                    return Some((Ok(heartbeat_event(state.after_position)), state));
                }
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        after_position = state.after_position,
                        "event SSE replay polling failed"
                    );
                    sleep(state.heartbeat).await;
                    return Some((Ok(stream_error_event()), state));
                }
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub(crate) async fn get_events_websocket(
    State(state): State<AppState>,
    Query(query): Query<EventStreamQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    let store = event_store(&state)?;
    let after_position = stream_start_position(&store, query.after_position).await?;
    let stream_state = EventStreamState {
        store,
        after_position,
        batch_size: query.batch_size.unwrap_or(100).clamp(1, 1000),
        heartbeat: Duration::from_secs(query.heartbeat_seconds.unwrap_or(15).clamp(1, 60)),
        pending: VecDeque::new(),
    };

    Ok(ws.on_upgrade(move |socket| event_websocket_loop(socket, stream_state)))
}

pub(crate) async fn get_audit_events(
    State(state): State<AppState>,
    Query(query): Query<AuditEventsQuery>,
) -> Result<Json<AuditEventsResponse>, ApiError> {
    let audit_log = api_audit_log(&state)?;
    let items = audit_log
        .list_event_records(
            query.target_id.as_deref(),
            query.actor_id.as_deref(),
            query.after_audit_id.unwrap_or(0),
            query.limit.unwrap_or(100),
        )
        .await?;

    Ok(Json(AuditEventsResponse { items }))
}

async fn stream_start_position(
    store: &hermes_events_postgres::store::EventStore,
    after_position: Option<i64>,
) -> Result<i64, ApiError> {
    match after_position {
        Some(position) if position < 0 => Err(ApiError::InvalidCommunicationQuery(
            "after_position must be non-negative",
        )),
        Some(position) => Ok(position),
        None => Ok(store.latest_position().await?),
    }
}

struct EventStreamState {
    store: hermes_events_postgres::store::EventStore,
    after_position: i64,
    batch_size: u32,
    heartbeat: Duration,
    pending: VecDeque<StoredEventEnvelope>,
}

fn stored_event_to_sse(envelope: StoredEventEnvelope) -> Event {
    let position = envelope.position;
    match serde_json::to_string(&sanitize_stored_event_payload(envelope)) {
        Ok(data) => Event::default()
            .id(position.to_string())
            .event("event")
            .data(data),
        Err(error) => {
            tracing::warn!(error = %error, position, "event SSE serialization failed");
            stream_error_event()
        }
    }
}

async fn event_websocket_loop(mut socket: WebSocket, mut state: EventStreamState) {
    loop {
        if let Some(envelope) = state.pending.pop_front() {
            state.after_position = envelope.position;
            if send_ws_json(
                &mut socket,
                "event",
                serde_json::to_value(sanitize_stored_event_payload(envelope)),
            )
            .await
            {
                continue;
            }
            return;
        }

        match state
            .store
            .list_after_position(state.after_position, state.batch_size)
            .await
        {
            Ok(events) if !events.is_empty() => {
                state.pending = events.into();
            }
            Ok(_) => {
                sleep(state.heartbeat).await;
                if !send_ws_json(
                    &mut socket,
                    "heartbeat",
                    Ok(json!({ "after_position": state.after_position })),
                )
                .await
                {
                    return;
                }
            }
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    after_position = state.after_position,
                    "event WebSocket replay polling failed"
                );
                sleep(state.heartbeat).await;
                if !send_ws_json(
                    &mut socket,
                    "error",
                    Ok(json!({ "error": "event_stream_unavailable" })),
                )
                .await
                {
                    return;
                }
            }
        }
    }
}

async fn send_ws_json(
    socket: &mut WebSocket,
    message_type: &str,
    data: Result<serde_json::Value, serde_json::Error>,
) -> bool {
    let Ok(data) = data else {
        tracing::warn!(message_type, "event WebSocket serialization failed");
        return false;
    };
    let payload = json!({ "type": message_type, "data": data });
    socket
        .send(Message::Text(payload.to_string().into()))
        .await
        .is_ok()
}

fn event_list_response(
    after_position: i64,
    limit: u32,
    mut events: Vec<StoredEventEnvelope>,
) -> EventListResponse {
    let has_more = events.len() > limit as usize;
    events.truncate(limit as usize);
    let next_after_position = events
        .last()
        .map(|event| event.position)
        .unwrap_or(after_position);

    EventListResponse {
        items: events,
        next_after_position,
        has_more,
    }
}

fn heartbeat_event(after_position: i64) -> Event {
    Event::default()
        .event("heartbeat")
        .data(json!({ "after_position": after_position }).to_string())
}

fn stream_error_event() -> Event {
    Event::default()
        .event("error")
        .data(json!({ "error": "event_stream_unavailable" }).to_string())
}

// ---------------------------------------------------------------------------
// Realtime bus subscription endpoint (ADR-0091)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct RealtimeQuery {
    /// Optional event type prefix filter (e.g., "telegram" or "telegram.message")
    event_prefix: Option<String>,
}

/// WebSocket endpoint that subscribes to the in-memory InMemoryEventBus for realtime events.
/// Filterable by event type prefix.
pub(crate) async fn get_realtime_websocket(
    State(state): State<AppState>,
    Query(query): Query<RealtimeQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    let mut rx = state.event_bus.subscribe();
    let prefix = query.event_prefix.unwrap_or_default();

    Ok(ws.on_upgrade(move |mut socket| async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if prefix.is_empty() || event.event_type.starts_with(&prefix) {
                        let payload = json!({
                            "type": "event",
                            "data": {
                                "event_id": &event.event_id,
                                "event_type": &event.event_type,
                                "schema_version": event.schema_version,
                                "occurred_at": event.occurred_at.to_rfc3339(),
                                "recorded_at": null,
                                "source": &event.source,
                                "actor": &event.actor,
                                "subject": &event.subject,
                                "provenance": &event.provenance,
                                "causation_id": &event.causation_id,
                                "correlation_id": &event.correlation_id,
                                "payload": sanitize_event_payload(event.payload.clone()),
                            }
                        });
                        if socket
                            .send(Message::Text(payload.to_string().into()))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    let payload = json!({
                        "type": "lagged",
                        "data": { "skipped": n }
                    });
                    if socket
                        .send(Message::Text(payload.to_string().into()))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => return,
            }
        }
    }))
}

use tokio::sync::broadcast;

fn sanitize_trace_payloads(mut trace: EventTrace) -> EventTrace {
    trace.events = trace
        .events
        .into_iter()
        .map(sanitize_stored_event_payload)
        .collect();
    trace
}

fn sanitize_stored_event_payload(mut envelope: StoredEventEnvelope) -> StoredEventEnvelope {
    envelope.event.payload = sanitize_event_payload(envelope.event.payload);
    envelope
}
