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
    let decision = NewDecision::new(
        outcome.title.clone(),
        description,
        outcome.confidence,
        DecisionReviewState::Suggested,
    )
    .metadata(metadata.clone());
    let evidence =
        NewDecisionEvidence::new(DecisionEvidenceSourceKind::Event, outcome.event_id.clone())
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

    Ok(Some(stored.decision_id))
}

async fn project_obligation_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
) -> Result<Option<String>, MeetingsError> {
    let description = outcome.description.as_deref().unwrap_or(&outcome.title);
    let (obligated_entity_kind, obligated_entity_id) = obligated_entity(outcome);
    let metadata = meeting_outcome_metadata(outcome);
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
    .quote(description)
    .confidence(outcome.confidence)
    .metadata(metadata);
    let stored =
        ObligationStore::upsert_with_evidence_in_transaction(transaction, &obligation, &[evidence])
            .await?;

    Ok(Some(stored.obligation_id))
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
