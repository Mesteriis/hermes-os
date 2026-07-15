use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonaMemoryError;
use crate::domains::personas::core::evidence::{
    link_persona_entity, link_persona_entity_in_transaction,
};
use crate::engines::timeline::TimelineEngine;
use crate::engines::timeline::models::TimelineEventDraft;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub id: String,
    #[serde(alias = "person_id")]
    pub persona_id: String,
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
        persona_id: &str,
        limit: i64,
    ) -> Result<Vec<RelationshipEvent>, PersonaMemoryError> {
        let limit = TimelineEngine::bounded_entity_limit(limit);
        let rows = sqlx::query(
            "SELECT id::text, persona_id, event_type, title, description, occurred_at, source,
             related_entity_id, related_entity_kind, confidence::float8 AS confidence, metadata, created_at
             FROM relationship_events WHERE persona_id = $1
             ORDER BY occurred_at DESC LIMIT $2",
        )
        .bind(persona_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_event).collect()
    }

    pub async fn add(
        &self,
        event: &NewRelationshipEvent,
    ) -> Result<RelationshipEvent, PersonaMemoryError> {
        TimelineEngine::validate_event(&TimelineEventDraft {
            entity_kind: "persona",
            entity_id: &event.persona_id,
            event_type: &event.event_type,
            title: &event.title,
            occurred_at: event.occurred_at,
            source: &event.source,
        })?;

        let row = sqlx::query(
            "INSERT INTO relationship_events (persona_id, event_type, title, description,
             occurred_at, source, related_entity_id, related_entity_kind)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id::text, persona_id, event_type, title, description, occurred_at, source,
                       related_entity_id, related_entity_kind, confidence::float8 AS confidence, metadata, created_at",
        )
        .bind(&event.persona_id)
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

    pub async fn add_with_observation(
        &self,
        event: &NewRelationshipEvent,
        observation_id: &str,
    ) -> Result<RelationshipEvent, PersonaMemoryError> {
        let event_record = self.add(event).await?;
        link_persona_entity(
            &self.pool,
            observation_id,
            "relationship_event",
            event_record.id.clone(),
            None,
            Some(json!({
                "persona_id": event_record.persona_id,
                "event_type": event_record.event_type,
            })),
        )
        .await?;
        Ok(event_record)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_email_message_event(
        &self,
        observation_id: &str,
        message_id: &str,
        occurred_at: DateTime<Utc>,
        persona_id: &str,
        event_type: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<bool, PersonaMemoryError> {
        TimelineEngine::validate_event(&TimelineEventDraft {
            entity_kind: "persona",
            entity_id: persona_id,
            event_type,
            title,
            occurred_at,
            source: "email_sync",
        })?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO relationship_events (
                persona_id,
                event_type,
                title,
                description,
                occurred_at,
                source,
                related_entity_id,
                related_entity_kind,
                metadata
            )
            SELECT
                $1,
                $2,
                $3,
                $4,
                $5,
                'email_sync',
                $6,
                'communication_message',
                '{}'::jsonb
            WHERE NOT EXISTS (
                SELECT 1
                FROM relationship_events
                WHERE persona_id = $1
                  AND event_type = $2
                  AND related_entity_id = $6
                  AND related_entity_kind = 'communication_message'
            )
            RETURNING id::text AS event_id
            "#,
        )
        .bind(persona_id)
        .bind(event_type)
        .bind(title)
        .bind(description)
        .bind(occurred_at)
        .bind(message_id)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            transaction.commit().await?;
            return Ok(false);
        };
        let event_id: String = row.try_get("event_id")?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "relationship_event",
            event_id,
            Some("email_sync_relationship_event"),
            Some(json!({
                "persona_id": persona_id,
                "event_type": event_type,
                "related_entity_id": message_id,
                "related_entity_kind": "communication_message",
                "source": "email_sync",
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(true)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewRelationshipEvent {
    #[serde(alias = "person_id")]
    pub persona_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
}

fn row_to_event(row: PgRow) -> Result<RelationshipEvent, PersonaMemoryError> {
    Ok(RelationshipEvent {
        id: row.try_get("id")?,
        persona_id: row.try_get("persona_id")?,
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
