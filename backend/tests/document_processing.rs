use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use hermes_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingRetryCommand, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use hermes_hub_backend::platform::events::{EventStore, NewEventEnvelope};
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::query_scalar;

#[tokio::test]
async fn enqueue_for_document_creates_extract_text_and_ocr_jobs() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing enqueue test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("enqueue both processing jobs").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_enqueue_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "pipeline.md",
            "# Draft\n\nRun processing queue",
        ))
        .await
        .expect("import markdown document");

    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");

    assert_eq!(jobs.len(), 2);
    assert!(
        jobs.iter()
            .any(|job| step_name(&job.step) == "extract_text")
    );
    assert!(jobs.iter().any(|job| step_name(&job.step) == "ocr"));
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn enqueue_for_document_does_not_reset_terminal_jobs() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live terminal job reset test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("terminal job reset protection").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_terminal_{suffix}");

    document_store
        .import_document(&NewDocumentImport::pdf_metadata(
            &document_id,
            "pipeline.pdf",
            "sha256:processing-terminal",
        ))
        .await
        .expect("import pdf document");

    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs");
    assert_eq!(report.jobs_skipped, 2);
    let terminal_state_before = terminal_state_for_document(&pool, &document_id).await;

    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs again");
    let terminal_state_after = terminal_state_for_document(&pool, &document_id).await;

    assert_eq!(terminal_state_before, terminal_state_after);
}

