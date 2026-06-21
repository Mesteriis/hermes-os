use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::decisions::{Decision, DecisionReviewState};
use crate::domains::obligations::{Obligation, ObligationReviewState};
use crate::domains::persons::identity::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};
use crate::domains::projects::link_reviews::{ProjectLinkReviewState, ProjectLinkTargetKind};
use crate::domains::relationships::{Relationship, RelationshipReviewState};
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxError, ReviewInboxPort, ReviewItem,
    ReviewItemKind, ReviewItemStatus, ReviewPromotionTarget,
};
use crate::domains::tasks::candidates::{
    StoredCandidateRow, TaskCandidateReviewState, task_id_from_candidate,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationPortError,
};

#[derive(Debug, Error)]
pub enum ReviewMirrorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Observation(#[from] ObservationPortError),

    #[error("review-backed observation is required: {0}")]
    ObservationRequired(String),
}

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
        &relationship.relationship_id,
        &relationship.relationship_type,
        relationship.source_entity_kind.as_str(),
        &relationship.source_entity_id,
        relationship.target_entity_kind.as_str(),
        &relationship.target_entity_id,
        relationship.confidence,
        summary.as_deref(),
        &observation_id,
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

pub(crate) async fn sync_identity_candidate_to_review(
    pool: &sqlx::postgres::PgPool,
    payload: &PersonIdentityCandidatePayload,
) -> Result<(), ReviewMirrorError> {
    let observation = ObservationPort::new(pool.clone())
        .capture(&identity_candidate_observation(payload))
        .await?;
    let _ =
        ensure_identity_candidate_review_item(pool, payload, &observation.observation_id).await?;
    Ok(())
}

pub(crate) async fn sync_identity_candidate_to_review_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonIdentityCandidatePayload,
) -> Result<(), ReviewMirrorError> {
    let observation = ObservationPort::capture_in_transaction(
        transaction,
        &identity_candidate_observation(payload),
    )
    .await?;
    let _ = ensure_identity_candidate_review_item_in_transaction(
        transaction,
        payload,
        &observation.observation_id,
    )
    .await?;
    Ok(())
}

pub(crate) async fn sync_identity_candidate_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    identity_candidate_id: &str,
    review_state: PersonIdentityReviewState,
    payload: &PersonIdentityCandidatePayload,
) -> Result<(), ReviewMirrorError> {
    let observation = ObservationPort::capture_in_transaction(
        transaction,
        &identity_candidate_observation(payload),
    )
    .await?;
    let review_item = ensure_identity_candidate_review_item_in_transaction(
        transaction,
        payload,
        &observation.observation_id,
    )
    .await?;

    match review_state {
        PersonIdentityReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        PersonIdentityReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        PersonIdentityReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new("persons", "identity_candidate", identity_candidate_id),
            )
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn ensure_identity_candidate_review_item(
    pool: &sqlx::postgres::PgPool,
    payload: &PersonIdentityCandidatePayload,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let item = ensure_identity_candidate_review_item_in_transaction(
        &mut transaction,
        payload,
        observation_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(item)
}

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

pub(crate) async fn ensure_identity_candidate_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &PersonIdentityCandidatePayload,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let identity_candidate_id = payload.identity_candidate_id();
    let evidence = identity_candidate_review_evidence(&identity_candidate_id, observation_id);

    match ReviewInboxPort::find_latest_by_kind_and_metadata_in_transaction(
        transaction,
        ReviewItemKind::IdentityCandidate,
        &json!({
            "identity_candidate_id": identity_candidate_id,
        }),
    )
    .await?
    {
        Some(item) => Ok(ReviewInboxPort::attach_evidence_in_transaction(
            transaction,
            &item.review_item_id,
            &[evidence],
        )
        .await?),
        None => {
            let item = identity_candidate_review_item(payload);
            Ok(ReviewInboxPort::create_with_evidence_in_transaction(
                transaction,
                &item,
                &[evidence],
            )
            .await?)
        }
    }
}

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
        project_id,
        target_kind,
        target_id,
        title,
        summary,
        confidence,
        observation_id,
        graph_node_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
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
        project_id,
        target_kind,
        target_id,
        title,
        summary,
        confidence,
        observation_id,
        None,
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

#[allow(clippy::too_many_arguments)]
async fn ensure_project_link_candidate_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    title: &str,
    summary: &str,
    confidence: f64,
    observation_id: &str,
    graph_node_id: Option<&str>,
) -> Result<ReviewItem, ReviewMirrorError> {
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn ensure_relationship_review_item(
    pool: &sqlx::postgres::PgPool,
    relationship_id: &str,
    relationship_type: &str,
    source_entity_kind: &str,
    source_entity_id: &str,
    target_entity_kind: &str,
    target_entity_id: &str,
    confidence: f64,
    summary: Option<&str>,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let item = ensure_relationship_review_item_in_transaction(
        &mut transaction,
        relationship_id,
        relationship_type,
        source_entity_kind,
        source_entity_id,
        target_entity_kind,
        target_entity_id,
        confidence,
        summary,
        observation_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(item)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn ensure_relationship_review_item_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship_id: &str,
    relationship_type: &str,
    source_entity_kind: &str,
    source_entity_id: &str,
    target_entity_kind: &str,
    target_entity_id: &str,
    confidence: f64,
    summary: Option<&str>,
    observation_id: &str,
) -> Result<ReviewItem, ReviewMirrorError> {
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

fn identity_candidate_observation(payload: &PersonIdentityCandidatePayload) -> NewObservation {
    let identity_candidate_id = payload.identity_candidate_id();
    NewObservation::new(
        "PERSON_IDENTITY_CANDIDATE",
        ObservationOriginKind::LocalRuntime,
        chrono::Utc::now(),
        json!({
            "identity_candidate_id": identity_candidate_id,
            "candidate_kind": payload.candidate_kind.as_str(),
            "left_person_id": payload.left_person_id,
            "right_person_id": payload.right_person_id,
            "email_address": payload.email_address,
            "evidence_summary": payload.evidence_summary,
            "confidence": payload.confidence,
        }),
        format!("identity-candidate://{identity_candidate_id}"),
    )
    .confidence(payload.confidence)
    .provenance(json!({
        "pipeline": "person_identity_candidates",
        "candidate_kind": payload.candidate_kind.as_str(),
    }))
}

fn identity_candidate_review_item(payload: &PersonIdentityCandidatePayload) -> NewReviewItem {
    NewReviewItem::new(
        ReviewItemKind::IdentityCandidate,
        payload.candidate_kind.as_str(),
        payload.evidence_summary.clone(),
        payload.confidence,
    )
    .metadata(json!({
        "mirrored_from": "identity_candidates",
        "identity_candidate_id": payload.identity_candidate_id(),
        "candidate_kind": payload.candidate_kind.as_str(),
        "left_person_id": payload.left_person_id,
        "right_person_id": payload.right_person_id,
        "email_address": payload.email_address,
    }))
}

fn identity_candidate_review_evidence(
    identity_candidate_id: &str,
    observation_id: &str,
) -> NewReviewItemEvidence {
    NewReviewItemEvidence::new(observation_id.to_owned())
        .role("primary")
        .metadata(json!({
            "mirrored_from": "identity_candidates",
            "identity_candidate_id": identity_candidate_id,
        }))
}
