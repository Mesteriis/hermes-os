use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::decisions::DecisionReviewPort;
use crate::domains::obligations::ObligationReviewPort;
use crate::domains::persons::identity::{
    PersonIdentityCandidatePayload, PersonIdentityError, load_identity_candidate_payload,
    parse_person_identity_candidate_kind, parse_person_identity_review_state,
    person_identity_candidate_detected_event_type,
};
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxError, ReviewInboxPort, ReviewItemKind,
};
use crate::domains::tasks::candidates::TaskCandidatePort;
use crate::platform::events::{EventStoreError, StoredEventEnvelope};
use crate::workflows::email_intelligence::{
    EmailIntelligenceService, EmailKnowledgeCandidate, EmailSummaryContract,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, ensure_decision_review_item, ensure_obligation_review_item,
    ensure_relationship_review_item, ensure_task_candidate_review_item,
    sync_identity_candidate_review_state_in_transaction, sync_identity_candidate_to_review,
};

#[derive(Debug, Error)]
pub enum ReviewInboxWorkflowError {
    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Decision(#[from] crate::domains::decisions::DecisionReviewPortError),

    #[error(transparent)]
    Obligation(#[from] crate::domains::obligations::ObligationReviewPortError),

    #[error(transparent)]
    Relationship(#[from] crate::domains::relationships::RelationshipReviewPortError),

    #[error(transparent)]
    TaskCandidate(#[from] crate::domains::tasks::candidates::TaskCandidateError),

    #[error(transparent)]
    PersonIdentity(#[from] PersonIdentityError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn refresh_message_task_candidates_into_review(
    pool: &PgPool,
    message_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if message_ids.is_empty() {
        return Ok(0);
    }

    let refreshed = TaskCandidatePort::new(pool.clone())
        .refresh_message_candidates_for_ids(message_ids)
        .await?;
    let observation_ids = load_message_observation_ids(pool, message_ids).await?;
    let _ = sync_task_candidates_to_review_for_observations(pool, &observation_ids).await?;
    Ok(refreshed)
}

pub async fn refresh_message_decisions_into_review(
    pool: &PgPool,
    message_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if message_ids.is_empty() {
        return Ok(0);
    }

    let refreshed = DecisionReviewPort::new(pool.clone())
        .refresh_message_candidates_for_ids(message_ids)
        .await?;
    let observation_ids = load_message_observation_ids(pool, message_ids).await?;
    let _ = sync_decisions_to_review_for_observations(pool, &observation_ids).await?;
    Ok(refreshed)
}

pub async fn refresh_message_knowledge_candidates_into_review(
    pool: &PgPool,
    messages: &[ProjectedMessage],
) -> Result<usize, ReviewInboxWorkflowError> {
    if messages.is_empty() {
        return Ok(0);
    }

    let review_store = ReviewInboxPort::new(pool.clone());
    let mut mirrored = 0;
    for message in messages {
        let summary_contract = message_summary_contract(message);
        for (candidate_group, candidate) in knowledge_candidates(&summary_contract) {
            let summary = if candidate.evidence.trim().is_empty() {
                format!("Source-backed {candidate_group} candidate from communication evidence")
            } else {
                candidate.evidence.clone()
            };
            let item = NewReviewItem::new(
                ReviewItemKind::KnowledgeCandidate,
                candidate.title.clone(),
                summary,
                knowledge_candidate_confidence(candidate_group),
            )
            .metadata(json!({
                "mirrored_from": "message_summary_contract",
                "message_id": message.message_id,
                "observation_id": message.observation_id,
                "candidate_group": candidate_group,
                "candidate_title": candidate.title,
            }));
            let evidence = NewReviewItemEvidence::new(message.observation_id.clone())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "message_summary_contract",
                    "message_id": message.message_id,
                    "candidate_group": candidate_group,
                }));
            let _ = review_store
                .create_with_evidence(&item, &[evidence])
                .await?;
            mirrored += 1;
        }
    }

    Ok(mirrored)
}

pub async fn refresh_message_people_candidates_into_review(
    pool: &PgPool,
    messages: &[ProjectedMessage],
) -> Result<usize, ReviewInboxWorkflowError> {
    if messages.is_empty() {
        return Ok(0);
    }

    let review_store = ReviewInboxPort::new(pool.clone());
    let mut mirrored = 0;
    for message in messages {
        let summary_contract = message_summary_contract(message);
        for candidate in &summary_contract.persona_candidates {
            let summary = if candidate.evidence.trim().is_empty() {
                "Source-backed persona candidate from communication evidence".to_owned()
            } else {
                candidate.evidence.clone()
            };
            let item = NewReviewItem::new(
                ReviewItemKind::NewPerson,
                candidate.title.clone(),
                summary,
                0.68,
            )
            .metadata(json!({
                "mirrored_from": "message_summary_contract",
                "message_id": message.message_id,
                "observation_id": message.observation_id,
                "candidate_group": "persona",
                "candidate_title": candidate.title,
            }));
            let evidence = NewReviewItemEvidence::new(message.observation_id.clone())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "message_summary_contract",
                    "message_id": message.message_id,
                    "candidate_group": "persona",
                }));
            let _ = review_store
                .create_with_evidence(&item, &[evidence])
                .await?;
            mirrored += 1;
        }

