use sqlx::postgres::{PgPool, PgRow, Postgres};
use sqlx::{Row, Transaction};

use super::errors::TaskCandidateError;
use super::models::{
    CandidatePayload, StoredCandidateRow, TaskCandidate, TaskCandidateReviewState,
};

pub(crate) async fn upsert_task_candidate(
    pool: &PgPool,
    payload: &CandidatePayload,
    task_candidate_id: String,
    review_state: TaskCandidateReviewState,
) -> Result<(), TaskCandidateError> {
    let update_result = sqlx::query(
        r#"
        UPDATE task_candidates
        SET
            source_kind = $2,
            source_id = $3,
            candidate_kind = $4,
            candidate_metadata = $5,
            project_id = COALESCE($6, project_id),
            title = $7,
            due_text = COALESCE($8, due_text),
            assignee_label = COALESCE($9, assignee_label),
            confidence = $10,
            review_state = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN review_state
                ELSE $11
            END,
            evidence_excerpt = $12,
            event_id = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN actor_id
                ELSE NULL
            END,
            reviewed_at = CASE
                WHEN review_state IN ('user_confirmed', 'user_rejected')
                    THEN reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        WHERE task_candidate_id = $1
           OR (source_kind = $2 AND source_id = $3 AND lower(title) = lower($7))
        "#,
    )
    .bind(&task_candidate_id)
    .bind(payload.source_kind.as_str())
    .bind(&payload.source_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.candidate_metadata)
    .bind(&payload.project_id)
    .bind(&payload.title)
    .bind(&payload.due_text)
    .bind(&payload.assignee_label)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .bind(&payload.evidence_excerpt)
    .execute(pool)
    .await?;

    if update_result.rows_affected() > 0 {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO task_candidates (
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
            review_state,
            evidence_excerpt,
            event_id,
            actor_id,
            reviewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NULL, NULL, NULL)
        ON CONFLICT (source_kind, source_id, lower(title))
        DO UPDATE SET
            source_kind = EXCLUDED.source_kind,
            source_id = EXCLUDED.source_id,
            candidate_kind = EXCLUDED.candidate_kind,
            candidate_metadata = EXCLUDED.candidate_metadata,
            project_id = COALESCE(EXCLUDED.project_id, task_candidates.project_id),
            title = EXCLUDED.title,
            due_text = COALESCE(EXCLUDED.due_text, task_candidates.due_text),
            assignee_label = COALESCE(EXCLUDED.assignee_label, task_candidates.assignee_label),
            confidence = EXCLUDED.confidence,
            review_state = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.review_state
                ELSE EXCLUDED.review_state
            END,
            evidence_excerpt = EXCLUDED.evidence_excerpt,
            event_id = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.event_id
                ELSE NULL
            END,
            actor_id = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.actor_id
                ELSE NULL
                END,
            reviewed_at = CASE
                WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                    THEN task_candidates.reviewed_at
                ELSE NULL
            END,
            updated_at = now()
        "#,
    )
    .bind(task_candidate_id)
    .bind(payload.source_kind.as_str())
    .bind(&payload.source_id)
    .bind(payload.candidate_kind.as_str())
    .bind(&payload.candidate_metadata)
    .bind(&payload.project_id)
    .bind(&payload.title)
    .bind(&payload.due_text)
    .bind(&payload.assignee_label)
    .bind(payload.confidence)
    .bind(review_state.as_str())
    .bind(&payload.evidence_excerpt)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn row_task_candidate(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
) -> Result<StoredCandidateRow, TaskCandidateError> {
    let row = sqlx::query(
        r#"
        SELECT
            source_kind,
            source_id,
            candidate_kind,
            candidate_metadata,
            project_id,
            title
        FROM task_candidates
        WHERE task_candidate_id = $1
        FOR UPDATE
        "#,
    )
    .bind(task_candidate_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(TaskCandidateError::TaskCandidateNotFound)?;

    Ok(StoredCandidateRow {
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        candidate_kind: row.try_get("candidate_kind")?,
        candidate_metadata: row.try_get("candidate_metadata")?,
        project_id: row.try_get("project_id")?,
        title: row.try_get("title")?,
    })
}

pub(crate) fn row_to_task_candidate(row: PgRow) -> Result<TaskCandidate, TaskCandidateError> {
    Ok(TaskCandidate {
        task_candidate_id: row.try_get("task_candidate_id")?,
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        project_id: row.try_get("project_id")?,
        title: row.try_get("title")?,
        due_text: row.try_get("due_text")?,
        assignee_label: row.try_get("assignee_label")?,
        confidence: row.try_get("confidence")?,
        review_state: row.try_get::<String, _>("review_state")?,
        evidence_excerpt: row.try_get("evidence_excerpt")?,
        generated_at: row.try_get("generated_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
