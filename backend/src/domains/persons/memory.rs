use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::engines::memory::{MemoryEngine, MemoryEngineError};
use crate::engines::timeline::{TimelineEngine, TimelineEventDraft};

// ── PersonFact ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonFact {
    pub id: String,
    pub person_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonFactStore {
    pool: PgPool,
}

impl PersonFactStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonFact>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, fact_type, value, source, confidence::float8 AS confidence, last_verified_at,
             valid_from, valid_to, is_active, created_at, updated_at
             FROM person_facts WHERE person_id = $1 ORDER BY created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_fact).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<PersonFact, PersonMemoryError> {
        let fact =
            MemoryEngine::persona_fact_memory(person_id, fact_type, value, source, confidence)?;
        let row = sqlx::query(
            "INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, fact_type, value, source, confidence::float8 AS confidence,
                       last_verified_at, valid_from, valid_to, is_active, created_at, updated_at",
        )
        .bind(&fact.affected_entity_id)
        .bind(&fact.fact_type)
        .bind(&fact.value)
        .bind(&fact.source)
        .bind(fact.confidence)
        .fetch_one(&self.pool)
        .await?;
        row_to_fact(row)
    }

    pub async fn update_confidence(
        &self,
        id: &str,
        confidence: f64,
    ) -> Result<(), PersonMemoryError> {
        sqlx::query("UPDATE person_facts SET confidence = $2, last_verified_at = now(), updated_at = now() WHERE id::text = $1")
            .bind(id).bind(confidence).execute(&self.pool).await?;
        Ok(())
    }
}

fn row_to_fact(row: PgRow) -> Result<PersonFact, PersonMemoryError> {
    Ok(PersonFact {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        fact_type: row.try_get("fact_type")?,
        value: row.try_get("value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// ── PersonMemoryCard ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonMemoryCard {
    pub id: String,
    pub person_id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct PersonMemoryCardStore {
    pool: PgPool,
}

impl PersonMemoryCardStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonMemoryCard>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, title, description, source, confidence::float8 AS confidence, importance,
             created_at, last_verified_at FROM person_memory_cards
             WHERE person_id = $1 ORDER BY importance DESC, created_at DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_memory_card).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        title: &str,
        description: &str,
        source: &str,
        importance: i16,
    ) -> Result<PersonMemoryCard, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_memory_cards (person_id, title, description, source, importance)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, title, description, source, confidence::float8 AS confidence, importance,
                       created_at, last_verified_at",
        )
        .bind(person_id)
        .bind(title)
        .bind(description)
        .bind(source)
        .bind(importance)
        .fetch_one(&self.pool)
        .await?;
        row_to_memory_card(row)
    }
}

fn row_to_memory_card(row: PgRow) -> Result<PersonMemoryCard, PersonMemoryError> {
    Ok(PersonMemoryCard {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        importance: row.try_get("importance")?,
        created_at: row.try_get("created_at")?,
        last_verified_at: row.try_get("last_verified_at")?,
    })
}

// ── PersonPreference ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPreference {
    pub id: String,
    pub person_id: String,
    pub preference_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPreferenceStore {
    pool: PgPool,
}

impl PersonPreferenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonPreference>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, preference_type, value, source, confidence::float8 AS confidence,
             last_verified_at, created_at, updated_at FROM person_preferences
             WHERE person_id = $1 ORDER BY preference_type",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_preference).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        preference_type: &str,
        value: &str,
        source: &str,
    ) -> Result<PersonPreference, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_preferences (person_id, preference_type, value, source)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (person_id, preference_type) DO UPDATE SET value = $3, source = $4, updated_at = now()
             RETURNING id::text, person_id, preference_type, value, source, confidence::float8 AS confidence,
                       last_verified_at, created_at, updated_at"
        ).bind(person_id).bind(preference_type).bind(value).bind(source).fetch_one(&self.pool).await?;
        row_to_preference(row)
    }
}

