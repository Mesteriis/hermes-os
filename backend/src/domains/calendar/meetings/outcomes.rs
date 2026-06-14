use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::outcome_projection::project_outcome_domain_record;
use super::rows::{MEETING_OUTCOME_COLUMNS, row_to_meeting_outcome};
use super::{MeetingOutcome, MeetingsError};

#[derive(Clone)]
pub struct MeetingOutcomeStore {
    pool: PgPool,
}

impl MeetingOutcomeStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<MeetingOutcome>, MeetingsError> {
        let query = format!(
            "SELECT {MEETING_OUTCOME_COLUMNS} FROM meeting_outcomes WHERE event_id=$1 ORDER BY outcome_type, title"
        );
        let rows = sqlx::query(&query)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_meeting_outcome).collect()
    }

    pub async fn add(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let query = format!(
            "INSERT INTO meeting_outcomes (event_id, outcome_type, title, description, owner_person_id, due_date) VALUES ($1,$2,$3,$4,$5,$6) RETURNING {MEETING_OUTCOME_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(outcome_type)
            .bind(title)
            .bind(description)
            .bind(owner_id)
            .bind(due_date)
            .fetch_one(&mut *transaction)
            .await?;
        let mut outcome = row_to_meeting_outcome(row)?;

        if let Some(linked_entity_id) =
            project_outcome_domain_record(&mut transaction, &outcome).await?
        {
            let query = format!(
                "UPDATE meeting_outcomes SET linked_entity_id = $1, updated_at = now() WHERE id::text = $2 RETURNING {MEETING_OUTCOME_COLUMNS}"
            );
            let row = sqlx::query(&query)
                .bind(linked_entity_id)
                .bind(&outcome.id)
                .fetch_one(&mut *transaction)
                .await?;
            outcome = row_to_meeting_outcome(row)?;
        }

        transaction.commit().await?;
        Ok(outcome)
    }

    pub async fn follow_up_status(&self, event_id: &str) -> Result<Value, MeetingsError> {
        let rows = sqlx::query(
            "SELECT outcome_type, COUNT(*) as cnt FROM meeting_outcomes WHERE event_id=$1 GROUP BY outcome_type",
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;
        let mut status = serde_json::Map::new();
        for row in &rows {
            let outcome_type: String = row.try_get("outcome_type")?;
            let count: i64 = row.try_get("cnt")?;
            status.insert(outcome_type, serde_json::Value::Number(count.into()));
        }
        Ok(Value::Object(status))
    }
}
