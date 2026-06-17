use crate::support::*;
use std::env;

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
