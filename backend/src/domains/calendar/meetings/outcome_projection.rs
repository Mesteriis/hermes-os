use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};

use super::{MeetingOutcome, MeetingsError};
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

pub(super) async fn project_outcome_domain_record(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
) -> Result<Option<String>, MeetingsError> {
    match outcome.outcome_type.as_str() {
        "decision" => project_decision_outcome(transaction, outcome).await,
        "task" | "promise" | "follow_up" => project_obligation_outcome(transaction, outcome).await,
        _ => Ok(None),
    }
}

async fn project_decision_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
) -> Result<Option<String>, MeetingsError> {
    let description = outcome.description.as_deref().unwrap_or(&outcome.title);
    let metadata = meeting_outcome_metadata(outcome);
    let observation_id = event_observation_id(transaction, &outcome.event_id).await?;
    let decision = NewDecision::new(
        outcome.title.clone(),
        description,
        outcome.confidence,
        DecisionReviewState::Suggested,
    )
    .metadata(metadata.clone());
    let evidence =
        NewDecisionEvidence::new(DecisionEvidenceSourceKind::Event, outcome.event_id.clone())
            .with_observation_id(Some(observation_id.clone()))
            .quote(description)
            .confidence(outcome.confidence)
            .metadata(metadata.clone());
    let impact = NewDecisionImpactedEntity::new(DecisionEntityKind::Event, &outcome.event_id)
        .impact_type("meeting_outcome")
        .metadata(metadata);
    let stored = DecisionStore::upsert_with_evidence_in_transaction(
        transaction,
        &decision,
        &[evidence],
        &[impact],
    )
    .await?;
    sync_decision_review_state_in_transaction(transaction, &stored).await?;

    Ok(Some(stored.decision_id))
}

async fn project_obligation_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
) -> Result<Option<String>, MeetingsError> {
    let description = outcome.description.as_deref().unwrap_or(&outcome.title);
    let (obligated_entity_kind, obligated_entity_id) = obligated_entity(outcome);
    let metadata = meeting_outcome_metadata(outcome);
    let observation_id = event_observation_id(transaction, &outcome.event_id).await?;
    let mut obligation = NewObligation::new(
        obligated_entity_kind,
        obligated_entity_id,
        outcome.title.clone(),
        outcome.confidence,
        ObligationReviewState::Suggested,
    )
    .metadata(metadata.clone());
    if let Some(due_date) = outcome.due_date {
        obligation = obligation.due_at(due_date);
    }
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Event,
        outcome.event_id.clone(),
    )
    .with_observation_id(Some(observation_id.clone()))
    .quote(description)
    .confidence(outcome.confidence)
    .metadata(metadata);
    let stored =
        ObligationStore::upsert_with_evidence_in_transaction(transaction, &obligation, &[evidence])
            .await?;
    sync_obligation_review_state_in_transaction(transaction, &stored).await?;

    Ok(Some(stored.obligation_id))
}

async fn event_observation_id(
    transaction: &mut Transaction<'_, Postgres>,
    event_id: &str,
) -> Result<String, MeetingsError> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT observation_id FROM calendar_events WHERE event_id = $1",
    )
    .bind(event_id)
    .fetch_one(&mut **transaction)
    .await?)
}

fn obligated_entity(outcome: &MeetingOutcome) -> (ObligationEntityKind, String) {
    outcome
        .owner_person_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|owner_person_id| (ObligationEntityKind::Persona, owner_person_id.to_owned()))
        .unwrap_or_else(|| (ObligationEntityKind::Event, outcome.event_id.clone()))
}

fn meeting_outcome_metadata(outcome: &MeetingOutcome) -> Value {
    json!({
        "source": "meeting_outcome_adapter",
        "meeting_outcome_id": outcome.id,
        "event_id": outcome.event_id,
        "outcome_type": outcome.outcome_type,
    })
}
