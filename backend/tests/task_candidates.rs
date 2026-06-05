use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::documents::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::event_log::{EventStore, NewEventEnvelope};
use hermes_hub_backend::messages::{MessageProjectionStore, project_raw_email_message};
use hermes_hub_backend::storage::Database;
use hermes_hub_backend::task_candidates::{
    TaskCandidateReviewCommand, TaskCandidateReviewState, TaskCandidateStore,
};

const TASK_CANDIDATE_REVIEW_EVENT_TYPE: &str = "task_candidate.review_state_changed";

#[tokio::test]
async fn task_candidate_refresh_creates_message_and_document_candidates_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task candidate test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = task_candidate_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let keyword = format!("TaskCandidate{suffix}");

    let message_id = seed_message(
        &context,
        suffix,
        &format!("sender-{suffix}@example.com"),
        &[format!("recipient-{suffix}@example.com")],
        &format!("provider-task-candidate-msg-{suffix}"),
        &format!("{keyword} Update"),
        "Please action: schedule sync call",
    )
    .await;
    let document_id = seed_document(
        &context.pool,
        &format!("document_task_candidate_{suffix}"),
        &format!("{keyword} architecture"),
        "Follow up: draft document",
    )
    .await;

    let refreshed = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh candidates");
    assert!(refreshed >= 2);

    let message_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, source_kind, review_state
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_all(&context.pool)
    .await
    .expect("message candidate rows");
    assert_eq!(
        message_rows.len(),
        1,
        "should persist deterministic message candidate"
    );
    assert_eq!(message_rows[0].1, "message");
    assert_eq!(message_rows[0].2, "suggested");

    let document_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT task_candidate_id, source_kind, review_state
        FROM task_candidates
        WHERE source_id = $1
        "#,
    )
    .bind(&document_id)
    .fetch_all(&context.pool)
    .await
    .expect("document candidate rows");
    assert_eq!(
        document_rows.len(),
        1,
        "should persist deterministic document candidate"
    );
    assert_eq!(document_rows[0].1, "document");
}

#[tokio::test]
async fn task_candidate_review_confirm_creates_active_task_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task candidate test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = task_candidate_context(&database_url)
        .await
        .expect("context");
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
async fn task_candidate_review_reset_removes_active_task_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task candidate test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = task_candidate_context(&database_url)
        .await
        .expect("context");
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

#[tokio::test]
async fn task_candidate_review_event_rebuilds_state_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task candidate test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = task_candidate_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("event-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-event-{suffix}"),
        &format!("Task event {suffix}"),
        "Action: verify integration",
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

    let confirm_event = build_review_event(
        &task_candidate_id,
        TaskCandidateReviewState::UserConfirmed,
        "event-reviewer",
        &format!("task-candidate-event-confirm-{suffix}"),
    );
    let reject_event = build_review_event(
        &task_candidate_id,
        TaskCandidateReviewState::UserRejected,
        "event-reviewer",
        &format!("task-candidate-event-reject-{suffix}"),
    );

    context
        .event_store
        .append(&confirm_event)
        .await
        .expect("append confirm event");
    context
        .event_store
        .append(&reject_event)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event.event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    context
        .store
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm event");
    let reject_event = context
        .event_store
        .get_by_id(&reject_event.event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    context
        .store
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject event");

    let state: String =
        sqlx::query_scalar("SELECT review_state FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("load state");
    assert_eq!(state, "user_rejected");

    let event_id: String =
        sqlx::query_scalar("SELECT event_id FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("load event id");
    assert_eq!(
        event_id,
        format!("task_candidate_review:task-candidate-event-reject-{suffix}")
    );
}

struct TaskCandidateTestContext {
    pool: PgPool,
    store: TaskCandidateStore,
    event_store: EventStore,
}

async fn task_candidate_context(database_url: &str) -> Option<TaskCandidateTestContext> {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(TaskCandidateTestContext {
        store: TaskCandidateStore::new(pool.clone()),
        event_store: EventStore::new(pool.clone()),
        pool,
    })
}

async fn seed_message(
    context: &TaskCandidateTestContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_task_candidate_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(context.pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Task Candidate Gmail",
            format!("task-candidate-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_task_candidate_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:task-candidate:{suffix}:{provider_record_id}"),
                format!("batch-task-candidate-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"task_candidate_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(context.pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

async fn seed_document(pool: &PgPool, document_id: &str, title: &str, body: &str) -> String {
    let import = NewDocumentImport::markdown(document_id, title, body);
    DocumentImportStore::new(pool.clone())
        .import_document(&import)
        .await
        .expect("document import");
    document_id.to_owned()
}

fn build_review_event(
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        format!("task_candidate_review:{command_id}"),
        TASK_CANDIDATE_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "task_candidate_review",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({"kind": "task_candidate_review"}),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "task_candidate_id": task_candidate_id,
        "review_state": review_state.as_str(),
    }))
    .build()
    .expect("review event")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