        for candidate in &summary_contract.organization_candidates {
            let summary = if candidate.evidence.trim().is_empty() {
                "Source-backed organization candidate from communication evidence".to_owned()
            } else {
                candidate.evidence.clone()
            };
            let item = NewReviewItem::new(
                ReviewItemKind::NewOrganization,
                candidate.title.clone(),
                summary,
                0.7,
            )
            .metadata(json!({
                "mirrored_from": "message_summary_contract",
                "message_id": message.message_id,
                "observation_id": message.observation_id,
                "candidate_group": "organization",
                "candidate_title": candidate.title,
            }));
            let evidence = NewReviewItemEvidence::new(message.observation_id.clone())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "message_summary_contract",
                    "message_id": message.message_id,
                    "candidate_group": "organization",
                }));
            let _ = review_store
                .create_with_evidence(&item, &[evidence])
                .await?;
            mirrored += 1;
        }
    }

    Ok(mirrored)
}

pub const PERSON_IDENTITY_REVIEW_INBOX_CONSUMER: &str = "person_identity_review_inbox";

pub async fn project_person_identity_review_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_person_identity_review_event_inner(&pool, event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

async fn project_person_identity_review_event_inner(
    pool: &PgPool,
    event: StoredEventEnvelope,
) -> Result<(), ReviewInboxWorkflowError> {
    if event.event.event_type == person_identity_candidate_detected_event_type() {
        let payload = identity_candidate_payload_from_event(&event)?;
        sync_identity_candidate_to_review(pool, &payload).await?;
        return Ok(());
    }

    if event.event.event_type == "person_identity.review_state_changed" {
        let identity_candidate_id =
            required_event_string(&event.event.payload, "identity_candidate_id")?;
        let review_state = parse_person_identity_review_state(required_event_string(
            &event.event.payload,
            "review_state",
        )?)?;
        let mut transaction = pool.begin().await?;
        let payload =
            load_identity_candidate_payload(&mut transaction, identity_candidate_id).await?;
        sync_identity_candidate_review_state_in_transaction(
            &mut transaction,
            identity_candidate_id,
            review_state,
            &payload,
        )
        .await?;
        transaction.commit().await?;
    }

    Ok(())
}

fn identity_candidate_payload_from_event(
    event: &StoredEventEnvelope,
) -> Result<PersonIdentityCandidatePayload, PersonIdentityError> {
    let payload = &event.event.payload;
    Ok(PersonIdentityCandidatePayload {
        candidate_kind: parse_person_identity_candidate_kind(required_event_string(
            payload,
            "candidate_kind",
        )?)?,
        left_person_id: required_event_string(payload, "left_person_id")?.to_owned(),
        right_person_id: payload
            .get("right_person_id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        email_address: payload
            .get("email_address")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        evidence_summary: required_event_string(payload, "evidence_summary")?.to_owned(),
        confidence: payload
            .get("confidence")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| PersonIdentityError::MissingPayloadField("confidence".to_owned()))?,
    })
}

fn required_event_string<'a>(
    payload: &'a serde_json::Value,
    field: &str,
) -> Result<&'a str, PersonIdentityError> {
    payload
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| PersonIdentityError::MissingPayloadField(field.to_owned()))
}

