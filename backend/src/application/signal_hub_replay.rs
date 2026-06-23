use chrono::{DateTime, Utc};
use serde_json::json;

use crate::domains::communications::messages::{
    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, replay_accepted_signal_event,
    supports_communication_projection_signal_event,
};
use crate::domains::persons::core::{
    PERSON_ROLE_ASSIGNED_EVENT_TYPE, PERSON_ROLE_REMOVED_EVENT_TYPE,
};
use crate::domains::persons::enrichment::PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::persons::trust::PERSON_PROMISE_CREATED_EVENT_TYPE;
use crate::domains::signal_hub::{
    SignalHubError, SignalHubSignalService, SignalHubStore, SignalReplayRequest,
    SignalReplayRequestCreate,
};
use crate::engines::timeline::TimelineEngine;
use crate::platform::events::{
    EventConsumerStore, EventLogQuery, EventStore, NewEventEnvelope, ProjectionCursorStore,
    StoredEventEnvelope,
};
use crate::workflows::project_link_review_effects::PROJECT_LINK_REVIEW_EVENT_TYPE;

use super::{
    PERSON_DERIVED_EVIDENCE_CONSUMER, PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
    project_link_review_effect_event, project_person_derived_evidence_event,
};

const DEFAULT_REPLAY_BATCH_SIZE: u32 = 500;
const COMMUNICATION_MESSAGES_PROJECTION: &str = "communication_messages";
const PERSON_DERIVED_EVIDENCE_PROJECTION: &str = "person_derived_evidence";
const PROJECT_LINK_REVIEW_EFFECTS_PROJECTION: &str = "project_link_review_effects";
const TIMELINE_EVENT_LOG_PROJECTION: &str = "timeline_event_log";
const TIMELINE_EVENT_LOG_CURSOR: &str = "signal_hub.timeline_event_log";

#[derive(Clone)]
pub struct SignalHubReplayService {
    signal_store: SignalHubStore,
    signal_service: SignalHubSignalService,
    event_store: EventStore,
}

