use super::*;

pub(crate) async fn sync_task_candidate_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    review_state: TaskCandidateReviewState,
) -> Result<(), ReviewMirrorError> {
    let observation_id = candidate
        .observation_id
        .clone()
        .or_else(|| (candidate.source_kind == "observation").then(|| candidate.source_id.clone()))
        .ok_or_else(|| {
            ReviewMirrorError::ObservationRequired(format!("task_candidate:{task_candidate_id}"))
        })?;

    let review_item = ensure_task_candidate_review_item_in_transaction(
        transaction,
        task_candidate_id,
        candidate,
        &observation_id,
    )
    .await?;

    match review_state {
        TaskCandidateReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        TaskCandidateReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        TaskCandidateReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "tasks",
                    "task",
                    task_id_from_candidate(task_candidate_id),
                ),
            )
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn ensure_task_candidate_review_item(
    pool: &sqlx::postgres::PgPool,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let item = ensure_task_candidate_review_item_in_transaction(
        &mut transaction,
        task_candidate_id,
        candidate,
        observation_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(item)
}
pub(crate) async fn ensure_task_candidate_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    match ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::PotentialTask,
        &json!({ "task_candidate_id": task_candidate_id }),
    )
    .await?
    {
        Some(item) => {
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "task_candidates",
                    "task_candidate_id": task_candidate_id,
                }));
            Ok(ReviewInboxPort::attach_evidence_in_transaction(
                transaction,
                &item.review_item_id,
                &[evidence],
            )
            .await?)
        }
        None => {
            let summary = if candidate.evidence_excerpt.trim().is_empty() {
                "Evidence-backed task candidate".to_owned()
            } else {
                candidate.evidence_excerpt.clone()
            };
            let item = NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                candidate.title.clone(),
                summary,
                candidate.confidence,
            )
            .metadata(json!({
                "mirrored_from": "task_candidates",
                "task_candidate_id": task_candidate_id,
                "candidate_kind": candidate.candidate_kind,
                "due_text": candidate.due_text,
                "assignee_label": candidate.assignee_label,
            }));
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "task_candidates",
                    "task_candidate_id": task_candidate_id,
                }));
            Ok(ReviewInboxPort::create_with_evidence_in_transaction(
                transaction,
                &item,
                &[evidence],
            )
            .await?)
        }
    }
}