#[tokio::test]
async fn run_queued_jobs_for_markdown_populates_extracted_text_artifact() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live markdown processing run test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("markdown run generates artifact").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_run_markdown_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "notes.md",
            "First line\n\nExtracted body text.",
        ))
        .await
        .expect("import markdown document");
    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs");

    assert_eq!(report.jobs_queued, 2);
    let extract_status: String = query_scalar::<_, String>(
        "SELECT status FROM document_processing_jobs WHERE document_id = $1 AND step = 'extract_text'",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("extract status");
    let artifact_count: i64 = query_scalar::<_, i64>(
        "SELECT count(*) FROM document_artifacts WHERE document_id = $1 AND artifact_kind = 'extracted_text'",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("artifact count");

    assert_eq!(extract_status, "succeeded");
    assert_eq!(artifact_count, 1);
}

#[tokio::test]
async fn run_queued_jobs_skips_non_markdown_text_extraction_with_summary() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live non-markdown skip test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("non-markdown extract skip").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;

    let suffix = unique_suffix();
    let document_id = format!("doc_processing_non_markdown_{suffix}");

    document_store
        .import_document(&NewDocumentImport::pdf_metadata(
            &document_id,
            "scan.pdf",
            "sha256:processing-non-markdown",
        ))
        .await
        .expect("import pdf document");
    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs");

    let summary: Option<String> = query_scalar::<_, Option<String>>(
        "SELECT last_error_summary FROM document_processing_jobs WHERE document_id = $1 AND step = 'extract_text'",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("extract skip summary");

    assert!(matches!(summary, Some(value) if !value.is_empty()));
}

#[tokio::test]
async fn document_processing_retry_failed_job_requeues_job_against_postgres() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing retry test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("retry failed processing job").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry.md",
            "# Retry\n\nProcessing retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");
    let extract_job = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job");

    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 2,
            last_error_summary = 'temporary extractor failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(&extract_job.job_id)
    .execute(&pool)
    .await
    .expect("mark extract job failed");

    let command_id = format!("document-processing-retry-{suffix}");
    let result = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: command_id.clone(),
            job_id: extract_job.job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect("retry failed job");

    assert_eq!(result.job_id, extract_job.job_id);
    assert_eq!(result.status, DocumentProcessingStatus::Queued);
    assert_eq!(
        result.event_id,
        format!("document_processing_retry:{command_id}")
    );

    let persisted = sqlx::query_as::<_, (String, i32, Option<String>)>(
        r#"
        SELECT status, attempts, last_error_summary
        FROM document_processing_jobs
        WHERE job_id = $1
        "#,
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("persisted retried job");

    assert_eq!(persisted.0, "queued");
    assert_eq!(persisted.1, 0);
    assert_eq!(persisted.2, None);
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn run_queued_jobs_requires_retry_command_for_failed_jobs() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing failed runner retry test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("failed job requires retry command").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_runner_{suffix}");
    let job_id =
        create_failed_extract_text_job(&pool, &document_store, &processing_store, &document_id)
            .await;
    quiesce_document_processing_jobs_except(&pool, &document_id, &job_id).await;

    let skipped_report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs without retry command");
    let failed_state = job_retry_state(&pool, &job_id).await;
    let artifact_count_before_retry = extracted_text_artifact_count(&pool, &document_id).await;

    assert_eq!(skipped_report.jobs_seen, 0);
    assert_eq!(skipped_report.jobs_queued, 0);
    assert_eq!(failed_state.0, "failed");
    assert_eq!(failed_state.1, 2);
    assert!(failed_state.2.is_some());
    assert_eq!(artifact_count_before_retry, 0);

    processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-runner-{suffix}"),
            job_id: job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect("retry failed job");
    let retried_report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run retried job");
    let retried_state = job_retry_state(&pool, &job_id).await;
    let artifact_count_after_retry = extracted_text_artifact_count(&pool, &document_id).await;

    assert_eq!(retried_report.jobs_seen, 1);
    assert_eq!(retried_report.jobs_queued, 1);
    assert_eq!(retried_report.jobs_succeeded, 1);
    assert_eq!(retried_state.0, "succeeded");
    assert_eq!(artifact_count_after_retry, 1);
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_duplicate_same_command_is_idempotent() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing duplicate retry test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("duplicate retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_idempotent_{suffix}");
    let job_id =
        create_failed_extract_text_job(&pool, &document_store, &processing_store, &document_id)
            .await;
    let command = DocumentProcessingRetryCommand {
        command_id: format!("document-processing-retry-idempotent-{suffix}"),
        job_id: job_id.clone(),
        actor_id: "document-processing-test-actor".to_owned(),
    };

    let first = processing_store
        .retry_failed_job(&command)
        .await
        .expect("first retry succeeds");
    let second = processing_store
        .retry_failed_job(&command)
        .await
        .expect("duplicate retry is idempotent");

    assert_eq!(first, second);
    assert_eq!(second.job_id, job_id);
    assert_eq!(second.status, DocumentProcessingStatus::Queued);
    assert_eq!(
        second.event_id,
        format!("document_processing_retry:{}", command.command_id)
    );
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_duplicate_command_for_different_job_is_rejected() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing retry collision test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("duplicate retry command collision").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let existing_document_id = format!("doc_processing_retry_collision_existing_{suffix}");
    let target_document_id = format!("doc_processing_retry_collision_target_{suffix}");
    let existing_job_id = create_failed_extract_text_job(
        &pool,
        &document_store,
        &processing_store,
        &existing_document_id,
    )
    .await;
    let target_job_id = create_failed_extract_text_job(
        &pool,
        &document_store,
        &processing_store,
        &target_document_id,
    )
    .await;
    let command_id = format!("document-processing-retry-collision-{suffix}");
    append_retry_event_for_job(&pool, &command_id, &existing_job_id).await;

    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id,
            job_id: target_job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("command collision must be rejected");

    assert!(matches!(
        error,
        DocumentProcessingError::RetryCommandConflict
    ));
    let persisted = job_retry_state(&pool, &target_job_id).await;
    assert_eq!(persisted.0, "failed");
    assert_eq!(persisted.1, 2);
    assert!(persisted.2.is_some());

    quiesce_processing_jobs_for_document(&pool, &existing_document_id).await;
    quiesce_processing_jobs_for_document(&pool, &target_document_id).await;
}

