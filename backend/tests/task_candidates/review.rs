use hermes_hub_backend::domains::tasks::candidates::{
    TaskCandidateReviewCommand, TaskCandidateReviewState,
};

use super::support::{live_task_candidate_context, seed_message, unique_suffix};

#[tokio::test]
async fn task_candidate_review_confirm_creates_active_task_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("confirm-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-confirm-{suffix}"),
        &format!("Task confirm {suffix}"),
        "Action: review this item",
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'message'",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");

    let result = context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);
    assert_eq!(result.task_candidate_id, task_candidate_id);

    let task_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM tasks
            WHERE task_candidate_id = $1
        )
        "#,
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("task exists");
    assert!(task_exists);
}

#[tokio::test]
async fn task_candidate_review_confirm_materializes_obligation_candidate_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the countersigned agreement {suffix}");
    let quote = format!("I will {statement} by Friday 5pm.");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("obligation-confirm-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-confirm-{suffix}"),
        &format!("Obligation confirm {suffix}"),
        &quote,
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'message'",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");

    let result = context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-obligation-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);

    let task_id: String =
        sqlx::query_scalar("SELECT task_id FROM tasks WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task id");

    let obligation_rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            o.obligation_id,
            o.review_state,
            o.obligated_entity_kind,
            o.obligated_entity_id,
            l.link_kind
        FROM obligations o
        JOIN obligation_task_links l
          ON l.obligation_id = o.obligation_id
        WHERE l.task_id = $1
        ORDER BY o.obligation_id
        "#,
    )
    .bind(&task_id)
    .fetch_all(&context.pool)
    .await
    .expect("linked obligation rows");
    assert_eq!(
        obligation_rows.len(),
        1,
        "confirming an obligation-derived candidate should create one linked obligation"
    );
    assert_eq!(obligation_rows[0].1, "user_confirmed");
    assert_eq!(obligation_rows[0].2, "persona");
    assert_eq!(obligation_rows[0].3, "persona:owner");
    assert_eq!(obligation_rows[0].4, "fulfillment_task");

    let evidence_row: (String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, quote
        FROM obligation_evidence
        WHERE obligation_id = $1
        "#,
    )
    .bind(&obligation_rows[0].0)
    .fetch_one(&context.pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence_row.0, "communication");
    assert_eq!(evidence_row.1, message_id);
    assert_eq!(evidence_row.2.as_deref(), Some(quote.as_str()));
}

#[tokio::test]
async fn obligation_task_candidate_reset_demotes_obligation_review_state_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the vendor approval memo {suffix}");
    let quote = format!("I will {statement} by Friday 5pm.");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("obligation-reset-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-reset-{suffix}"),
        &format!("Obligation reset {suffix}"),
        &quote,
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'message'",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");

    context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-obligation-reset-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");

    let reset = context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-obligation-reset-reset-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::Suggested,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("reset");
    assert_eq!(reset.review_state, TaskCandidateReviewState::Suggested);

    let obligation_row: (String, String) = sqlx::query_as(
        r#"
        SELECT obligation_id, review_state
        FROM obligations
        WHERE statement = $1
        "#,
    )
    .bind(&statement)
    .fetch_one(&context.pool)
    .await
    .expect("obligation row");
    assert_eq!(
        obligation_row.1, "suggested",
        "resetting the candidate must demote the durable Obligation review state"
    );

    let task_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM tasks WHERE task_candidate_id = $1)")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task exists");
    assert!(!task_exists);

    let link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(&obligation_row.0)
            .fetch_one(&context.pool)
            .await
            .expect("obligation task link count");
    assert_eq!(link_count, 0);
}

#[tokio::test]
async fn task_candidate_review_reset_removes_active_task_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("reset-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-reset-{suffix}"),
        &format!("Task reset {suffix}"),
        "Please handle next step",
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'message'",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");

    let _ = context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-reset-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");

    let reset = context
        .store
        .set_review_state(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-reset-reset-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::Suggested,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("reset");
    assert_eq!(reset.review_state, TaskCandidateReviewState::Suggested);

    let state: String =
        sqlx::query_scalar("SELECT review_state FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("candidate state");
    assert_eq!(state, "suggested");

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM tasks WHERE task_candidate_id = $1)")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task exists");
    assert!(!exists);
}
