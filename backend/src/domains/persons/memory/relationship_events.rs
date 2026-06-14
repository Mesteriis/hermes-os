use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonMemoryError;
use crate::engines::timeline::{TimelineEngine, TimelineEventDraft};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub id: String,
    pub person_id: String,
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
pub struct RelationshipEventStore {
    pool: PgPool,
}

impl RelationshipEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn timeline(
        &self,
        person_id: &str,
        limit: i64,
    ) -> Result<Vec<RelationshipEvent>, PersonMemoryError> {
        let limit = TimelineEngine::bounded_entity_limit(limit);
        let rows = sqlx::query(
            "SELECT id::text, person_id, event_type, title, description, occurred_at, source,
             related_entity_id, related_entity_kind, confidence::float8 AS confidence, metadata, created_at
             FROM relationship_events WHERE person_id = $1
             ORDER BY occurred_at DESC LIMIT $2",
        )
        .bind(person_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_event).collect()
    }

    pub async fn add(
        &self,
        event: &NewRelationshipEvent,
    ) -> Result<RelationshipEvent, PersonMemoryError> {
        TimelineEngine::validate_event(&TimelineEventDraft {
            entity_kind: "persona",
            entity_id: &event.person_id,
            event_type: &event.event_type,
            title: &event.title,
            occurred_at: event.occurred_at,
            source: &event.source,
        })?;

        let row = sqlx::query(
            "INSERT INTO relationship_events (person_id, event_type, title, description,
             occurred_at, source, related_entity_id, related_entity_kind)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id::text, person_id, event_type, title, description, occurred_at, source,
                       related_entity_id, related_entity_kind, confidence::float8 AS confidence, metadata, created_at",
        )
        .bind(&event.person_id)
        .bind(&event.event_type)
        .bind(&event.title)
        .bind(&event.description)
        .bind(event.occurred_at)
        .bind(&event.source)
        .bind(&event.related_entity_id)
        .bind(&event.related_entity_kind)
        .fetch_one(&self.pool)
        .await?;
        row_to_event(row)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewRelationshipEvent {
    pub person_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
}

fn row_to_event(row: PgRow) -> Result<RelationshipEvent, PersonMemoryError> {
    Ok(RelationshipEvent {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
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
