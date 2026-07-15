use super::validation::{
    parse_optional_uuid, truncate_redacted_error, validate_non_empty, validate_object,
};
use super::{PausedSignalEvent, SignalHubError, SignalHubStore, event_type_pattern_matches};
use crate::domains::signal_hub::replay_contracts::{
    SignalReplayRequest, SignalReplayRequestCreate,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

impl SignalHubStore {
    pub async fn list_replay_requests(&self) -> Result<Vec<SignalReplayRequest>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                event_pattern,
                status,
                requested_by,
                requested_at,
                started_at,
                completed_at,
                last_error_redacted,
                replayed_count,
                metadata
            FROM signal_replay_requests
            ORDER BY requested_at DESC, id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_replay_request).collect()
    }

    pub async fn create_replay_request(
        &self,
        request: &SignalReplayRequestCreate,
    ) -> Result<SignalReplayRequest, SignalHubError> {
        validate_object("metadata", &request.metadata)?;
        let requested_by = validate_non_empty("requested_by", &request.requested_by)?;
        let mut source_code = request
            .source_code
            .as_deref()
            .map(|value| validate_non_empty("source_code", value))
            .transpose()?;
        let target_consumer = request
            .target_consumer
            .as_deref()
            .map(|value| validate_non_empty("target_consumer", value))
            .transpose()?;
        let target_projection = request
            .target_projection
            .as_deref()
            .map(|value| validate_non_empty("target_projection", value))
            .map(|result| result.map(|value| canonical_target_projection(value.as_str())))
            .transpose()?;
        let connection_id = match request.connection_id.as_deref() {
            Some(value) => Some(
                Uuid::parse_str(value.trim())
                    .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))?,
            ),
            None => None,
        };
        let event_pattern = request
            .event_pattern
            .as_deref()
            .map(|value| validate_non_empty("event_pattern", value))
            .transpose()?;
        let from_position = request.from_position;
        let to_position = request.to_position;
        let from_time = request.from_time;
        let to_time = request.to_time;

        let has_selector = source_code.is_some()
            || connection_id.is_some()
            || event_pattern.is_some()
            || from_position.is_some()
            || to_position.is_some()
            || from_time.is_some()
            || to_time.is_some();

        if target_consumer.is_some() && target_projection.is_some() {
            return Err(SignalHubError::InvalidReplayRequest(
                "target_consumer and target_projection are mutually exclusive".to_owned(),
            ));
        }

        if !has_selector && target_consumer.is_none() && target_projection.is_none() {
            return Err(SignalHubError::InvalidReplayRequest(
                "at least one replay selector is required".to_owned(),
            ));
        }

        if target_consumer.is_some() && !has_selector {
            return Err(SignalHubError::InvalidReplayRequest(
                "target_consumer replay requires at least one source, pattern, connection or range selector"
                    .to_owned(),
            ));
        }

        if let Some(target_projection) = target_projection.as_deref() {
            match target_projection {
                "communication_messages"
                | "timeline_event_log"
                | "persona_derived_evidence"
                | "project_link_review_effects" => {}
                other => {
                    return Err(SignalHubError::InvalidReplayRequest(format!(
                        "unsupported target_projection: {other}"
                    )));
                }
            }
        }

        if let (Some(from_position), Some(to_position)) = (from_position, to_position) {
            if from_position < 0 || to_position < 0 || from_position > to_position {
                return Err(SignalHubError::InvalidReplayRequest(
                    "from_position and to_position must define a non-negative inclusive range"
                        .to_owned(),
                ));
            }
        } else if from_position.is_some_and(|value| value < 0)
            || to_position.is_some_and(|value| value < 0)
        {
            return Err(SignalHubError::InvalidReplayRequest(
                "replay positions must be non-negative".to_owned(),
            ));
        }

        if let (Some(from_time), Some(to_time)) = (from_time, to_time)
            && from_time > to_time
        {
            return Err(SignalHubError::InvalidReplayRequest(
                "from_time must be earlier than or equal to to_time".to_owned(),
            ));
        }

        if let Some(connection_id) = connection_id {
            let connection = self
                .connection_by_id(connection_id)
                .await?
                .ok_or_else(|| SignalHubError::ConnectionNotFound(connection_id.to_string()))?;
            if source_code
                .as_deref()
                .is_some_and(|value| value != connection.source_code)
            {
                return Err(SignalHubError::InvalidReplayRequest(
                    "connection_id source does not match source_code".to_owned(),
                ));
            }
            source_code = Some(connection.source_code);
        }

        if let Some(source_code_value) = source_code.as_deref() {
            let source = self
                .source_by_code(source_code_value)
                .await?
                .ok_or_else(|| SignalHubError::SourceNotFound(source_code_value.to_owned()))?;
            if !source.supports_replay {
                return Err(SignalHubError::InvalidReplayRequest(format!(
                    "signal source does not support replay: {}",
                    source_code_value
                )));
            }
        }

        let metadata = build_replay_metadata(
            request.metadata.clone(),
            from_position,
            to_position,
            from_time,
            to_time,
            target_consumer.as_deref(),
            target_projection.as_deref(),
        )?;

        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO signal_replay_requests (
                id,
                source_code,
                connection_id,
                event_pattern,
                status,
                requested_by,
                metadata
            )
            VALUES ($1, $2, $3, $4, 'queued', $5, $6)
            "#,
        )
        .bind(id)
        .bind(&source_code)
        .bind(connection_id)
        .bind(&event_pattern)
        .bind(requested_by)
        .bind(&metadata)
        .execute(&self.pool)
        .await?;

        self.replay_request_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::InvalidReplayRequest(id.to_string()))
    }

    pub async fn claim_next_replay_request(
        &self,
    ) -> Result<Option<SignalReplayRequest>, SignalHubError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            WITH candidate AS (
                SELECT id
                FROM signal_replay_requests
                WHERE status IN ('queued', 'requested')
                ORDER BY requested_at ASC, id ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE signal_replay_requests AS replay
            SET
                status = 'running',
                started_at = now(),
                completed_at = NULL,
                last_error_redacted = NULL
            FROM candidate
            WHERE replay.id = candidate.id
            RETURNING
                replay.id,
                replay.source_code,
                replay.connection_id,
                replay.event_pattern,
                replay.status,
                replay.requested_by,
                replay.requested_at,
                replay.started_at,
                replay.completed_at,
                replay.last_error_redacted,
                replay.replayed_count,
                replay.metadata
            "#,
        )
        .fetch_optional(&mut *transaction)
        .await?;
        transaction.commit().await?;

        row.map(row_to_replay_request).transpose()
    }

    pub async fn mark_replay_request_completed(
        &self,
        replay_request_id: &str,
        replayed_count: i32,
    ) -> Result<(), SignalHubError> {
        let replay_request_id = Uuid::parse_str(replay_request_id.trim())
            .map_err(|_| SignalHubError::InvalidReplayRequest(replay_request_id.to_owned()))?;

        sqlx::query(
            r#"
            UPDATE signal_replay_requests
            SET
                status = 'completed',
                replayed_count = $2,
                completed_at = now(),
                last_error_redacted = NULL
            WHERE id = $1
            "#,
        )
        .bind(replay_request_id)
        .bind(replayed_count.max(0))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_replay_request_failed(
        &self,
        replay_request_id: &str,
        error: &str,
    ) -> Result<(), SignalHubError> {
        let replay_request_id = Uuid::parse_str(replay_request_id.trim())
            .map_err(|_| SignalHubError::InvalidReplayRequest(replay_request_id.to_owned()))?;

        sqlx::query(
            r#"
            UPDATE signal_replay_requests
            SET
                status = 'failed',
                completed_at = now(),
                last_error_redacted = $2
            WHERE id = $1
            "#,
        )
        .bind(replay_request_id)
        .bind(truncate_redacted_error(error))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_paused_events_for_replay(
        &self,
        request: &SignalReplayRequest,
        limit: u32,
    ) -> Result<Vec<PausedSignalEvent>, SignalHubError> {
        let limit = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, source_code, raw_event_type, event_envelope, paused_at
            FROM signal_paused_events
            WHERE released_at IS NULL
              AND ($1::text IS NULL OR source_code = $1)
              AND ($2::uuid IS NULL OR connection_id = $2)
            ORDER BY paused_at ASC, id ASC
            LIMIT $3
            "#,
        )
        .bind(&request.source_code)
        .bind(parse_optional_uuid(request.connection_id.as_deref())?)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut paused_events = Vec::with_capacity(rows.len());
        for row in rows {
            let paused_event = row_to_paused_signal_event(row)?;
            if request.event_pattern.as_deref().is_some_and(|pattern| {
                !event_type_pattern_matches(pattern, &paused_event.raw_event_type)
            }) {
                continue;
            }
            paused_events.push(paused_event);
        }

        Ok(paused_events)
    }

    pub async fn release_paused_event(&self, event_id: &str) -> Result<(), SignalHubError> {
        sqlx::query(
            r#"
            UPDATE signal_paused_events
            SET released_at = now()
            WHERE event_id = $1
              AND released_at IS NULL
            "#,
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn paused_event_count(&self, source_code: &str) -> Result<i64, SignalHubError> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM signal_paused_events
            WHERE source_code = $1
              AND released_at IS NULL
            "#,
        )
        .bind(source_code)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn replay_request_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<SignalReplayRequest>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                event_pattern,
                status,
                requested_by,
                requested_at,
                started_at,
                completed_at,
                last_error_redacted,
                replayed_count,
                metadata
            FROM signal_replay_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_replay_request).transpose()
    }
}

