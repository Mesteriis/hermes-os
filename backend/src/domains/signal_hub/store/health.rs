use super::validation::{parse_required_uuid, validate_non_empty, validate_object};
use super::{
    SignalConnection, SignalHealth, SignalHealthCheckRequest, SignalHealthSnapshotWrite,
    SignalHubError, SignalHubStore, SignalRuntimeState, SignalSource,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

impl SignalHubStore {
    pub async fn list_health(&self) -> Result<Vec<SignalHealth>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT id, source_code, connection_id, level, summary, last_ok_at,
                   last_failure_at, failure_count, consecutive_failure_count,
                   next_retry_at, evidence, updated_at
            FROM signal_health
            ORDER BY source_code ASC, updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_health).collect()
    }

    pub async fn run_health_check(
        &self,
        request: &SignalHealthCheckRequest,
    ) -> Result<SignalHealth, SignalHubError> {
        let source_code = validate_non_empty("source_code", &request.source_code)?;
        let source = self
            .source_by_code(&source_code)
            .await?
            .ok_or_else(|| SignalHubError::SourceNotFound(source_code.clone()))?;
        let connection = match request.connection_id.as_deref() {
            Some(connection_id) => {
                let id = Uuid::parse_str(connection_id.trim())
                    .map_err(|_| SignalHubError::InvalidConnectionId(connection_id.to_owned()))?;
                Some(
                    self.connection_by_id(id).await?.ok_or_else(|| {
                        SignalHubError::ConnectionNotFound(connection_id.to_owned())
                    })?,
                )
            }
            None => None,
        };
        if connection
            .as_ref()
            .is_some_and(|connection| connection.source_code != source_code)
        {
            return Err(SignalHubError::InvalidConnectionId(
                request.connection_id.clone().unwrap_or_default(),
            ));
        }

        let runtime = self
            .runtime_state(
                &source_code,
                request
                    .runtime_kind
                    .as_deref()
                    .unwrap_or("signal_health_check"),
            )
            .await?;
        self.upsert_health_snapshot(
            request,
            SignalHealthSnapshotWrite::from(build_health_snapshot(
                &source,
                connection.as_ref(),
                runtime.as_ref(),
            )),
        )
        .await
    }

    pub async fn upsert_health_snapshot(
        &self,
        request: &SignalHealthCheckRequest,
        snapshot: SignalHealthSnapshotWrite,
    ) -> Result<SignalHealth, SignalHubError> {
        let source_code = validate_non_empty("source_code", &request.source_code)?;
        validate_object("evidence", &snapshot.evidence)?;
        let existing = self
            .health_by_scope(&source_code, request.connection_id.as_deref())
            .await?;
        let health_id = existing
            .as_ref()
            .map(|health| Uuid::parse_str(&health.id))
            .transpose()
            .map_err(|_| {
                SignalHubError::InvalidHealthId(existing.map(|item| item.id).unwrap_or_default())
            })?
            .unwrap_or_else(Uuid::now_v7);
        let connection_uuid = request
            .connection_id
            .as_deref()
            .map(parse_required_uuid)
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO signal_health (
                id, source_code, connection_id, level, summary, last_ok_at, last_failure_at,
                failure_count, consecutive_failure_count, next_retry_at, evidence, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (id) DO UPDATE SET
                level = EXCLUDED.level,
                summary = EXCLUDED.summary,
                last_ok_at = EXCLUDED.last_ok_at,
                last_failure_at = EXCLUDED.last_failure_at,
                failure_count = EXCLUDED.failure_count,
                consecutive_failure_count = EXCLUDED.consecutive_failure_count,
                next_retry_at = EXCLUDED.next_retry_at,
                evidence = EXCLUDED.evidence,
                updated_at = now()
            "#,
        )
        .bind(health_id)
        .bind(&source_code)
        .bind(connection_uuid)
        .bind(&snapshot.level)
        .bind(&snapshot.summary)
        .bind(snapshot.last_ok_at)
        .bind(snapshot.last_failure_at)
        .bind(snapshot.failure_count)
        .bind(snapshot.consecutive_failure_count)
        .bind(snapshot.next_retry_at)
        .bind(snapshot.evidence)
        .execute(&self.pool)
        .await?;

        self.health_by_id(health_id)
            .await?
            .ok_or_else(|| SignalHubError::InvalidHealthId(health_id.to_string()))
    }

    async fn health_by_scope(
        &self,
        source_code: &str,
        connection_id: Option<&str>,
    ) -> Result<Option<SignalHealth>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT id, source_code, connection_id, level, summary, last_ok_at,
                   last_failure_at, failure_count, consecutive_failure_count,
                   next_retry_at, evidence, updated_at
            FROM signal_health
            WHERE source_code = $1
              AND ($2::uuid IS NULL OR connection_id = $2)
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(source_code)
        .bind(connection_id.map(parse_required_uuid).transpose()?)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_health).transpose()
    }

    async fn health_by_id(&self, id: Uuid) -> Result<Option<SignalHealth>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT id, source_code, connection_id, level, summary, last_ok_at,
                   last_failure_at, failure_count, consecutive_failure_count,
                   next_retry_at, evidence, updated_at
            FROM signal_health
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_health).transpose()
    }
}

