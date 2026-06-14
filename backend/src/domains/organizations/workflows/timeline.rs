use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::engines::timeline::{TimelineEngine, TimelineEventDraft};

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgTimelineEvent {
    pub id: String,
    pub organization_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgTimelineStore {
    pool: PgPool,
}

impl OrgTimelineStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        org_id: &str,
        limit: i64,
    ) -> Result<Vec<OrgTimelineEvent>, OrgWorkflowError> {
        let limit = TimelineEngine::bounded_entity_limit(limit);
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, event_type, title, description, occurred_at,
                   source, related_entity_id, related_entity_kind, confidence, metadata, created_at
            FROM organization_timeline_events
            WHERE organization_id=$1
            ORDER BY occurred_at DESC
            LIMIT $2
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(timeline_event_from_row).collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        event_type: &str,
        title: &str,
        occurred_at: DateTime<Utc>,
        source: &str,
    ) -> Result<OrgTimelineEvent, OrgWorkflowError> {
        TimelineEngine::validate_event(&TimelineEventDraft {
            entity_kind: "organization",
            entity_id: org_id,
            event_type,
            title,
            occurred_at,
            source,
        })?;

        let row = sqlx::query(
            r#"
            INSERT INTO organization_timeline_events (organization_id, event_type, title, occurred_at, source)
            VALUES ($1,$2,$3,$4,$5)
            RETURNING id::text, organization_id, event_type, title, description, occurred_at,
                      source, related_entity_id, related_entity_kind, confidence, metadata, created_at
            "#,
        )
        .bind(org_id)
        .bind(event_type)
        .bind(title)
        .bind(occurred_at)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;

        timeline_event_from_row(row)
    }
}

fn timeline_event_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<OrgTimelineEvent, OrgWorkflowError> {
    Ok(OrgTimelineEvent {
        id: row.try_get("id")?,
        organization_id: row.try_get("organization_id")?,
        event_type: row.try_get("event_type")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        occurred_at: row.try_get("occurred_at")?,
        source: row.try_get("source")?,
        related_entity_id: row.try_get("related_entity_id")?,
        related_entity_kind: row.try_get("related_entity_kind")?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}