fn row_to_replay_request(row: PgRow) -> Result<SignalReplayRequest, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    let metadata: Value = row.try_get("metadata")?;
    let selector = replay_selector_from_metadata(&metadata)?;
    Ok(SignalReplayRequest {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        event_pattern: row.try_get("event_pattern")?,
        from_position: selector.from_position,
        to_position: selector.to_position,
        from_time: selector.from_time,
        to_time: selector.to_time,
        target_consumer: selector.target_consumer,
        target_projection: selector.target_projection,
        status: row.try_get("status")?,
        requested_by: row.try_get("requested_by")?,
        requested_at: row.try_get("requested_at")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        last_error_redacted: row.try_get("last_error_redacted")?,
        replayed_count: row.try_get("replayed_count")?,
        metadata,
    })
}

#[derive(Clone, Debug, Default)]
struct ReplaySelector {
    from_position: Option<i64>,
    to_position: Option<i64>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    target_consumer: Option<String>,
    target_projection: Option<String>,
}

fn replay_selector_from_metadata(metadata: &Value) -> Result<ReplaySelector, SignalHubError> {
    let from_position = metadata.get("from_position").and_then(Value::as_i64);
    let to_position = metadata.get("to_position").and_then(Value::as_i64);
    let from_time = metadata
        .get("from_time")
        .and_then(Value::as_str)
        .map(parse_replay_timestamp)
        .transpose()?;
    let to_time = metadata
        .get("to_time")
        .and_then(Value::as_str)
        .map(parse_replay_timestamp)
        .transpose()?;
    let target_consumer = metadata
        .get("target_consumer")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let target_projection = metadata
        .get("target_projection")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(canonical_target_projection);

    Ok(ReplaySelector {
        from_position,
        to_position,
        from_time,
        to_time,
        target_consumer,
        target_projection,
    })
}