fn row_to_health(row: PgRow) -> Result<SignalHealth, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    Ok(SignalHealth {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        level: row.try_get("level")?,
        summary: row.try_get("summary")?,
        last_ok_at: row.try_get("last_ok_at")?,
        last_failure_at: row.try_get("last_failure_at")?,
        failure_count: row.try_get("failure_count")?,
        consecutive_failure_count: row.try_get("consecutive_failure_count")?,
        next_retry_at: row.try_get("next_retry_at")?,
        evidence: row.try_get("evidence")?,
        updated_at: row.try_get("updated_at")?,
    })
}

struct SignalHealthSnapshot {
    level: String,
    summary: String,
    last_ok_at: Option<DateTime<Utc>>,
    last_failure_at: Option<DateTime<Utc>>,
    failure_count: i32,
    consecutive_failure_count: i32,
    next_retry_at: Option<DateTime<Utc>>,
    evidence: Value,
}

impl From<SignalHealthSnapshot> for SignalHealthSnapshotWrite {
    fn from(value: SignalHealthSnapshot) -> Self {
        Self {
            level: value.level,
            summary: value.summary,
            last_ok_at: value.last_ok_at,
            last_failure_at: value.last_failure_at,
            failure_count: value.failure_count,
            consecutive_failure_count: value.consecutive_failure_count,
            next_retry_at: value.next_retry_at,
            evidence: value.evidence,
        }
    }
}

fn build_health_snapshot(
    source: &SignalSource,
    connection: Option<&SignalConnection>,
    runtime: Option<&SignalRuntimeState>,
) -> SignalHealthSnapshot {
    if let Some(connection) = connection {
        match connection.status.as_str() {
            "disabled" => {
                return SignalHealthSnapshot {
                    level: "disabled".to_owned(),
                    summary: format!("{} connection is disabled", source.display_name),
                    last_ok_at: None,
                    last_failure_at: Some(Utc::now()),
                    failure_count: 1,
                    consecutive_failure_count: 1,
                    next_retry_at: None,
                    evidence: serde_json::json!({ "source_status": connection.status, "connection_id": connection.id }),
                };
            }
            "error" | "disconnected" | "degraded" => {
                return SignalHealthSnapshot {
                    level: "degraded".to_owned(),
                    summary: format!("{} connection requires attention", source.display_name),
                    last_ok_at: None,
                    last_failure_at: Some(Utc::now()),
                    failure_count: 1,
                    consecutive_failure_count: 1,
                    next_retry_at: Some(Utc::now() + chrono::Duration::minutes(5)),
                    evidence: serde_json::json!({ "source_status": connection.status, "connection_id": connection.id }),
                };
            }
            _ => {}
        }
    }

    if let Some(runtime) = runtime {
        return match runtime.state.as_str() {
            "running" => SignalHealthSnapshot {
                level: "healthy".to_owned(),
                summary: format!("{} runtime is healthy", source.display_name),
                last_ok_at: Some(Utc::now()),
                last_failure_at: None,
                failure_count: 0,
                consecutive_failure_count: 0,
                next_retry_at: None,
                evidence: serde_json::json!({ "runtime_kind": runtime.runtime_kind, "runtime_state": runtime.state, "source_code": source.code }),
            },
            "paused" | "muted" | "stopped" => SignalHealthSnapshot {
                level: "degraded".to_owned(),
                summary: format!("{} runtime is {}", source.display_name, runtime.state),
                last_ok_at: None,
                last_failure_at: Some(Utc::now()),
                failure_count: 1,
                consecutive_failure_count: 1,
                next_retry_at: None,
                evidence: serde_json::json!({ "runtime_kind": runtime.runtime_kind, "runtime_state": runtime.state, "source_code": source.code }),
            },
            _ => SignalHealthSnapshot {
                level: "unknown".to_owned(),
                summary: format!("{} runtime state is {}", source.display_name, runtime.state),
                last_ok_at: None,
                last_failure_at: None,
                failure_count: 0,
                consecutive_failure_count: 0,
                next_retry_at: None,
                evidence: serde_json::json!({ "runtime_kind": runtime.runtime_kind, "runtime_state": runtime.state, "source_code": source.code }),
            },
        };
    }

    SignalHealthSnapshot {
        level: if source.supports_runtime {
            "unknown".to_owned()
        } else {
            "healthy".to_owned()
        },
        summary: if source.supports_runtime {
            format!("{} runtime is not registered", source.display_name)
        } else {
            format!("{} source has no runtime checks", source.display_name)
        },
        last_ok_at: (!source.supports_runtime).then(Utc::now),
        last_failure_at: None,
        failure_count: 0,
        consecutive_failure_count: 0,
        next_retry_at: None,
        evidence: serde_json::json!({ "source_code": source.code, "supports_runtime": source.supports_runtime }),
    }
}
