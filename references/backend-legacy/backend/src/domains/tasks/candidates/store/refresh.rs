use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::engines::obligation::{
    engine::ObligationEngine,
    models::{ObligationEntityKind, ObligationExtractionInput},
};

use super::super::constants::OWNER_PERSONA_EXTRACTION_CONTEXT_ID;
use super::super::errors::TaskCandidateError;
use super::super::extraction::{
    evidence_excerpt, extract_candidate_fragment, task_candidate_payload_from_obligation,
    title_from_fragment,
};
use super::super::models::{
    CandidatePayload, TaskCandidateKind, TaskCandidateReviewState, TaskCandidateSourceKind,
};
use super::super::persistence::upsert_task_candidate;
use super::super::validation::validate_limit;

pub(super) async fn refresh_deterministic_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, TaskCandidateError> {
    let limit = validate_limit(limit)?;

    let message_count = refresh_message_candidates(pool, limit).await?;
    let document_count = refresh_document_candidates(pool, limit).await?;

    Ok(message_count + document_count)
}

async fn refresh_message_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, TaskCandidateError> {
    let rows = sqlx::query(
        r#"
        SELECT
            message_id,
            observation_id,
            subject,
            body_text
        FROM communication_messages
        ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let source_id = row.try_get::<String, _>("message_id")?;
        let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
        let source_text = format!(
            "{}\n{}",
            row.try_get::<String, _>("subject")?,
            row.try_get::<String, _>("body_text")?,
        );

        count +=
            refresh_message_candidate_from_text(pool, &source_id, &observation_id, &source_text)
                .await?;
    }

    Ok(count)
}

pub(super) async fn refresh_message_candidates_for_ids(
    pool: &PgPool,
    message_ids: &[String],
) -> Result<usize, TaskCandidateError> {
    if message_ids.is_empty() {
        return Ok(0);
    }

    let rows = sqlx::query(
        r#"
        SELECT
            message_id,
            observation_id,
            subject,
            body_text
        FROM communication_messages
        WHERE message_id = ANY($1)
        ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
        "#,
    )
    .bind(message_ids.to_vec())
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let source_id = row.try_get::<String, _>("message_id")?;
        let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
        let source_text = format!(
            "{}\n{}",
            row.try_get::<String, _>("subject")?,
            row.try_get::<String, _>("body_text")?,
        );
        count +=
            refresh_message_candidate_from_text(pool, &source_id, &observation_id, &source_text)
                .await?;
    }

    Ok(count)
}

async fn refresh_message_candidate_from_text(
    pool: &PgPool,
    source_id: &str,
    observation_id: &Option<String>,
    source_text: &str,
) -> Result<usize, TaskCandidateError> {
    let observation_id = observation_id
        .clone()
        .ok_or_else(|| TaskCandidateError::ObservationRequired(source_id.to_owned()))?;

    if let Some(fragment) = extract_candidate_fragment(source_text) {
        let payload = CandidatePayload {
            source_kind: TaskCandidateSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id.clone()),
            candidate_kind: TaskCandidateKind::Task,
            candidate_metadata: json!({}),
            project_id: None,
            title: title_from_fragment(&fragment.text),
            due_text: fragment.due_text,
            assignee_label: fragment.assignee_label,
            confidence: 0.8,
            evidence_excerpt: evidence_excerpt(&fragment.text),
        };
        upsert_suggested_candidate(pool, &payload).await?;
        return Ok(1);
    }

    let input = ObligationExtractionInput::communication(
        source_id,
        source_text,
        ObligationEntityKind::Persona,
        OWNER_PERSONA_EXTRACTION_CONTEXT_ID,
    );
    let extraction = ObligationEngine::detect_candidates(&input)?;

    let mut count = 0usize;
    for obligation_candidate in extraction.obligations {
        let payload =
            task_candidate_payload_from_obligation(observation_id.clone(), &obligation_candidate);
        upsert_suggested_candidate(pool, &payload).await?;
        count += 1;
    }

    Ok(count)
}

async fn refresh_document_candidates(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, TaskCandidateError> {
    let rows = sqlx::query(
        r#"
            SELECT
                document_id,
                observation_id,
                title,
                extracted_text
            FROM documents
            ORDER BY imported_at DESC, document_id
            LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut count = 0usize;
    for row in rows {
        let source_id = row.try_get::<String, _>("document_id")?;
        let observation_id = row.try_get::<Option<String>, _>("observation_id")?;
        let source_text = format!(
            "{}\n{}",
            row.try_get::<String, _>("title")?,
            row.try_get::<String, _>("extracted_text")?,
        );

        count +=
            refresh_document_candidate_from_text(pool, &source_id, observation_id, &source_text)
                .await?;
    }

    Ok(count)
}

async fn refresh_document_candidate_from_text(
    pool: &PgPool,
    source_id: &str,
    observation_id: Option<String>,
    source_text: &str,
) -> Result<usize, TaskCandidateError> {
    let observation_id = observation_id
        .ok_or_else(|| TaskCandidateError::ObservationRequired(source_id.to_owned()))?;

    if let Some(fragment) = extract_candidate_fragment(source_text) {
        let payload = CandidatePayload {
            source_kind: TaskCandidateSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id.clone()),
            candidate_kind: TaskCandidateKind::Task,
            candidate_metadata: json!({}),
            project_id: None,
            title: title_from_fragment(&fragment.text),
            due_text: fragment.due_text,
            assignee_label: fragment.assignee_label,
            confidence: 0.7,
            evidence_excerpt: evidence_excerpt(&fragment.text),
        };
        upsert_suggested_candidate(pool, &payload).await?;
        return Ok(1);
    }

    let input = ObligationExtractionInput::document(
        source_id,
        source_text,
        ObligationEntityKind::Persona,
        OWNER_PERSONA_EXTRACTION_CONTEXT_ID,
    );
    let extraction = ObligationEngine::detect_candidates(&input)?;

    let mut count = 0usize;
    for obligation_candidate in extraction.obligations {
        let payload =
            task_candidate_payload_from_obligation(observation_id.clone(), &obligation_candidate);
        upsert_suggested_candidate(pool, &payload).await?;
        count += 1;
    }

    Ok(count)
}

async fn upsert_suggested_candidate(
    pool: &PgPool,
    payload: &CandidatePayload,
) -> Result<(), TaskCandidateError> {
    upsert_task_candidate(
        pool,
        payload,
        payload.task_candidate_id(),
        TaskCandidateReviewState::Suggested,
    )
    .await
}