pub async fn sync_task_candidates_to_review_for_observations(
    pool: &PgPool,
    observation_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if observation_ids.is_empty() {
        return Ok(0);
    }

    let rows = sqlx::query(
        r#"
        SELECT
            task_candidate_id,
            source_kind,
            source_id,
            candidate_kind,
            candidate_metadata,
            project_id,
            title,
            due_text,
            assignee_label,
            confidence,
            evidence_excerpt,
            observation_id
        FROM task_candidates
        WHERE source_kind = 'observation'
          AND review_state = 'suggested'
          AND observation_id = ANY($1)
        ORDER BY updated_at DESC, task_candidate_id
        "#,
    )
    .bind(observation_ids.to_vec())
    .fetch_all(pool)
    .await?;
    let row_count = rows.len();

    for row in rows {
        let task_candidate_id: String = row.try_get("task_candidate_id")?;
        let observation_id: String = row.try_get("observation_id")?;
        let candidate = crate::domains::tasks::candidates::StoredCandidateRow {
            source_kind: row.try_get("source_kind")?,
            source_id: row.try_get("source_id")?,
            observation_id: Some(observation_id.clone()),
            candidate_kind: row.try_get("candidate_kind")?,
            candidate_metadata: row.try_get("candidate_metadata")?,
            project_id: row.try_get("project_id")?,
            title: row.try_get("title")?,
            due_text: row.try_get("due_text")?,
            assignee_label: row.try_get("assignee_label")?,
            confidence: row.try_get("confidence")?,
            evidence_excerpt: row.try_get("evidence_excerpt")?,
        };
        let _ = ensure_task_candidate_review_item(
            pool,
            &task_candidate_id,
            &candidate,
            &observation_id,
        )
        .await?;
    }

    Ok(row_count)
}

pub async fn sync_ai_run_task_candidates_to_review(
    pool: &PgPool,
    run_id: &str,
) -> Result<usize, ReviewInboxWorkflowError> {
    let observation_ids = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT observation_id
        FROM task_candidates
        WHERE agent_run_id = $1
          AND observation_id IS NOT NULL
        ORDER BY observation_id
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;

    sync_task_candidates_to_review_for_observations(pool, &observation_ids).await
}

pub async fn sync_decisions_to_review_for_observations(
    pool: &PgPool,
    observation_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if observation_ids.is_empty() {
        return Ok(0);
    }

    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            decision.decision_id,
            decision.title,
            decision.rationale,
            decision.confidence::float8 AS confidence,
            decision_evidence.observation_id
        FROM decisions decision
        JOIN decision_evidence
          ON decision_evidence.decision_id = decision.decision_id
        WHERE decision.review_state = 'suggested'
          AND decision_evidence.observation_id = ANY($1)
        ORDER BY decision.decision_id
        "#,
    )
    .bind(observation_ids.to_vec())
    .fetch_all(pool)
    .await?;
    let row_count = rows.len();

    for row in rows {
        let decision_id: String = row.try_get("decision_id")?;
        let title: String = row.try_get("title")?;
        let rationale: String = row.try_get("rationale")?;
        let confidence: f64 = row.try_get("confidence")?;
        let observation_id: String = row.try_get("observation_id")?;
        let _ = ensure_decision_review_item(
            pool,
            &decision_id,
            &title,
            &rationale,
            confidence,
            &observation_id,
        )
        .await?;
    }

    Ok(row_count)
}

