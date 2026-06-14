use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::CalendarCoreError;

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
        let row = sqlx::query("INSERT INTO event_relations (event_id, entity_type, entity_id, relation_type) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING RETURNING id::text, event_id, entity_type, entity_id, relation_type, source, confidence, created_at")
            .bind(event_id).bind(entity_type).bind(entity_id).bind(relation_type).fetch_one(&self.pool).await?;
        Ok(EventRelation {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: f64::from(row.try_get::<f32, _>("confidence")?),
            created_at: row.try_get("created_at")?,
        })
    }
}
