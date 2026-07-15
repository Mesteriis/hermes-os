use sqlx::postgres::PgPool;

use super::super::errors::TaskCandidateError;
use super::super::models::TaskCandidate;
use super::super::persistence::row_to_task_candidate;
use super::super::validation::validate_optional_limit;

pub(super) async fn list_candidates(
    pool: &PgPool,
    limit: Option<i64>,
) -> Result<Vec<TaskCandidate>, TaskCandidateError> {
    let limit = validate_optional_limit(limit)?;

    let rows = sqlx::query(
        r#"
        SELECT
            task_candidate_id,
            source_kind,
            source_id,
            observation_id,
            project_id,
            title,
            due_text,
            assignee_label,
            confidence,
            review_state,
            evidence_excerpt,
            generated_at,
            reviewed_at,
            updated_at
        FROM task_candidates
        ORDER BY updated_at DESC, task_candidate_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_task_candidate).collect()
}
