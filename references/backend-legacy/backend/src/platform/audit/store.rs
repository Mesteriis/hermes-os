use sqlx::Row;
use sqlx::postgres::PgPool;

use super::constants::EVENT_TARGET_KIND;
use super::errors::ApiAuditError;
use super::models::{ApiAuditRecord, NewApiAuditRecord};

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
            WHERE target_kind = $1
              AND ($2::text IS NULL OR target_id = $2)
              AND ($3::text IS NULL OR actor_id = $3)
              AND audit_id > $4
            ORDER BY audit_id ASC
            LIMIT $5
            "#,
        )
        .bind(EVENT_TARGET_KIND)
        .bind(target_id)
        .bind(actor_id)
        .bind(after_audit_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_audit_record).collect()
    }
}

fn row_to_audit_record(row: sqlx::postgres::PgRow) -> Result<ApiAuditRecord, ApiAuditError> {
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
}
