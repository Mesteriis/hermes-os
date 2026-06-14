use sqlx::Postgres;
use sqlx::Transaction;

use super::super::errors::TaskCandidateError;
use super::super::ids::task_id_from_candidate;
use super::super::models::StoredCandidateRow;

pub(super) async fn upsert_task_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    candidate: &StoredCandidateRow,
    event_id: &str,
    actor_id: &str,
) -> Result<(), TaskCandidateError> {
    sqlx::query(
        r#"
        INSERT INTO tasks (
            task_id,
            task_candidate_id,
            title,
            source_kind,
            source_id,
            project_id,
            status,
            created_from_event_id,
            created_by_actor_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'active', $7, $8)
        ON CONFLICT (task_candidate_id)
        DO UPDATE SET
            title = EXCLUDED.title,
            source_kind = EXCLUDED.source_kind,
            source_id = EXCLUDED.source_id,
            project_id = EXCLUDED.project_id,
            status = EXCLUDED.status,
            created_from_event_id = EXCLUDED.created_from_event_id,
            created_by_actor_id = EXCLUDED.created_by_actor_id,
            updated_at = now()
        "#,
    )
    .bind(task_id_from_candidate(task_candidate_id))
    .bind(task_candidate_id)
    .bind(&candidate.title)
    .bind(&candidate.source_kind)
    .bind(&candidate.source_id)
    .bind(&candidate.project_id)
    .bind(event_id)
    .bind(actor_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
