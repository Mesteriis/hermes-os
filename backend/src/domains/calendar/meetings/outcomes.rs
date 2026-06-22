use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::evidence::link_calendar_entity_in_transaction;
use crate::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, DecisionStore,
    NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};
use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationReviewState, ObligationStore,
};
use crate::workflows::review_mirror::{
    sync_decision_review_state_in_transaction, sync_obligation_review_state_in_transaction,
};

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
            .fetch_one(&mut *transaction)
            .await?;
        let mut outcome = row_to_meeting_outcome(row)?;

        if let Some(linked_entity_id) =
            linked_entity_for_outcome(&mut transaction, &outcome, observation_id).await?
        {
            let query = format!(
                "UPDATE meeting_outcomes SET linked_entity_id=$2, updated_at=now() WHERE id=$1::uuid RETURNING {MEETING_OUTCOME_COLUMNS}"
            );
            let row = sqlx::query(&query)
                .bind(&outcome.id)
                .bind(&linked_entity_id)
                .fetch_one(&mut *transaction)
                .await?;
            outcome = row_to_meeting_outcome(row)?;
        }

        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
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

async fn linked_entity_for_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
    observation_id: Option<&str>,
) -> Result<Option<String>, MeetingsError> {
    let evidence_observation_id = match observation_id.filter(|value| !value.trim().is_empty()) {
        Some(observation_id) => observation_id.to_owned(),
        None => calendar_event_observation_id(transaction, &outcome.event_id).await?,
    };

    match outcome.outcome_type.as_str() {
        "decision" => {
            let decision = NewDecision::new(
                outcome.title.clone(),
                outcome
                    .description
                    .clone()
                    .unwrap_or_else(|| outcome.title.clone()),
                outcome.confidence,
                DecisionReviewState::Suggested,
            )
            .metadata(json!({
                "source_domain": "calendar",
                "source_entity_kind": "meeting_outcome",
                "meeting_outcome_id": outcome.id,
                "event_id": outcome.event_id,
            }));
            let evidence = NewDecisionEvidence::new(
                DecisionEvidenceSourceKind::Event,
                outcome.event_id.clone(),
            )
            .with_observation_id(Some(evidence_observation_id.clone()))
            .quote(
                outcome
                    .description
                    .clone()
                    .unwrap_or_else(|| outcome.title.clone()),
            )
            .metadata(json!({
                "source_domain": "calendar",
                "meeting_outcome_id": outcome.id,
            }));
            let impacted_entity =
                NewDecisionImpactedEntity::new(DecisionEntityKind::Event, outcome.event_id.clone())
                    .impact_type("meeting_outcome")
                    .metadata(json!({ "meeting_outcome_id": outcome.id }));
            let stored = DecisionStore::upsert_with_evidence_in_transaction(
                transaction,
                &decision,
                &[evidence],
                &[impacted_entity],
            )
            .await?;
            sync_decision_review_state_in_transaction(transaction, &stored).await?;
            Ok(Some(stored.decision_id))
        }
        "promise" => {
            let Some(owner_person_id) = outcome
                .owner_person_id
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            else {
                return Ok(None);
            };
            let mut obligation = NewObligation::new(
                ObligationEntityKind::Persona,
                owner_person_id.to_owned(),
                outcome.title.clone(),
                outcome.confidence,
                ObligationReviewState::Suggested,
            )
            .metadata(json!({
                "source_domain": "calendar",
                "source_entity_kind": "meeting_outcome",
                "meeting_outcome_id": outcome.id,
                "event_id": outcome.event_id,
            }));
            if let Some(due_date) = outcome.due_date {
                obligation = obligation.due_at(due_date);
            }
            let evidence = NewObligationEvidence::new(
                ObligationEvidenceSourceKind::Event,
                outcome.event_id.clone(),
            )
            .with_observation_id(Some(evidence_observation_id.clone()))
            .quote(
                outcome
                    .description
                    .clone()
                    .unwrap_or_else(|| outcome.title.clone()),
            )
            .metadata(json!({
                "source_domain": "calendar",
                "meeting_outcome_id": outcome.id,
            }));
            let stored = ObligationStore::upsert_with_evidence_in_transaction(
                transaction,
                &obligation,
                &[evidence],
            )
            .await?;
            sync_obligation_review_state_in_transaction(transaction, &stored).await?;
            Ok(Some(stored.obligation_id))
        }
        _ => Ok(None),
    }
}

async fn calendar_event_observation_id(
    transaction: &mut Transaction<'_, Postgres>,
    event_id: &str,
) -> Result<String, MeetingsError> {
    let observation_id = sqlx::query_scalar::<_, String>(
        "SELECT observation_id FROM calendar_events WHERE event_id = $1",
    )
    .bind(event_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(MeetingsError::NotFound)?;
    Ok(observation_id)
}