fn row_to_preference(row: PgRow) -> Result<PersonPreference, PersonMemoryError> {
    Ok(PersonPreference {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        preference_type: row.try_get("preference_type")?,
        value: row.try_get("value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

// ── RelationshipEvent ───────────────────────────────────────────────────────

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

// ── Error ───────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum PersonMemoryError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Memory(#[from] MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
    #[error("fact not found")]
    NotFound,
}

// ── PersonSnapshot ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonSnapshot {
    pub id: String,
    pub person_id: String,
    pub snapshot_date: DateTime<Utc>,
    pub data: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonSnapshotStore {
    pool: PgPool,
}

impl PersonSnapshotStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, person_id: &str) -> Result<Vec<PersonSnapshot>, PersonMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, snapshot_date, data, source, created_at
             FROM person_snapshots WHERE person_id = $1 ORDER BY snapshot_date DESC LIMIT 20",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_snapshot).collect()
    }

    pub async fn create(
        &self,
        person_id: &str,
        data: Value,
        source: &str,
    ) -> Result<PersonSnapshot, PersonMemoryError> {
        let row = sqlx::query(
            "INSERT INTO person_snapshots (person_id, data, source)
             VALUES ($1, $2, $3)
             RETURNING id::text, person_id, snapshot_date, data, source, created_at",
        )
        .bind(person_id)
        .bind(&data)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_snapshot(row)
    }

    /// Compare two snapshots and return the differences.
    pub async fn history_diff(
        &self,
        person_id: &str,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<HistoryDiff, PersonMemoryError> {
        let from = sqlx::query(
            "SELECT id::text, person_id, snapshot_date, data, source, created_at
             FROM person_snapshots WHERE person_id = $1 AND snapshot_date <= $2
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .bind(person_id)
        .bind(from_date)
        .fetch_optional(&self.pool)
        .await?;

        let to = sqlx::query(
            "SELECT id::text, person_id, snapshot_date, data, source, created_at
             FROM person_snapshots WHERE person_id = $1 AND snapshot_date <= $2
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .bind(person_id)
        .bind(to_date)
        .fetch_optional(&self.pool)
        .await?;

        let mut changes: Vec<FieldChange> = Vec::new();
        if let (Some(from_row), Some(to_row)) = (&from, &to) {
            let from_data: Value = from_row.try_get("data").unwrap_or_default();
            let to_data: Value = to_row.try_get("data").unwrap_or_default();
            if let (Some(from_obj), Some(to_obj)) = (from_data.as_object(), to_data.as_object()) {
                for (key, to_val) in to_obj {
                    let from_val = from_obj.get(key);
                    if from_val != Some(to_val) {
                        changes.push(FieldChange {
                            field: key.clone(),
                            old_value: from_val.cloned(),
                            new_value: Some(to_val.clone()),
                        });
                    }
                }
                for key in from_obj.keys() {
                    if !to_obj.contains_key(key) {
                        changes.push(FieldChange {
                            field: key.clone(),
                            old_value: from_obj.get(key).cloned(),
                            new_value: None,
                        });
                    }
                }
            }
        }

        Ok(HistoryDiff {
            person_id: person_id.to_string(),
            from_date: from.map(|r| r.try_get("snapshot_date").unwrap_or(from_date)),
            to_date: to.map(|r| r.try_get("snapshot_date").unwrap_or(to_date)),
            changes,
        })
    }
}

fn row_to_snapshot(row: PgRow) -> Result<PersonSnapshot, PersonMemoryError> {
    Ok(PersonSnapshot {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        snapshot_date: row.try_get("snapshot_date")?,
        data: row.try_get("data")?,
        source: row.try_get("source")?,
        created_at: row.try_get("created_at")?,
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct HistoryDiff {
    pub person_id: String,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub changes: Vec<FieldChange>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}

// ── Memory Decay ────────────────────────────────────────────────────────────

impl PersonFactStore {
    /// Lower confidence for facts not verified within the given threshold days.
    pub async fn decay_unverified(&self, threshold_days: i64) -> Result<u64, PersonMemoryError> {
        let result = sqlx::query(
            "UPDATE person_facts SET confidence = confidence * 0.5, updated_at = now()
             WHERE last_verified_at IS NULL
                OR last_verified_at < now() - ($1 || ' days')::interval",
        )
        .bind(threshold_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
