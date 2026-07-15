use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn ensure_project_link_candidate_review_item(
    pool: &sqlx::postgres::PgPool,
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    title: &str,
    summary: &str,
    confidence: f64,
    observation_id: &str,
    graph_node_id: Option<&str>,
) -> Result<(), ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let _ = ensure_project_link_candidate_review_item_in_transaction(
        &mut transaction,
        ProjectLinkReviewCandidateInput {
            project_id,
            target_kind,
            target_id,
            title,
            summary,
            confidence,
            observation_id,
            graph_node_id,
        },
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

struct ProjectLinkReviewCandidateInput<'a> {
    project_id: &'a str,
    target_kind: ProjectLinkTargetKind,
    target_id: &'a str,
    title: &'a str,
    summary: &'a str,
    confidence: f64,
    observation_id: &'a str,
    graph_node_id: Option<&'a str>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn sync_project_link_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    review_state: ProjectLinkReviewState,
    title: &str,
    summary: &str,
    confidence: f64,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    let review_item = ensure_project_link_candidate_review_item_in_transaction(
        transaction,
        ProjectLinkReviewCandidateInput {
            project_id,
            target_kind,
            target_id,
            title,
            summary,
            confidence,
            observation_id,
            graph_node_id: None,
        },
    )
    .await?;

    match review_state {
        ProjectLinkReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        ProjectLinkReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        ProjectLinkReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "projects",
                    "project_link_candidate",
                    format!("{project_id}:{}:{target_id}", target_kind.as_str()),
                ),
            )
            .await?;
        }
    }

    Ok(())
}

async fn ensure_project_link_candidate_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    input: ProjectLinkReviewCandidateInput<'_>,
) -> Result<ReviewItem, ReviewMirrorError> {
    let ProjectLinkReviewCandidateInput {
        project_id,
        target_kind,
        target_id,
        title,
        summary,
        confidence,
        observation_id,
        graph_node_id,
    } = input;
    match ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::ProjectLinkCandidate,
        &json!({
            "project_id": project_id,
            "target_kind": target_kind.as_str(),
            "target_id": target_id,
        }),
    )
    .await?
    {
        Some(item) => {
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "project_link_candidates",
                    "project_id": project_id,
                    "target_kind": target_kind.as_str(),
                    "target_id": target_id,
                }));
            Ok(ReviewInboxPort::attach_evidence_in_transaction(
                transaction,
                &item.review_item_id,
                &[evidence],
            )
            .await?)
        }
        None => {
            let mut metadata = json!({
                "mirrored_from": "project_link_candidates",
                "project_id": project_id,
                "target_kind": target_kind.as_str(),
                "target_id": target_id,
            });
            if let Some(graph_node_id) = graph_node_id {
                metadata["graph_node_id"] = json!(graph_node_id);
            }
            let item = NewReviewItem::new(
                ReviewItemKind::ProjectLinkCandidate,
                title,
                summary,
                confidence,
            )
            .metadata(metadata);
            let evidence = NewReviewItemEvidence::new(observation_id.to_owned())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "project_link_candidates",
                    "project_id": project_id,
                    "target_kind": target_kind.as_str(),
                    "target_id": target_id,
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
