use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::CalendarCoreError;
use super::evidence::link_calendar_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRelation {
    pub id: String,
    pub event_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventRelationStore {
    pool: PgPool,
}

impl EventRelationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<EventRelation>, CalendarCoreError> {
        let rows = sqlx::query("SELECT id::text, event_id, entity_type, entity_id, relation_type, source, confidence, created_at FROM event_relations WHERE event_id=$1 ORDER BY entity_type")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(EventRelation {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    entity_type: r.try_get("entity_type")?,
                    entity_id: r.try_get("entity_id")?,
                    relation_type: r.try_get("relation_type")?,
                    source: r.try_get("source")?,
                    confidence: f64::from(r.try_get::<f32, _>("confidence")?),
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn link(
        &self,
        event_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
    ) -> Result<EventRelation, CalendarCoreError> {
        self.link_with_source(event_id, entity_type, entity_id, relation_type, "manual")
            .await
    }

    pub async fn link_with_source(
        &self,
        event_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
        source: &str,
    ) -> Result<EventRelation, CalendarCoreError> {
        self.link_with_observation(
            event_id,
            entity_type,
            entity_id,
            relation_type,
            source,
            None,
        )
        .await
    }

    pub async fn link_with_observation(
        &self,
        event_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
        source: &str,
        observation_id: Option<&str>,
    ) -> Result<EventRelation, CalendarCoreError> {
        if let Some(existing) = self
            .get_by_identity(event_id, entity_type, entity_id, relation_type)
            .await?
        {
            return Ok(existing);
        }
        let row = sqlx::query("INSERT INTO event_relations (event_id, entity_type, entity_id, relation_type, source) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING RETURNING id::text, event_id, entity_type, entity_id, relation_type, source, confidence, created_at")
            .bind(event_id).bind(entity_type).bind(entity_id).bind(relation_type).bind(source).fetch_one(&self.pool).await?;
        let relation = EventRelation {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: f64::from(row.try_get::<f32, _>("confidence")?),
            created_at: row.try_get("created_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity(
                &self.pool,
                observation_id,
                "event_relation",
                relation.id.clone(),
                Some(serde_json::json!({
                    "event_id": event_id,
                    "entity_type": relation.entity_type,
                    "entity_id": relation.entity_id,
                    "relation_type": relation.relation_type,
                })),
            )
            .await?;
        }
        Ok(relation)
    }

    async fn get_by_identity(
        &self,
        event_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
    ) -> Result<Option<EventRelation>, CalendarCoreError> {
        let row = sqlx::query(
            "SELECT id::text, event_id, entity_type, entity_id, relation_type, source, confidence, created_at
             FROM event_relations
             WHERE event_id = $1
               AND entity_type = $2
               AND entity_id = $3
               AND relation_type = $4
             ORDER BY created_at ASC
             LIMIT 1",
        )
        .bind(event_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(relation_type)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| {
            Ok(EventRelation {
                id: r.try_get("id")?,
                event_id: r.try_get("event_id")?,
                entity_type: r.try_get("entity_type")?,
                entity_id: r.try_get("entity_id")?,
                relation_type: r.try_get("relation_type")?,
                source: r.try_get("source")?,
                confidence: f64::from(r.try_get::<f32, _>("confidence")?),
                created_at: r.try_get("created_at")?,
            })
        })
        .transpose()
    }
}
