use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonaMemoryError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaSnapshot {
    pub id: String,
    #[serde(alias = "person_id")]
    pub persona_id: String,
    pub snapshot_date: DateTime<Utc>,
    pub data: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonaSnapshotStore {
    pool: PgPool,
}

impl PersonaSnapshotStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, persona_id: &str) -> Result<Vec<PersonaSnapshot>, PersonaMemoryError> {
        let rows = sqlx::query(
            "SELECT id::text, persona_id, snapshot_date, data, source, created_at
             FROM persona_snapshots WHERE persona_id = $1 ORDER BY snapshot_date DESC LIMIT 20",
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_snapshot).collect()
    }

    pub async fn create(
        &self,
        persona_id: &str,
        data: Value,
        source: &str,
    ) -> Result<PersonaSnapshot, PersonaMemoryError> {
        let row = sqlx::query(
            "INSERT INTO persona_snapshots (persona_id, data, source)
             VALUES ($1, $2, $3)
             RETURNING id::text, persona_id, snapshot_date, data, source, created_at",
        )
        .bind(persona_id)
        .bind(&data)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_snapshot(row)
    }

    pub async fn history_diff(
        &self,
        persona_id: &str,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<HistoryDiff, PersonaMemoryError> {
        let from = sqlx::query(
            "SELECT id::text, persona_id, snapshot_date, data, source, created_at
             FROM persona_snapshots WHERE persona_id = $1 AND snapshot_date <= $2
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .bind(persona_id)
        .bind(from_date)
        .fetch_optional(&self.pool)
        .await?;

        let to = sqlx::query(
            "SELECT id::text, persona_id, snapshot_date, data, source, created_at
             FROM persona_snapshots WHERE persona_id = $1 AND snapshot_date <= $2
             ORDER BY snapshot_date DESC LIMIT 1",
        )
        .bind(persona_id)
        .bind(to_date)
        .fetch_optional(&self.pool)
        .await?;

        let changes = snapshot_changes(&from, &to);

        Ok(HistoryDiff {
            persona_id: persona_id.to_string(),
            from_date: from.map(|r| r.try_get("snapshot_date").unwrap_or(from_date)),
            to_date: to.map(|r| r.try_get("snapshot_date").unwrap_or(to_date)),
            changes,
        })
    }
}

fn row_to_snapshot(row: PgRow) -> Result<PersonaSnapshot, PersonaMemoryError> {
    Ok(PersonaSnapshot {
        id: row.try_get("id")?,
        persona_id: row.try_get("persona_id")?,
        snapshot_date: row.try_get("snapshot_date")?,
        data: row.try_get("data")?,
        source: row.try_get("source")?,
        created_at: row.try_get("created_at")?,
    })
}

fn snapshot_changes(from: &Option<PgRow>, to: &Option<PgRow>) -> Vec<FieldChange> {
    let mut changes: Vec<FieldChange> = Vec::new();
    if let (Some(from_row), Some(to_row)) = (from, to) {
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
    changes
}

#[derive(Clone, Debug, Serialize)]
pub struct HistoryDiff {
    #[serde(rename = "persona_id")]
    pub persona_id: String,
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
