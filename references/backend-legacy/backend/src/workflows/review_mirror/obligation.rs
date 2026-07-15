use super::*;

pub async fn sync_obligation_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &Obligation,
) -> Result<(), ReviewMirrorError> {
    let evidence_row = sqlx::query(
        r#"
        SELECT observation_id, quote
        FROM obligation_evidence
        WHERE obligation_id = $1
          AND observation_id IS NOT NULL
        ORDER BY created_at ASC, evidence_id ASC
        LIMIT 1
        "#,
    )
    .bind(&obligation.obligation_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        ReviewInboxError::ReviewItemNotFound(format!("obligation:{}", obligation.obligation_id))
    })?;
    let observation_id: String = evidence_row.try_get("observation_id")?;
    let summary: Option<String> = evidence_row.try_get("quote")?;
    let review_item = ensure_obligation_review_item_in_transaction(
        transaction,
        &obligation.obligation_id,
        &obligation.statement,
        summary.as_deref(),
        obligation.confidence,
        &observation_id,
    )
    .await?;

    sync_obligation_review_item_status_in_transaction(transaction, obligation, &review_item).await
}

pub(crate) async fn sync_obligation_review_state_with_observation(
    pool: &sqlx::postgres::PgPool,
    obligation: &Obligation,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let review_item = ensure_obligation_review_item_in_transaction(
        &mut transaction,
        &obligation.obligation_id,
        &obligation.statement,
        None,
        obligation.confidence,
        observation_id,
    )
    .await?;
    sync_obligation_review_item_status_in_transaction(&mut transaction, obligation, &review_item)
        .await?;
    transaction.commit().await?;
    Ok(())
}

async fn sync_obligation_review_item_status_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &Obligation,
    review_item: &ReviewItem,
) -> Result<(), ReviewMirrorError> {
    match obligation.review_state {
        ObligationReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        ObligationReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        ObligationReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new("obligations", "obligation", &obligation.obligation_id),
            )
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn ensure_obligation_review_item(
    pool: &sqlx::postgres::PgPool,
    obligation_id: &str,
    statement: &str,
    summary: Option<&str>,
    confidence: f64,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let item = ensure_obligation_review_item_in_transaction(
        &mut transaction,
        obligation_id,
        statement,
        summary,
        confidence,
        observation_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(item)
}

pub(crate) async fn ensure_obligation_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation_id: &str,
    statement: &str,
    summary: Option<&str>,
    confidence: f64,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    match ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::PotentialObligation,
        &json!({ "obligation_id": obligation_id }),
    )
    .await?
    {
        Some(item) => {
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "obligations",
                    "obligation_id": obligation_id,
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
                ReviewItemKind::PotentialObligation,
                statement,
                summary.unwrap_or(statement),
                confidence,
            )
            .metadata(json!({
                "mirrored_from": "obligations",
                "obligation_id": obligation_id,
            }));
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "obligations",
                    "obligation_id": obligation_id,
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
