use super::*;

pub async fn sync_decision_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    decision: &Decision,
) -> Result<(), ReviewMirrorError> {
    let evidence_row = sqlx::query(
        r#"
        SELECT observation_id
        FROM decision_evidence
        WHERE decision_id = $1
          AND observation_id IS NOT NULL
        ORDER BY created_at ASC, evidence_id ASC
        LIMIT 1
        "#,
    )
    .bind(&decision.decision_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        ReviewInboxError::ReviewItemNotFound(format!("decision:{}", decision.decision_id))
    })?;
    let observation_id: String = evidence_row.try_get("observation_id")?;
    let review_item = ensure_decision_review_item_in_transaction(
        transaction,
        &decision.decision_id,
        &decision.title,
        &decision.rationale,
        decision.confidence,
        &observation_id,
    )
    .await?;

    sync_decision_review_item_status_in_transaction(transaction, decision, &review_item).await
}

pub(crate) async fn sync_decision_review_state_with_observation(
    pool: &sqlx::postgres::PgPool,
    decision: &Decision,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let review_item = ensure_decision_review_item_in_transaction(
        &mut transaction,
        &decision.decision_id,
        &decision.title,
        &decision.rationale,
        decision.confidence,
        observation_id,
    )
    .await?;
    sync_decision_review_item_status_in_transaction(&mut transaction, decision, &review_item)
        .await?;
    transaction.commit().await?;
    Ok(())
}

async fn sync_decision_review_item_status_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    decision: &Decision,
    review_item: &ReviewItem,
) -> Result<(), ReviewMirrorError> {
    match decision.review_state {
        DecisionReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        DecisionReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        DecisionReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new("decisions", "decision", &decision.decision_id),
            )
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn ensure_decision_review_item(
    pool: &sqlx::postgres::PgPool,
    decision_id: &str,
    title: &str,
    rationale: &str,
    confidence: f64,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let item = ensure_decision_review_item_in_transaction(
        &mut transaction,
        decision_id,
        title,
        rationale,
        confidence,
        observation_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(item)
}

pub(crate) async fn ensure_decision_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    decision_id: &str,
    title: &str,
    rationale: &str,
    confidence: f64,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    match ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::PotentialDecision,
        &json!({ "decision_id": decision_id }),
    )
    .await?
    {
        Some(item) => {
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "decisions",
                    "decision_id": decision_id,
                }));
            Ok(ReviewInboxPort::attach_evidence_in_transaction(
                transaction,
                &item.review_item_id,
                &[evidence],
            )
            .await?)
        }
        None => {
            let item = NewReviewItem::new(
                ReviewItemKind::PotentialDecision,
                title,
                rationale,
                confidence,
            )
            .metadata(json!({
                "mirrored_from": "decisions",
                "decision_id": decision_id,
            }));
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "decisions",
                    "decision_id": decision_id,
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
