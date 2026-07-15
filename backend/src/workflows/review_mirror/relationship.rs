use super::*;

pub async fn sync_relationship_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
) -> Result<(), ReviewMirrorError> {
    let evidence_row = sqlx::query(
        r#"
        SELECT observation_id, excerpt
        FROM relationship_evidence
        WHERE relationship_id = $1
          AND observation_id IS NOT NULL
        ORDER BY created_at ASC, evidence_id ASC
        LIMIT 1
        "#,
    )
    .bind(&relationship.relationship_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        ReviewInboxError::ReviewItemNotFound(format!(
            "relationship:{}",
            relationship.relationship_id
        ))
    })?;
    let observation_id: String = evidence_row.try_get("observation_id")?;
    let summary: Option<String> = evidence_row.try_get("excerpt")?;
    let review_item = ensure_relationship_review_item_in_transaction(
        transaction,
        RelationshipReviewInput {
            relationship_id: &relationship.relationship_id,
            relationship_type: &relationship.relationship_type,
            source_entity_kind: relationship.source_entity_kind.as_str(),
            source_entity_id: &relationship.source_entity_id,
            target_entity_kind: relationship.target_entity_kind.as_str(),
            target_entity_id: &relationship.target_entity_id,
            confidence: relationship.confidence,
            summary: summary.as_deref(),
            observation_id: &observation_id,
        },
    )
    .await?;

    match relationship.review_state {
        RelationshipReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        RelationshipReviewState::SystemAccepted => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Approved,
            )
            .await?;
        }
        RelationshipReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        RelationshipReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "relationships",
                    "relationship",
                    &relationship.relationship_id,
                ),
            )
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn sync_relationship_review_state_with_observation(
    pool: &sqlx::postgres::PgPool,
    relationship: &Relationship,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let review_item = ensure_relationship_review_item_in_transaction(
        &mut transaction,
        RelationshipReviewInput {
            relationship_id: &relationship.relationship_id,
            relationship_type: &relationship.relationship_type,
            source_entity_kind: relationship.source_entity_kind.as_str(),
            source_entity_id: &relationship.source_entity_id,
            target_entity_kind: relationship.target_entity_kind.as_str(),
            target_entity_id: &relationship.target_entity_id,
            confidence: relationship.confidence,
            summary: None,
            observation_id,
        },
    )
    .await?;

    match relationship.review_state {
        RelationshipReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        RelationshipReviewState::SystemAccepted => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Approved,
            )
            .await?;
        }
        RelationshipReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        RelationshipReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "relationships",
                    "relationship",
                    &relationship.relationship_id,
                ),
            )
            .await?;
        }
    }

    transaction.commit().await?;
    Ok(())
}

pub(crate) async fn ensure_relationship_review_item(
    pool: &sqlx::postgres::PgPool,
    input: RelationshipReviewInput<'_>,
) -> Result<ReviewItem, ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let item = ensure_relationship_review_item_in_transaction(&mut transaction, input).await?;
    transaction.commit().await?;
    Ok(item)
}

pub(crate) struct RelationshipReviewInput<'a> {
    pub(crate) relationship_id: &'a str,
    pub(crate) relationship_type: &'a str,
    pub(crate) source_entity_kind: &'a str,
    pub(crate) source_entity_id: &'a str,
    pub(crate) target_entity_kind: &'a str,
    pub(crate) target_entity_id: &'a str,
    pub(crate) confidence: f64,
    pub(crate) summary: Option<&'a str>,
    pub(crate) observation_id: &'a str,
}

pub(crate) async fn ensure_relationship_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    input: RelationshipReviewInput<'_>,
) -> Result<ReviewItem, ReviewMirrorError> {
    let RelationshipReviewInput {
        relationship_id,
        relationship_type,
        source_entity_kind,
        source_entity_id,
        target_entity_kind,
        target_entity_id,
        confidence,
        summary,
        observation_id,
    } = input;
    match ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::PotentialRelationship,
        &json!({ "relationship_id": relationship_id }),
    )
    .await?
    {
        Some(item) => {
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "relationships",
                    "relationship_id": relationship_id,
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
                ReviewItemKind::PotentialRelationship,
                relationship_type,
                summary.unwrap_or("Evidence-backed relationship candidate"),
                confidence,
            )
            .metadata(json!({
                "mirrored_from": "relationships",
                "relationship_id": relationship_id,
                "relationship_type": relationship_type,
                "source_entity_kind": source_entity_kind,
                "source_entity_id": source_entity_id,
                "target_entity_kind": target_entity_kind,
                "target_entity_id": target_entity_id,
            }));
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "relationships",
                    "relationship_id": relationship_id,
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
