use super::validation::{runtime_state_value, validate_non_empty, validate_object};
use super::{
    SignalHubError, SignalHubStore, SignalRuntimeState, SignalRuntimeStateUpdate,
    row_to_runtime_state,
};
use serde_json::Value;
use uuid::Uuid;

impl SignalHubStore {
    pub async fn ensure_runtime_state(
        &self,
        source_code: &str,
        runtime_kind: &str,
        default_state: &str,
        metadata: Value,
    ) -> Result<SignalRuntimeState, SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let runtime_kind = validate_non_empty("runtime_kind", runtime_kind)?;
        let default_state = runtime_state_value(default_state)?;
        validate_object("metadata", &metadata)?;
        if let Some(runtime) = self.runtime_state(&source_code, &runtime_kind).await? {
            return Ok(runtime);
        }

        sqlx::query(
            r#"
            INSERT INTO signal_runtime_states (
                id, source_code, runtime_kind, state, last_started_at, metadata
            )
            VALUES ($1, $2, $3, $4, CASE WHEN $4 = 'running' THEN now() ELSE NULL END, $5)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(&source_code)
        .bind(&runtime_kind)
        .bind(default_state)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        self.runtime_state(&source_code, &runtime_kind)
            .await?
            .ok_or_else(|| SignalHubError::InvalidRuntimeState(default_state.to_owned()))
    }

    pub async fn list_runtime_states(&self) -> Result<Vec<SignalRuntimeState>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT id, source_code, connection_id, runtime_kind, state,
                   last_started_at, last_stopped_at, last_heartbeat_at,
                   last_error_at, last_error_code, last_error_message_redacted,
                   metadata, updated_at
            FROM signal_runtime_states
            ORDER BY source_code ASC, runtime_kind ASC, updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_runtime_state).collect()
    }

    pub async fn runtime_state(
        &self,
        source_code: &str,
        runtime_kind: &str,
    ) -> Result<Option<SignalRuntimeState>, SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let runtime_kind = validate_non_empty("runtime_kind", runtime_kind)?;
        let row = sqlx::query(
            r#"
            SELECT id, source_code, connection_id, runtime_kind, state,
                   last_started_at, last_stopped_at, last_heartbeat_at,
                   last_error_at, last_error_code, last_error_message_redacted,
                   metadata, updated_at
            FROM signal_runtime_states
            WHERE source_code = $1 AND connection_id IS NULL AND runtime_kind = $2
            "#,
        )
        .bind(source_code)
        .bind(runtime_kind)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_runtime_state).transpose()
    }

    pub async fn set_source_runtime_state(
        &self,
        source_code: &str,
        state: &str,
    ) -> Result<u64, SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let state = runtime_state_value(state)?;

        let result = sqlx::query(
            r#"
            UPDATE signal_runtime_states
            SET
                state = $2,
                last_started_at = CASE
                    WHEN $2 = 'running' AND state <> 'running' THEN now()
                    ELSE last_started_at
                END,
                last_stopped_at = CASE
                    WHEN $2 IN ('stopped', 'paused', 'muted') AND state <> $2 THEN now()
                    ELSE last_stopped_at
                END,
                last_heartbeat_at = CASE
                    WHEN $2 = 'running' THEN now()
                    ELSE last_heartbeat_at
                END,
                updated_at = now()
            WHERE source_code = $1
              AND connection_id IS NULL
            "#,
        )
        .bind(source_code)
        .bind(state)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn set_runtime_state(
        &self,
        request: &SignalRuntimeStateUpdate,
    ) -> Result<SignalRuntimeState, SignalHubError> {
        let source_code = validate_non_empty("source_code", &request.source_code)?;
        let runtime_kind = validate_non_empty("runtime_kind", &request.runtime_kind)?;
        let state = runtime_state_value(&request.state)?;
        validate_object("metadata", &request.metadata)?;

        let existing = self.runtime_state(&source_code, &runtime_kind).await?;
        if let Some(runtime) = existing {
            let runtime_id = Uuid::parse_str(&runtime.id)
                .map_err(|_| SignalHubError::InvalidRuntimeId(runtime.id.clone()))?;
            sqlx::query(
                r#"
                UPDATE signal_runtime_states
                SET
                    state = $2,
                    last_started_at = CASE
                        WHEN $2 = 'running' AND state <> 'running' THEN now()
                        ELSE last_started_at
                    END,
                    last_stopped_at = CASE
                        WHEN $2 IN ('stopped', 'paused', 'muted') AND state <> $2 THEN now()
                        ELSE last_stopped_at
                    END,
                    last_heartbeat_at = CASE
                        WHEN $2 = 'running' THEN now()
                        ELSE last_heartbeat_at
                    END,
                    metadata = $3,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(runtime_id)
            .bind(state)
            .bind(&request.metadata)
            .execute(&self.pool)
            .await?;
        } else {
            self.ensure_runtime_state(&source_code, &runtime_kind, state, request.metadata.clone())
                .await?;
            if state != "running" {
                sqlx::query(
                    r#"
                    UPDATE signal_runtime_states
                    SET
                        state = $2,
                        last_started_at = CASE WHEN $2 = 'running' THEN now() ELSE last_started_at END,
                        last_stopped_at = CASE
                            WHEN $2 IN ('stopped', 'paused', 'muted') THEN now()
                            ELSE last_stopped_at
                        END,
                        metadata = $3,
                        updated_at = now()
                    WHERE source_code = $1
                      AND connection_id IS NULL
                      AND runtime_kind = $4
                    "#,
                )
                .bind(&source_code)
                .bind(state)
                .bind(&request.metadata)
                .bind(&runtime_kind)
                .execute(&self.pool)
                .await?;
            }
        }

        self.runtime_state(&source_code, &runtime_kind)
            .await?
            .ok_or(SignalHubError::InvalidRuntimeState(state.to_owned()))
    }
}