fn build_replay_metadata(
    mut metadata: Value,
    from_position: Option<i64>,
    to_position: Option<i64>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    target_consumer: Option<&str>,
    target_projection: Option<&str>,
) -> Result<Value, SignalHubError> {
    let metadata_object = metadata.as_object_mut().ok_or_else(|| {
        SignalHubError::InvalidReplayRequest("metadata must be a JSON object".to_owned())
    })?;

    if let Some(from_position) = from_position {
        metadata_object.insert("from_position".to_owned(), Value::from(from_position));
    }
    if let Some(to_position) = to_position {
        metadata_object.insert("to_position".to_owned(), Value::from(to_position));
    }
    if let Some(from_time) = from_time {
        metadata_object.insert(
            "from_time".to_owned(),
            Value::String(from_time.to_rfc3339()),
        );
    }
    if let Some(to_time) = to_time {
        metadata_object.insert("to_time".to_owned(), Value::String(to_time.to_rfc3339()));
    }
    if let Some(target_consumer) = target_consumer {
        metadata_object.insert(
            "target_consumer".to_owned(),
            Value::String(target_consumer.to_owned()),
        );
    }
    if let Some(target_projection) = target_projection {
        metadata_object.insert(
            "target_projection".to_owned(),
            Value::String(target_projection.to_owned()),
        );
    }

    Ok(metadata)
}

fn canonical_target_projection(value: &str) -> String {
    match value {
        "person_derived_evidence" => "persona_derived_evidence".to_owned(),
        other => other.to_owned(),
    }
}

fn parse_replay_timestamp(value: &str) -> Result<DateTime<Utc>, SignalHubError> {
    DateTime::parse_from_rfc3339(value.trim())
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            SignalHubError::InvalidReplayRequest(format!(
                "invalid replay timestamp `{value}`: {error}"
            ))
        })
}

fn row_to_paused_signal_event(row: PgRow) -> Result<PausedSignalEvent, SignalHubError> {
    let envelope: Value = row.try_get("event_envelope")?;
    Ok(PausedSignalEvent {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        event_id: row.try_get("event_id")?,
        source_code: row.try_get("source_code")?,
        raw_event_type: row.try_get("raw_event_type")?,
        event: serde_json::from_value(envelope)?,
        paused_at: row.try_get("paused_at")?,
    })
}