#[tokio::test]
async fn document_processing_retry_non_failed_job_requires_failed_status() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing non-failed retry test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((pool, document_store, processing_store)) =
        live_context("non-failed retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_non_failed_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry-non-failed.md",
            "# Retry\n\nQueued retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");
    let extract_job = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job");

    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-non-failed-{suffix}"),
            job_id: extract_job.job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("queued job retry must be rejected");

    assert!(matches!(
        error,
        DocumentProcessingError::RetryRequiresFailedJob
    ));
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_missing_job_returns_job_not_found() {
    let Some(_database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing missing retry test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let Some((_pool, _document_store, processing_store)) =
        live_context("missing retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-missing-{suffix}"),
            job_id: format!("document_processing_job:v1:missing-{suffix}:extract_text"),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("missing job retry must be rejected");

    assert!(matches!(error, DocumentProcessingError::JobNotFound));
}

async fn live_context(
    test_name: &str,
) -> Option<(PgPool, DocumentImportStore, DocumentProcessingStore)> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((
        pool.clone(),
        DocumentImportStore::new(pool.clone()),
        DocumentProcessingStore::new(pool),
    ))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn step_name(
    step: &hermes_hub_backend::domains::documents::processing::DocumentProcessingStep,
) -> &'static str {
    match step {
        hermes_hub_backend::domains::documents::processing::DocumentProcessingStep::ExtractText => {
            "extract_text"
        }
        hermes_hub_backend::domains::documents::processing::DocumentProcessingStep::Ocr => "ocr",
    }
}

async fn terminal_state_for_document(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
) -> Vec<(String, String, i32)> {
    sqlx::query_as::<_, (String, String, i32)>(
        "SELECT step, status, attempts FROM document_processing_jobs WHERE document_id = $1 ORDER BY step",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await
    .expect("terminal state")
}

async fn create_failed_extract_text_job(
    pool: &sqlx::postgres::PgPool,
    document_store: &DocumentImportStore,
    processing_store: &DocumentProcessingStore,
    document_id: &str,
) -> String {
    document_store
        .import_document(&NewDocumentImport::markdown(
            document_id,
            "retry-collision.md",
            "# Retry\n\nProcessing retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(document_id)
        .await
        .expect("enqueue processing jobs");
    let job_id = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job")
        .job_id
        .clone();

    fail_processing_job(pool, &job_id).await;

    job_id
}

async fn fail_processing_job(pool: &sqlx::postgres::PgPool, job_id: &str) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 2,
            last_error_summary = 'temporary extractor failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .execute(pool)
    .await
    .expect("mark extract job failed");
}

async fn append_retry_event_for_job(pool: &sqlx::postgres::PgPool, command_id: &str, job_id: &str) {
    let event = NewEventEnvelope::builder(
        format!("document_processing_retry:{command_id}"),
        "document_processing.retry_requested",
        Utc::now(),
        json!({
            "kind": "document_processing_retry",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({
            "kind": "document_processing_job",
            "job_id": job_id,
        }),
    )
    .actor(json!({ "actor_id": "document-processing-test-actor" }))
    .payload(json!({ "job_id": job_id }))
    .build()
    .expect("retry event envelope");

    EventStore::new(pool.clone())
        .append(&event)
        .await
        .expect("append retry collision event");
}

async fn job_retry_state(
    pool: &sqlx::postgres::PgPool,
    job_id: &str,
) -> (String, i32, Option<String>) {
    sqlx::query_as::<_, (String, i32, Option<String>)>(
        r#"
        SELECT status, attempts, last_error_summary
        FROM document_processing_jobs
        WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .expect("job retry state")
}

async fn extracted_text_artifact_count(pool: &sqlx::postgres::PgPool, document_id: &str) -> i64 {
    query_scalar::<_, i64>(
        "SELECT count(*) FROM document_artifacts WHERE document_id = $1 AND artifact_kind = 'extracted_text'",
    )
    .bind(document_id)
    .fetch_one(pool)
    .await
    .expect("extracted text artifact count")
}

async fn quiesce_document_processing_jobs_except(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
    active_job_id: &str,
) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id = $1
          AND job_id <> $2
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .bind(document_id)
    .bind(active_job_id)
    .execute(pool)
    .await
    .expect("quiesce non-target document processing jobs");
}

async fn quiesce_retryable_test_processing_jobs(pool: &sqlx::postgres::PgPool) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id LIKE 'doc_processing_%'
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .execute(pool)
    .await
    .expect("quiesce retryable test processing jobs");
}

async fn quiesce_processing_jobs_for_document(pool: &sqlx::postgres::PgPool, document_id: &str) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id = $1
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .bind(document_id)
    .execute(pool)
    .await
    .expect("quiesce document processing jobs for test document");
}
