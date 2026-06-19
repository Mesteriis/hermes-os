use sqlx::Postgres;
use sqlx::Transaction;

use crate::domains::obligations::{NewObligationEvidence, ObligationStore};
use crate::domains::tasks::core::{ObligationTaskLinkStore, TaskCoreError};

use super::super::constants::{OBLIGATION_TASK_LINK_KIND, TASK_CANDIDATE_KIND_OBLIGATION_TASK};
use super::super::errors::TaskCandidateError;
use super::super::extraction::{
    obligation_candidate_from_metadata, obligation_review_state_from_task_candidate,
};
use super::super::ids::task_id_from_candidate;
use super::super::models::{StoredCandidateRow, TaskCandidateReviewState};

pub(super) async fn sync_obligation_candidate_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    review_state: TaskCandidateReviewState,
) -> Result<(), TaskCandidateError> {
    if candidate.candidate_kind != TASK_CANDIDATE_KIND_OBLIGATION_TASK {
        return Ok(());
    }

    let mut obligation_candidate = obligation_candidate_from_metadata(candidate)?;
    obligation_candidate.review_state = obligation_review_state_from_task_candidate(review_state);
    let (obligation, mut evidence) = obligation_candidate.to_obligation_draft();
    if let Some(observation_id) = &candidate.observation_id {
        evidence = NewObligationEvidence::observation(observation_id.clone())
            .quote(obligation_candidate.quote.clone())
            .confidence(obligation_candidate.confidence)
            .metadata(serde_json::json!({
                "engine": "obligation",
                "candidate_kind": obligation_candidate.kind.as_str(),
                "task_candidate_id": task_candidate_id,
                "legacy_source_kind": candidate.source_kind,
                "legacy_source_id": candidate.source_id,
            }));
    }
    let stored_obligation =
        ObligationStore::upsert_with_evidence_in_transaction(transaction, &obligation, &[evidence])
            .await?;

    if review_state != TaskCandidateReviewState::UserConfirmed {
        return Ok(());
    }

    debug_assert_eq!(OBLIGATION_TASK_LINK_KIND, "fulfillment_task");
    ObligationTaskLinkStore::link_fulfillment_task_in_transaction(
        transaction,
        &stored_obligation.obligation_id,
        &task_id_from_candidate(task_candidate_id),
    )
    .await
    .map_err(|error| match error {
        TaskCoreError::Sqlx(inner) => TaskCandidateError::Sqlx(inner),
        TaskCoreError::Relationship(inner) => TaskCandidateError::InvalidCandidateMetadata(
            format!("unexpected relationship error while linking obligation task: {inner}"),
        ),
        TaskCoreError::ContextPack(inner) => TaskCandidateError::InvalidCandidateMetadata(format!(
            "unexpected context pack error while linking obligation task: {inner}"
        )),
        TaskCoreError::Observation(inner) => TaskCandidateError::ObservationStore(inner),
        TaskCoreError::NotFound => TaskCandidateError::InvalidCandidateMetadata(
            "obligation task link target was not found".to_owned(),
        ),
    })?;

    Ok(())
}
