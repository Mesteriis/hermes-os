use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::evidence::link_calendar_entity_in_transaction;

use super::errors::MeetingsError;
use super::models::MeetingOutcome;
use super::rows::{MEETING_OUTCOME_COLUMNS, row_to_meeting_outcome};

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

    #[allow(clippy::too_many_arguments)]
    pub async fn add(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        source: Option<&str>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        self.add_with_observation(
            event_id,
            outcome_type,
            title,
            description,
            owner_id,
            due_date,
            source,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_with_observation(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let outcome = Self::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            outcome_type,
            title,
            description,
            owner_id,
            due_date,
            source,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(outcome)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let query = format!(
            "INSERT INTO meeting_outcomes (event_id, outcome_type, title, description, owner_person_id, due_date, source) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING {MEETING_OUTCOME_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(outcome_type)
            .bind(title)
            .bind(description)
            .bind(owner_id)
            .bind(due_date)
            .bind(source.unwrap_or("manual"))
            .fetch_one(&mut **transaction)
            .await?;
        let outcome = row_to_meeting_outcome(row)?;

        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "meeting_outcome",
                outcome.id.clone(),
                None,
                serde_json::json!({
                    "event_id": event_id,
                    "outcome_type": outcome.outcome_type,
                    "linked_entity_id": outcome.linked_entity_id,
                }),
                None,
            )
            .await?;
        }

        Ok(outcome)
    }

    pub(crate) async fn set_linked_entity_id_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        outcome_id: &str,
        linked_entity_id: &str,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let query = format!(
            "UPDATE meeting_outcomes SET linked_entity_id=$2, updated_at=now() WHERE id=$1::uuid RETURNING {MEETING_OUTCOME_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(outcome_id)
            .bind(linked_entity_id)
            .fetch_one(&mut **transaction)
            .await?;
        row_to_meeting_outcome(row)
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
