use sqlx::Postgres;
use sqlx::Transaction;

use crate::domains::obligations::ObligationStore;

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
    let (obligation, evidence) = obligation_candidate.to_obligation_draft();
    let stored_obligation =
        ObligationStore::upsert_with_evidence_in_transaction(transaction, &obligation, &[evidence])
            .await?;

    if review_state != TaskCandidateReviewState::UserConfirmed {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO obligation_task_links (
            obligation_id,
            task_id,
            link_kind
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (obligation_id, task_id, link_kind) DO NOTHING
        "#,
    )
    .bind(&stored_obligation.obligation_id)
    .bind(task_id_from_candidate(task_candidate_id))
    .bind(OBLIGATION_TASK_LINK_KIND)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