impl SignalHubReplayService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        let signal_service = SignalHubSignalService::new(signal_store.clone(), event_store.clone());
        Self {
            signal_store,
            signal_service,
            event_store,
        }
    }

    pub async fn request_replay(
        &self,
        request: &SignalReplayRequestCreate,
    ) -> Result<crate::domains::signal_hub::SignalReplayRequest, SignalHubError> {
        let replay_request = self.signal_store.create_replay_request(request).await?;
        self.append_replay_lifecycle_event(
            "signal.replay.requested",
            &replay_request.id,
            json!({
                "status": replay_request.status,
                "source_code": replay_request.source_code,
                "connection_id": replay_request.connection_id,
                "event_pattern": replay_request.event_pattern,
                "target_consumer": replay_request.target_consumer,
                "target_projection": replay_request.target_projection,
                "requested_by": replay_request.requested_by,
                "requested_at": replay_request.requested_at,
                "metadata": replay_request.metadata,
            }),
        )
        .await?;
        Ok(replay_request)
    }

    pub async fn process_next_request(
        &self,
    ) -> Result<Option<SignalReplayRunReport>, SignalHubError> {
        let Some(request) = self.signal_store.claim_next_replay_request().await? else {
            return Ok(None);
        };

        match self.process_claimed_request(&request).await {
            Ok(report) => Ok(Some(report)),
            Err(error) => {
                self.signal_store
                    .mark_replay_request_failed(&request.id, &error.to_string())
                    .await?;
                self.append_replay_lifecycle_event(
                    "signal.replay.failed",
                    &request.id,
                    json!({
                        "status": "failed",
                        "error": error.to_string(),
                    }),
                )
                .await?;
                Err(error)
            }
        }
    }

    async fn process_claimed_request(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<SignalReplayRunReport, SignalHubError> {
        let mut replayed_count: u32 = 0;
        if let Some(target_projection) = request.target_projection.as_deref() {
            replayed_count = self.rebuild_projection(target_projection, request).await?;
        } else if let Some(target_consumer) = request.target_consumer.as_deref() {
            let replay_events = self.list_consumer_replay_events(request).await?;
            self.prepare_consumer_replay(target_consumer, &replay_events)
                .await?;
            replayed_count = u32::try_from(replay_events.len()).unwrap_or(u32::MAX);
        } else if uses_event_log_replay(request) {
            let replay_events = self.list_event_log_events_for_replay(request).await?;
            for replay_event in replay_events {
                self.signal_service
                    .replay_raw_signal(&replay_event.event)
                    .await?;
                replayed_count = replayed_count.saturating_add(1);
            }
        } else {
            let paused_events = self
                .signal_store
                .list_paused_events_for_replay(request, DEFAULT_REPLAY_BATCH_SIZE)
                .await?;

            for paused_event in paused_events {
                self.signal_service
                    .replay_raw_signal(&paused_event.event)
                    .await?;
                self.signal_store
                    .release_paused_event(&paused_event.event_id)
                    .await?;
                replayed_count = replayed_count.saturating_add(1);
            }
        }

        self.signal_store
            .mark_replay_request_completed(
                &request.id,
                i32::try_from(replayed_count).unwrap_or(i32::MAX),
            )
            .await?;
        self.append_replay_lifecycle_event(
            "signal.replay.completed",
            &request.id,
            json!({
                "status": "completed",
                "replayed_count": replayed_count,
                "source_code": request.source_code,
                "connection_id": request.connection_id,
                "event_pattern": request.event_pattern,
                "target_consumer": request.target_consumer,
                "target_projection": request.target_projection,
                "from_position": request.from_position,
                "to_position": request.to_position,
                "from_time": request.from_time.map(|value| value.to_rfc3339()),
                "to_time": request.to_time.map(|value| value.to_rfc3339()),
            }),
        )
        .await?;

        Ok(SignalReplayRunReport {
            request_id: request.id.clone(),
            replayed_count,
        })
    }

    async fn rebuild_projection(
        &self,
        target_projection: &str,
        request: &SignalReplayRequest,
    ) -> Result<u32, SignalHubError> {
        match target_projection {
            COMMUNICATION_MESSAGES_PROJECTION => {
                self.rebuild_communication_messages_projection(request)
                    .await
            }
            PERSON_DERIVED_EVIDENCE_PROJECTION => {
                self.rebuild_person_derived_evidence_projection(request)
                    .await
            }
            PROJECT_LINK_REVIEW_EFFECTS_PROJECTION => {
                self.rebuild_project_link_review_effects_projection(request)
                    .await
            }
            TIMELINE_EVENT_LOG_PROJECTION => self.rebuild_timeline_projection(request).await,
            other => Err(SignalHubError::InvalidReplayRequest(format!(
                "unsupported target_projection: {other}"
            ))),
        }
    }

    async fn append_replay_lifecycle_event(
        &self,
        event_type: &str,
        replay_request_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!("evt_{}_{}", event_type.replace('.', "_"), replay_request_id),
            event_type,
            chrono::Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": "system",
                "source_id": replay_request_id,
            }),
            json!({
                "kind": "signal_replay_request",
                "entity_id": replay_request_id,
            }),
        )
        .payload(payload)
        .correlation_id(replay_request_id)
        .build()?;

        self.event_store
            .append_for_dispatch_idempotent(&event)
            .await?;
        Ok(())
    }

    async fn list_event_log_events_for_replay(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<Vec<StoredEventEnvelope>, SignalHubError> {
        let events = self.list_matching_signal_events(request).await?;
        Ok(events
            .into_iter()
            .filter(|event| event.event.event_type.starts_with("signal.raw."))
            .collect())
    }

    async fn list_consumer_replay_events(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<Vec<StoredEventEnvelope>, SignalHubError> {
        self.list_matching_signal_events(request).await
    }

    async fn list_matching_signal_events(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<Vec<StoredEventEnvelope>, SignalHubError> {
        let mut query = EventLogQuery::default().limit(DEFAULT_REPLAY_BATCH_SIZE);
        if let Some(source_code) = request.source_code.as_deref() {
            query = query.source_code(source_code);
        }
        if let (Some(from_position), Some(to_position)) =
            (request.from_position, request.to_position)
        {
            query = query.position_between(from_position, to_position);
        } else {
            if let Some(from_position) = request.from_position {
                query = query.position_after(from_position);
            }
            if let Some(to_position) = request.to_position {
                query = query.position_before(to_position);
            }
        }
        if let (Some(from_time), Some(to_time)) = (request.from_time, request.to_time) {
            query = query.occurred_between(from_time, to_time);
        } else {
            query.occurred_after = request.from_time;
            query.occurred_before = request.to_time;
        }

        let events = self.event_store.list_matching(query).await?;
        let mut filtered_events = Vec::new();
        for event in events.into_iter().filter(|event| {
            event.event.event_type.starts_with("signal.")
                && request.event_pattern.as_deref().is_none_or(|pattern| {
                    crate::domains::signal_hub::event_type_pattern_matches(
                        pattern,
                        &event.event.event_type,
                    )
                })
        }) {
            if let Some(connection_id) = request.connection_id.as_deref() {
                let Some(source_code) = request.source_code.as_deref() else {
                    continue;
                };
                let event_connection_id = self
                    .signal_store
                    .resolve_connection_id_for_event(source_code, &event.event)
                    .await?;
                if event_connection_id.as_deref() != Some(connection_id) {
                    continue;
                }
            }
            filtered_events.push(event);
        }
        Ok(filtered_events)
    }

    async fn list_matching_projection_events(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<Vec<StoredEventEnvelope>, SignalHubError> {
        let mut query = EventLogQuery::default().limit(DEFAULT_REPLAY_BATCH_SIZE);
        if let Some(source_code) = request.source_code.as_deref() {
            query = query.source_code(source_code);
        }
        if let (Some(from_position), Some(to_position)) =
            (request.from_position, request.to_position)
        {
            query = query.position_between(from_position, to_position);
        } else {
            if let Some(from_position) = request.from_position {
                query = query.position_after(from_position);
            }
            if let Some(to_position) = request.to_position {
                query = query.position_before(to_position);
            }
        }
        if let (Some(from_time), Some(to_time)) = (request.from_time, request.to_time) {
            query = query.occurred_between(from_time, to_time);
        } else {
            query.occurred_after = request.from_time;
            query.occurred_before = request.to_time;
        }

        let events = self.event_store.list_matching(query).await?;
        Ok(events
            .into_iter()
            .filter(|event| {
                request.event_pattern.as_deref().is_none_or(|pattern| {
                    crate::domains::signal_hub::event_type_pattern_matches(
                        pattern,
                        &event.event.event_type,
                    )
                })
            })
            .collect())
    }

    async fn rebuild_timeline_projection(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<u32, SignalHubError> {
        let replay_events = self.list_matching_projection_events(request).await?;
        let Some(first_position) = replay_events.first().map(|event| event.position) else {
            return Ok(0);
        };
        let last_position = replay_events
            .last()
            .map(|event| event.position)
            .unwrap_or(first_position);
        let period_start = request
            .from_time
            .or_else(|| replay_events.first().map(|event| event.event.occurred_at))
            .unwrap_or_else(Utc::now);
        let period_end = request
            .to_time
            .or_else(|| replay_events.last().map(|event| event.event.occurred_at))
            .unwrap_or(period_start);
        let timeline_replay = TimelineEngine::replay_event_log(
            &replay_events,
            period_start,
            max_timestamp(period_end, period_start),
            i64::from(DEFAULT_REPLAY_BATCH_SIZE),
        )
        .map_err(|error| {
            SignalHubError::InvalidReplayRequest(format!(
                "timeline projection replay failed: {error}"
            ))
        })?;

        let cursor_store = ProjectionCursorStore::new(self.event_store.pool().clone());
        cursor_store
            .rewind_position(
                TIMELINE_EVENT_LOG_CURSOR,
                first_position.saturating_sub(1).max(0),
            )
            .await?;
        cursor_store
            .save_position(TIMELINE_EVENT_LOG_CURSOR, last_position)
            .await?;

        self.append_projection_lifecycle_event(
            "timeline.projection.updated",
            request,
            json!({
                "target_projection": TIMELINE_EVENT_LOG_PROJECTION,
                "projection_name": TIMELINE_EVENT_LOG_CURSOR,
                "from_position": first_position,
                "to_position": last_position,
                "replayed_count": replay_events.len(),
                "entries_count": timeline_replay.entries.len(),
                "last_replayed_position": timeline_replay.last_replayed_position,
                "period_start": period_start.to_rfc3339(),
                "period_end": max_timestamp(period_end, period_start).to_rfc3339(),
            }),
        )
        .await?;

        Ok(u32::try_from(replay_events.len()).unwrap_or(u32::MAX))
    }

    async fn rebuild_communication_messages_projection(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<u32, SignalHubError> {
        let replay_events = self
            .list_matching_signal_events(request)
            .await?
            .into_iter()
            .filter(|event| supports_communication_projection_signal_event(&event.event.event_type))
            .collect::<Vec<_>>();
        let Some(first_position) = replay_events.first().map(|event| event.position) else {
            return Ok(0);
        };
        let last_position = replay_events
            .last()
            .map(|event| event.position)
            .unwrap_or(first_position);
        let positions: Vec<i64> = replay_events.iter().map(|event| event.position).collect();
        let consumer_store = EventConsumerStore::new(self.event_store.pool().clone());
        let cleared_processed = consumer_store
            .clear_processed_for_positions(COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, &positions)
            .await?;
        consumer_store
            .clear_failures_for_positions(COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, &positions)
            .await?;
        consumer_store
            .rewind_position(
                COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                first_position.saturating_sub(1),
            )
            .await?;

        for replay_event in &replay_events {
            replay_accepted_signal_event(self.event_store.pool().clone(), replay_event.clone())
                .await
                .map_err(|error| {
                    SignalHubError::InvalidReplayRequest(format!(
                        "communication_messages projection replay failed: {error}"
                    ))
                })?;
            consumer_store
                .record_processed(COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, replay_event)
                .await?;
            consumer_store
                .mark_dead_letter_replayed_for_event(
                    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                    replay_event.position,
                )
                .await?;
            consumer_store
                .clear_failure(
                    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                    replay_event.position,
                )
                .await?;
            consumer_store
                .save_position(
                    COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                    replay_event.position,
                )
                .await?;
        }

        self.append_projection_lifecycle_event(
            "communications.projection.updated",
            request,
            json!({
                "target_projection": COMMUNICATION_MESSAGES_PROJECTION,
                "consumer_name": COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
                "from_position": first_position,
                "to_position": last_position,
                "replayed_count": replay_events.len(),
                "cleared_processed_count": cleared_processed,
            }),
        )
        .await?;

        Ok(u32::try_from(replay_events.len()).unwrap_or(u32::MAX))
    }

    async fn rebuild_person_derived_evidence_projection(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<u32, SignalHubError> {
        let replay_events = self
            .list_matching_projection_events(request)
            .await?
            .into_iter()
            .filter(|event| {
                supports_person_derived_evidence_projection_event(&event.event.event_type)
            })
            .collect::<Vec<_>>();
        let Some(first_position) = replay_events.first().map(|event| event.position) else {
            return Ok(0);
        };
        let last_position = replay_events
            .last()
            .map(|event| event.position)
            .unwrap_or(first_position);
        let positions: Vec<i64> = replay_events.iter().map(|event| event.position).collect();
        let consumer_store = EventConsumerStore::new(self.event_store.pool().clone());
        let cleared_processed = consumer_store
            .clear_processed_for_positions(PERSON_DERIVED_EVIDENCE_CONSUMER, &positions)
            .await?;
        consumer_store
            .clear_failures_for_positions(PERSON_DERIVED_EVIDENCE_CONSUMER, &positions)
            .await?;
        consumer_store
            .rewind_position(
                PERSON_DERIVED_EVIDENCE_CONSUMER,
                first_position.saturating_sub(1),
            )
            .await?;

        for replay_event in &replay_events {
            project_person_derived_evidence_event(
                self.event_store.pool().clone(),
                replay_event.clone(),
            )
            .await
            .map_err(|error| {
                SignalHubError::InvalidReplayRequest(format!(
                    "person_derived_evidence projection replay failed: {error}"
                ))
            })?;
            consumer_store
                .record_processed(PERSON_DERIVED_EVIDENCE_CONSUMER, replay_event)
                .await?;
            consumer_store
                .mark_dead_letter_replayed_for_event(
                    PERSON_DERIVED_EVIDENCE_CONSUMER,
                    replay_event.position,
                )
                .await?;
            consumer_store
                .clear_failure(PERSON_DERIVED_EVIDENCE_CONSUMER, replay_event.position)
                .await?;
            consumer_store
                .save_position(PERSON_DERIVED_EVIDENCE_CONSUMER, replay_event.position)
                .await?;
        }

        self.append_projection_lifecycle_event(
            "persons.derived_evidence.updated",
            request,
            json!({
                "target_projection": PERSON_DERIVED_EVIDENCE_PROJECTION,
                "consumer_name": PERSON_DERIVED_EVIDENCE_CONSUMER,
                "from_position": first_position,
                "to_position": last_position,
                "replayed_count": replay_events.len(),
                "cleared_processed_count": cleared_processed,
            }),
        )
        .await?;

        Ok(u32::try_from(replay_events.len()).unwrap_or(u32::MAX))
    }

    async fn rebuild_project_link_review_effects_projection(
        &self,
        request: &SignalReplayRequest,
    ) -> Result<u32, SignalHubError> {
        let replay_events = self
            .list_matching_projection_events(request)
            .await?
            .into_iter()
            .filter(|event| {
                supports_project_link_review_effects_projection_event(&event.event.event_type)
            })
            .collect::<Vec<_>>();
        let Some(first_position) = replay_events.first().map(|event| event.position) else {
            return Ok(0);
        };
        let last_position = replay_events
            .last()
            .map(|event| event.position)
            .unwrap_or(first_position);
        let positions: Vec<i64> = replay_events.iter().map(|event| event.position).collect();
        let consumer_store = EventConsumerStore::new(self.event_store.pool().clone());
        let cleared_processed = consumer_store
            .clear_processed_for_positions(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, &positions)
            .await?;
        consumer_store
            .clear_failures_for_positions(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, &positions)
            .await?;
        consumer_store
            .rewind_position(
                PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
                first_position.saturating_sub(1),
            )
            .await?;

        for replay_event in &replay_events {
            project_link_review_effect_event(self.event_store.pool().clone(), replay_event.clone())
                .await
                .map_err(|error| {
                    SignalHubError::InvalidReplayRequest(format!(
                        "project_link_review_effects projection replay failed: {error}"
                    ))
                })?;
            consumer_store
                .record_processed(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, replay_event)
                .await?;
            consumer_store
                .mark_dead_letter_replayed_for_event(
                    PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
                    replay_event.position,
                )
                .await?;
            consumer_store
                .clear_failure(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, replay_event.position)
                .await?;
            consumer_store
                .save_position(PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, replay_event.position)
                .await?;
        }

        self.append_projection_lifecycle_event(
            "projects.link_review_effects.updated",
            request,
            json!({
                "target_projection": PROJECT_LINK_REVIEW_EFFECTS_PROJECTION,
                "consumer_name": PROJECT_LINK_REVIEW_EFFECTS_CONSUMER,
                "from_position": first_position,
                "to_position": last_position,
                "replayed_count": replay_events.len(),
                "cleared_processed_count": cleared_processed,
            }),
        )
        .await?;

        Ok(u32::try_from(replay_events.len()).unwrap_or(u32::MAX))
    }

    async fn prepare_consumer_replay(
        &self,
        consumer_name: &str,
        events: &[StoredEventEnvelope],
    ) -> Result<(), SignalHubError> {
        let positions: Vec<i64> = events.iter().map(|event| event.position).collect();
        EventConsumerStore::new(self.event_store.pool().clone())
            .request_replay_for_positions(consumer_name, &positions)
            .await?;
        Ok(())
    }

    async fn append_projection_lifecycle_event(
        &self,
        event_type: &str,
        request: &SignalReplayRequest,
        payload: serde_json::Value,
    ) -> Result<(), SignalHubError> {
        let event = NewEventEnvelope::builder(
            format!(
                "evt_{}_{}_{}",
                event_type.replace('.', "_"),
                request.target_projection.as_deref().unwrap_or("projection"),
                request.id
            ),
            event_type,
            Utc::now(),
            json!({
                "kind": "signal_source",
                "source_code": "system",
                "source_id": request.id,
            }),
            json!({
                "kind": "timeline_projection",
                "entity_id": request
                    .target_projection
                    .as_deref()
                    .unwrap_or(TIMELINE_EVENT_LOG_PROJECTION),
            }),
        )
        .payload(payload)
        .correlation_id(&request.id)
        .build()?;

        self.event_store
            .append_for_dispatch_idempotent(&event)
            .await?;
        Ok(())
    }
}

fn uses_event_log_replay(request: &SignalReplayRequest) -> bool {
    request.from_position.is_some()
        || request.to_position.is_some()
        || request.from_time.is_some()
        || request.to_time.is_some()
}

fn supports_person_derived_evidence_projection_event(event_type: &str) -> bool {
    matches!(
        event_type,
        PERSON_ROLE_ASSIGNED_EVENT_TYPE
            | PERSON_ROLE_REMOVED_EVENT_TYPE
            | PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE
            | PERSON_PROMISE_CREATED_EVENT_TYPE
    )
}

fn supports_project_link_review_effects_projection_event(event_type: &str) -> bool {
    event_type == PROJECT_LINK_REVIEW_EVENT_TYPE
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalReplayRunReport {
    pub request_id: String,
    pub replayed_count: u32,
}

fn max_timestamp(left: DateTime<Utc>, right: DateTime<Utc>) -> DateTime<Utc> {
    if left >= right { left } else { right }
}