pub async fn sync_obligations_to_review_for_observations(
    pool: &PgPool,
    observation_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if observation_ids.is_empty() {
        return Ok(0);
    }

    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            obligation.obligation_id,
            obligation.statement,
            obligation.confidence::float8 AS confidence,
            obligation_evidence.quote,
            obligation_evidence.observation_id
        FROM obligations obligation
        JOIN obligation_evidence
          ON obligation_evidence.obligation_id = obligation.obligation_id
        WHERE obligation.review_state = 'suggested'
          AND obligation_evidence.observation_id = ANY($1)
        ORDER BY obligation.obligation_id
        "#,
    )
    .bind(observation_ids.to_vec())
    .fetch_all(pool)
    .await?;
    let row_count = rows.len();

    for row in rows {
        let obligation_id: String = row.try_get("obligation_id")?;
        let statement: String = row.try_get("statement")?;
        let confidence: f64 = row.try_get("confidence")?;
        let summary: Option<String> = row.try_get("quote")?;
        let observation_id: String = row.try_get("observation_id")?;
        let _ = ensure_obligation_review_item(
            pool,
            &obligation_id,
            &statement,
            summary.as_deref(),
            confidence,
            &observation_id,
        )
        .await?;
    }

    Ok(row_count)
}

pub async fn sync_relationships_to_review_for_observations(
    pool: &PgPool,
    observation_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if observation_ids.is_empty() {
        return Ok(0);
    }

    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            relationship.relationship_id,
            relationship.relationship_type,
            relationship.source_entity_kind,
            relationship.source_entity_id,
            relationship.target_entity_kind,
            relationship.target_entity_id,
            relationship.confidence::float8 AS confidence,
            relationship_evidence.excerpt,
            relationship_evidence.observation_id
        FROM relationships relationship
        JOIN relationship_evidence
          ON relationship_evidence.relationship_id = relationship.relationship_id
        WHERE relationship.review_state = 'suggested'
          AND relationship_evidence.observation_id = ANY($1)
        ORDER BY relationship.relationship_id
        "#,
    )
    .bind(observation_ids.to_vec())
    .fetch_all(pool)
    .await?;
    let row_count = rows.len();

    for row in rows {
        let relationship_id: String = row.try_get("relationship_id")?;
        let relationship_type: String = row.try_get("relationship_type")?;
        let source_entity_kind: String = row.try_get("source_entity_kind")?;
        let source_entity_id: String = row.try_get("source_entity_id")?;
        let target_entity_kind: String = row.try_get("target_entity_kind")?;
        let target_entity_id: String = row.try_get("target_entity_id")?;
        let confidence: f64 = row.try_get("confidence")?;
        let summary: Option<String> = row.try_get("excerpt")?;
        let observation_id: String = row.try_get("observation_id")?;
        let _ = ensure_relationship_review_item(
            pool,
            &relationship_id,
            &relationship_type,
            &source_entity_kind,
            &source_entity_id,
            &target_entity_kind,
            &target_entity_id,
            confidence,
            summary.as_deref(),
            &observation_id,
        )
        .await?;
    }

    Ok(row_count)
}

async fn load_message_observation_ids(
    pool: &PgPool,
    message_ids: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT observation_id
        FROM communication_messages
        WHERE message_id = ANY($1)
          AND observation_id IS NOT NULL
        ORDER BY observation_id
        "#,
    )
    .bind(message_ids.to_vec())
    .fetch_all(pool)
    .await
}

fn message_summary_contract(message: &ProjectedMessage) -> EmailSummaryContract {
    message
        .message_metadata
        .get("ai_summary_contract")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_else(|| EmailIntelligenceService::heuristic_structured_summary(message))
}

fn knowledge_candidates(
    summary_contract: &EmailSummaryContract,
) -> Vec<(&'static str, &EmailKnowledgeCandidate)> {
    let mut candidates = Vec::with_capacity(
        summary_contract.document_candidates.len() + summary_contract.agreement_candidates.len(),
    );
    candidates.extend(
        summary_contract
            .document_candidates
            .iter()
            .map(|candidate| ("document", candidate)),
    );
    candidates.extend(
        summary_contract
            .agreement_candidates
            .iter()
            .map(|candidate| ("agreement", candidate)),
    );
    candidates
}

fn knowledge_candidate_confidence(candidate_group: &str) -> f64 {
    match candidate_group {
        "agreement" => 0.79,
        _ => 0.72,
    }
}
