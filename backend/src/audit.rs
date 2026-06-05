use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

const LOCAL_API_TOKEN_ACTOR_KIND: &str = "local_api_token";
const EVENT_TARGET_KIND: &str = "event";

#[derive(Clone)]
pub struct ApiAuditLog {
    pool: PgPool,
}

impl ApiAuditLog {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(&self, record: &NewApiAuditRecord) -> Result<i64, ApiAuditError> {
        let audit_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO api_audit_log (
                actor_kind,
                actor_id,
                operation,
                method,
                path_template,
                target_kind,
                target_id,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING audit_id
            "#,
        )
        .bind(&record.actor_kind)
        .bind(&record.actor_id)
        .bind(&record.operation)
        .bind(&record.method)
        .bind(&record.path_template)
        .bind(&record.target_kind)
        .bind(&record.target_id)
        .bind(&record.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(audit_id)
    }

    pub async fn list_event_records(
        &self,
        target_id: Option<&str>,
        actor_id: Option<&str>,
        after_audit_id: i64,
        limit: u32,
    ) -> Result<Vec<ApiAuditRecord>, ApiAuditError> {
        let target_id = target_id.map(str::trim).filter(|value| !value.is_empty());
        let actor_id = actor_id.map(str::trim).filter(|value| !value.is_empty());
        let after_audit_id = after_audit_id.max(0);
        let limit = i64::from(limit.clamp(1, 500));

        let rows = sqlx::query(
            r#"
            SELECT
                audit_id,
                recorded_at,
                actor_kind,
                actor_id,
                operation,
                method,
                path_template,
                target_kind,
                target_id,
                metadata
            FROM api_audit_log
            WHERE target_kind = 'event'
              AND ($1::text IS NULL OR target_id = $1)
              AND ($2::text IS NULL OR actor_id = $2)
              AND audit_id > $3
            ORDER BY audit_id ASC
            LIMIT $4
            "#,
        )
        .bind(target_id)
        .bind(actor_id)
        .bind(after_audit_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ApiAuditRecord {
                    audit_id: row.try_get("audit_id")?,
                    recorded_at: row.try_get("recorded_at")?,
                    actor_kind: row.try_get("actor_kind")?,
                    actor_id: row.try_get("actor_id")?,
                    operation: row.try_get("operation")?,
                    method: row.try_get("method")?,
                    path_template: row.try_get("path_template")?,
                    target_kind: row.try_get("target_kind")?,
                    target_id: row.try_get("target_id")?,
                    metadata: row.try_get("metadata")?,
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApiAuditRecord {
    pub audit_id: i64,
    pub recorded_at: DateTime<Utc>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub operation: String,
    pub method: String,
    pub path_template: String,
    pub target_kind: String,
    pub target_id: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewApiAuditRecord {
    actor_kind: String,
    actor_id: String,
    operation: String,
    method: String,
    path_template: String,
    target_kind: String,
    target_id: Option<String>,
    metadata: Value,
}

impl NewApiAuditRecord {
    pub fn event_append(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            actor_kind: LOCAL_API_TOKEN_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "event.append".to_owned(),
            method: "POST".to_owned(),
            path_template: "/api/events".to_owned(),
            target_kind: EVENT_TARGET_KIND.to_owned(),
            target_id: Some(event_id.into()),
            metadata: json!({}),
        }
    }

    pub fn event_get(actor_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            actor_kind: LOCAL_API_TOKEN_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "event.get".to_owned(),
            method: "GET".to_owned(),
            path_template: "/api/events/{event_id}".to_owned(),
            target_kind: EVENT_TARGET_KIND.to_owned(),
            target_id: Some(event_id.into()),
            metadata: json!({}),
        }
    }

    pub fn project_link_review_set(
        actor_id: impl Into<String>,
        project_id: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        let project_id = project_id.into();
        let target_kind = target_kind.into();
        let target_id = target_id.into();

        Self {
            actor_kind: LOCAL_API_TOKEN_ACTOR_KIND.to_owned(),
            actor_id: actor_id.into(),
            operation: "project.link_review.set".to_owned(),
            method: "PUT".to_owned(),
            path_template: "/api/v2/projects/{project_id}/link-reviews".to_owned(),
            target_kind: "project_link".to_owned(),
            target_id: Some(format!("{project_id}:{target_kind}:{target_id}")),
            metadata: json!({
                "project_id": project_id,
                "target_kind": target_kind,
                "target_id": target_id,
            }),
        }
    }
}

#[derive(Debug, Error)]
pub enum ApiAuditError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
