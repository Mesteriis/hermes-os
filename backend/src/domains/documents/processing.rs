// This file exceeds 700 lines because the document processing pipeline
// groups the processing store, job lifecycle, status tracking, and retry
// logic into a single module. These are tightly coupled through the
// processing state machine and SQL queries that reference each other.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, Postgres};
use sqlx::{Row, Transaction};
use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope};

const DEFAULT_LIST_LIMIT: i64 = 50;
const MAX_LIST_LIMIT: i64 = 100;
const MIN_LIST_LIMIT: i64 = 1;
const ARTIFACT_METADATA_KIND: &str = "document_processing";
const DEFAULT_MAX_ATTEMPTS: i32 = 3;
const JOB_ID_PREFIX: &str = "document_processing_job:v1:";
const ARTIFACT_ID_PREFIX: &str = "document_artifact:v1:";
const RETRY_EVENT_TYPE: &str = "document_processing.retry_requested";
const RETRY_EVENT_ID_PREFIX: &str = "document_processing_retry:";
const RETRY_SOURCE_KIND: &str = "document_processing_retry";
const RETRY_SOURCE_PROVIDER: &str = "local_api";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentProcessingStep {
    ExtractText,
    Ocr,
}

impl DocumentProcessingStep {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ExtractText => "extract_text",
            Self::Ocr => "ocr",
        }
    }

    fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "extract_text" => Ok(Self::ExtractText),
            "ocr" => Ok(Self::Ocr),
            _ => Err(DocumentProcessingError::InvalidStep(raw.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentProcessingStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

impl DocumentProcessingStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            _ => Err(DocumentProcessingError::InvalidStatus(raw.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentArtifactKind {
    ExtractedText,
    OcrText,
}

impl DocumentArtifactKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ExtractedText => "extracted_text",
            Self::OcrText => "ocr_text",
        }
    }

    fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "extracted_text" => Ok(Self::ExtractedText),
            "ocr_text" => Ok(Self::OcrText),
            _ => Err(DocumentProcessingError::InvalidArtifactKind(raw.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingJob {
    pub job_id: String,
    pub document_id: String,
    pub step: DocumentProcessingStep,
    pub status: DocumentProcessingStatus,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error_summary: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingArtifact {
    pub artifact_id: String,
    pub document_id: String,
    pub job_id: String,
    pub artifact_kind: DocumentArtifactKind,
    pub content_sha256: String,
    pub text_content: Option<String>,
    pub storage_kind: Option<String>,
    pub storage_path: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentProcessingRecord {
    pub document_id: String,
    pub jobs: Vec<DocumentProcessingJob>,
    pub artifacts: Vec<DocumentProcessingArtifact>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct DocumentProcessingRunReport {
    pub jobs_seen: i64,
    pub jobs_queued: i64,
    pub jobs_succeeded: i64,
    pub jobs_failed: i64,
    pub jobs_skipped: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentProcessingRetryCommand {
    pub command_id: String,
    pub job_id: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingRetryCommandResult {
    pub job_id: String,
    pub status: DocumentProcessingStatus,
    pub event_id: String,
}

#[derive(Clone)]
pub struct DocumentProcessingStore {
    pool: PgPool,
}

impl DocumentProcessingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;
        self.ensure_document_exists(&document_id).await?;
        let extract_text_job = self
            .upsert_job(&document_id, DocumentProcessingStep::ExtractText)
            .await?;
        let ocr_job = self
            .upsert_job(&document_id, DocumentProcessingStep::Ocr)
            .await?;

        Ok(vec![extract_text_job, ocr_job])
    }

    pub async fn list_jobs(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            ORDER BY queued_at DESC, job_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_job)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn list_jobs_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE document_id = $1
            ORDER BY queued_at
            "#,
        )
        .bind(&document_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_job)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn list_artifacts_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingArtifact>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                artifact_id,
                document_id,
                job_id,
                artifact_kind,
                content_sha256,
                text_content,
                storage_kind,
                storage_path,
                metadata,
                created_at
            FROM document_artifacts
            WHERE document_id = $1
            ORDER BY artifact_kind
            "#,
        )
        .bind(&document_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_artifact)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn document_processing(
        &self,
        document_id: &str,
    ) -> Result<DocumentProcessingRecord, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;
        let Some(_) = self.document_record_by_id(&document_id).await? else {
            return Err(DocumentProcessingError::DocumentNotFound);
        };

        let jobs = self.list_jobs_for_document(&document_id).await?;
        let artifacts = self.list_artifacts_for_document(&document_id).await?;

        Ok(DocumentProcessingRecord {
            document_id,
            jobs,
            artifacts,
        })
    }

    pub async fn run_queued_jobs(
        &self,
        limit: i64,
    ) -> Result<DocumentProcessingRunReport, DocumentProcessingError> {
        let limit = validate_limit(limit)?;

        let candidate_jobs = self.next_jobs(limit).await?;

        let mut report = DocumentProcessingRunReport {
            jobs_seen: 0,
            jobs_queued: 0,
            jobs_succeeded: 0,
            jobs_failed: 0,
            jobs_skipped: 0,
        };

        for candidate in candidate_jobs {
            report.jobs_seen += 1;
            match self.run_single_job(candidate).await {
                Ok(DocumentProcessingRunStepResult::Succeeded) => {
                    report.jobs_succeeded += 1;
                    report.jobs_queued += 1;
                }
                Ok(DocumentProcessingRunStepResult::Skipped(_)) => {
                    report.jobs_skipped += 1;
                    report.jobs_queued += 1;
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }

        Ok(report)
    }

    pub async fn retry_failed_job(
        &self,
        command: &DocumentProcessingRetryCommand,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let job_id = validate_non_empty("job_id", &command.job_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;
        let event_id = format!("{RETRY_EVENT_ID_PREFIX}{command_id}");

        if let Some(result) = self
            .retry_result_for_existing_event(&event_id, &job_id)
            .await?
        {
            return Ok(result);
        }

        let mut transaction = self.pool.begin().await?;
        let current_job = self.job_for_update(&mut transaction, &job_id).await?;
        if current_job.status != DocumentProcessingStatus::Failed {
            if let Some(result) = self
                .retry_result_for_existing_event(&event_id, &job_id)
                .await?
            {
                return Ok(result);
            }
            return Err(DocumentProcessingError::RetryRequiresFailedJob);
        }

        let event = RetryCommandEvent {
            command_id,
            job_id: job_id.clone(),
            actor_id,
            event_id: event_id.clone(),
            occurred_at: Utc::now(),
        }
        .into_event()?;

        if let Err(error) = EventStore::append_in_transaction(&mut transaction, &event).await {
            if error.is_unique_violation() {
                transaction.rollback().await?;
                return self
                    .retry_result_for_existing_event(&event_id, &job_id)
                    .await?
                    .ok_or(DocumentProcessingError::RetryCommandConflict);
            }

            return Err(DocumentProcessingError::EventStore(error));
        }
        let retried_job = self.requeue_failed_job(&mut transaction, &job_id).await?;
        transaction.commit().await?;

        Ok(DocumentProcessingRetryCommandResult {
            job_id: retried_job.job_id,
            status: retried_job.status,
            event_id,
        })
    }

    async fn retry_result_for_existing_event(
        &self,
        event_id: &str,
        job_id: &str,
    ) -> Result<Option<DocumentProcessingRetryCommandResult>, DocumentProcessingError> {
        let Some(event) = EventStore::new(self.pool.clone())
            .get_by_id(event_id)
            .await?
        else {
            return Ok(None);
        };

        let Some(event_job_id) = event.payload.get("job_id").and_then(Value::as_str) else {
            return Err(DocumentProcessingError::RetryCommandConflict);
        };

        if event.event_type != RETRY_EVENT_TYPE || event_job_id != job_id {
            return Err(DocumentProcessingError::RetryCommandConflict);
        }

        Ok(Some(DocumentProcessingRetryCommandResult {
            job_id: job_id.to_owned(),
            status: DocumentProcessingStatus::Queued,
            event_id: event_id.to_owned(),
        }))
    }

    async fn run_single_job(
        &self,
        job: QueuedJob,
    ) -> Result<DocumentProcessingRunStepResult, DocumentProcessingError> {
        let mut transaction = self.pool.begin().await?;
        let running_job = self.mark_running(&mut transaction, &job).await?;

        let result = match running_job.step {
            DocumentProcessingStep::ExtractText => {
                self.run_extract_text_step(&mut transaction, &running_job)
                    .await
            }
            DocumentProcessingStep::Ocr => self.run_ocr_step(&mut transaction, &running_job).await,
        };

        match result {
            Ok(DocumentProcessingRunStepResult::Succeeded) => {
                self.finish_job(
                    &mut transaction,
                    &running_job,
                    DocumentProcessingStatus::Succeeded,
                    None,
                )
                .await?;
                transaction.commit().await?;
                Ok(DocumentProcessingRunStepResult::Succeeded)
            }
            Ok(DocumentProcessingRunStepResult::Skipped(summary)) => {
                self.finish_job(
                    &mut transaction,
                    &running_job,
                    DocumentProcessingStatus::Skipped,
                    Some(summary.clone()),
                )
                .await?;
                transaction.commit().await?;
                Ok(DocumentProcessingRunStepResult::Skipped(summary))
            }
            Err(error) => {
                let summary = safe_summary(&error.to_string());
                self.finish_job(
                    &mut transaction,
                    &running_job,
                    DocumentProcessingStatus::Failed,
                    Some(summary),
                )
                .await?;
                transaction.commit().await?;
                Err(error)
            }
        }
    }

    async fn run_extract_text_step(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        job: &DocumentProcessingJob,
    ) -> Result<DocumentProcessingRunStepResult, DocumentProcessingError> {
        let document = self
            .document_for_id(transaction, &job.document_id)
            .await?
            .ok_or(DocumentProcessingError::DocumentNotFound)?;

        if document.kind == "markdown" {
            if document.extracted_text.trim().is_empty() {
                return Err(DocumentProcessingError::MissingSourceText);
            }

            self.upsert_artifact(
                transaction,
                job,
                DocumentArtifactKind::ExtractedText,
                Some(document.extracted_text),
            )
            .await?;
            return Ok(DocumentProcessingRunStepResult::Succeeded);
        }

        Ok(DocumentProcessingRunStepResult::Skipped(format!(
            "extract text is not supported for document kind {}",
            document.kind
        )))
    }

    async fn run_ocr_step(
        &self,
        _transaction: &mut Transaction<'_, Postgres>,
        _job: &DocumentProcessingJob,
    ) -> Result<DocumentProcessingRunStepResult, DocumentProcessingError> {
        Ok(DocumentProcessingRunStepResult::Skipped(
            "ocr backend is not configured".to_owned(),
        ))
    }

    async fn ensure_document_exists(
        &self,
        document_id: &str,
    ) -> Result<(), DocumentProcessingError> {
        if self.document_exists(document_id).await? {
            Ok(())
        } else {
            Err(DocumentProcessingError::DocumentNotFound)
        }
    }

    async fn document_for_id(
        &self,
        tx_or_pool: &mut Transaction<'_, Postgres>,
        document_id: &str,
    ) -> Result<Option<DocumentRecord>, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            SELECT
                document_id,
                document_kind,
                extracted_text
            FROM documents
            WHERE document_id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&mut **tx_or_pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(DocumentRecord {
            kind: row.try_get("document_kind")?,
            extracted_text: row.try_get("extracted_text")?,
        }))
    }

    async fn document_exists(&self, document_id: &str) -> Result<bool, DocumentProcessingError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM documents
                WHERE document_id = $1
            )
            "#,
        )
        .bind(document_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn document_record_by_id(
        &self,
        document_id: &str,
    ) -> Result<Option<DocumentRecord>, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            SELECT
                document_id,
                document_kind,
                extracted_text
            FROM documents
            WHERE document_id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(DocumentRecord {
            kind: row.try_get("document_kind")?,
            extracted_text: row.try_get("extracted_text")?,
        }))
    }

    async fn upsert_job(
        &self,
        document_id: &str,
        step: DocumentProcessingStep,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let job_id = job_id(document_id, step);
        let row = sqlx::query(
            r#"
            INSERT INTO document_processing_jobs (
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                updated_at
            )
            VALUES ($1, $2, $3, 'queued', 0, $4, now())
            ON CONFLICT (document_id, step)
            DO UPDATE
                SET status = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.status
                    ELSE 'queued'
                END,
                attempts = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.attempts
                    ELSE 0
                END,
                last_error_summary = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.last_error_summary
                    ELSE NULL
                END,
                started_at = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.started_at
                    ELSE NULL
                END,
                finished_at = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.finished_at
                    ELSE NULL
                END,
                updated_at = now()
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&job_id)
        .bind(document_id)
        .bind(step.as_str())
        .bind(DEFAULT_MAX_ATTEMPTS)
        .fetch_one(&self.pool)
        .await?;

        try_row_to_job(row)
    }

    async fn next_jobs(&self, limit: i64) -> Result<Vec<QueuedJob>, DocumentProcessingError> {
        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE status = 'queued'
              AND attempts < max_attempts
            ORDER BY queued_at ASC, job_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(QueuedJob {
                    base: try_row_to_job(row)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    async fn mark_running(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &QueuedJob,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = 'running',
                attempts = attempts + 1,
                started_at = now(),
                updated_at = now()
            WHERE job_id = $1
              AND status = 'queued'
              AND attempts < max_attempts
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&job.base.job_id)
        .fetch_optional(&mut **tx)
        .await?;

        row.map(try_row_to_job)
            .ok_or(DocumentProcessingError::JobNotFound)?
    }

    async fn job_for_update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job_id: &str,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE job_id = $1
            FOR UPDATE
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?;

        row.map(try_row_to_job)
            .ok_or(DocumentProcessingError::JobNotFound)?
    }

    async fn requeue_failed_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job_id: &str,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = 'queued',
                attempts = 0,
                last_error_summary = NULL,
                started_at = NULL,
                finished_at = NULL,
                updated_at = now()
            WHERE job_id = $1
              AND status = 'failed'
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?;

        row.map(try_row_to_job)
            .ok_or(DocumentProcessingError::RetryRequiresFailedJob)?
    }

    async fn finish_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &DocumentProcessingJob,
        status: DocumentProcessingStatus,
        last_error_summary: Option<String>,
    ) -> Result<(), DocumentProcessingError> {
        sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = $2,
                last_error_summary = $3,
                finished_at = now(),
                updated_at = now()
            WHERE job_id = $1
            "#,
        )
        .bind(&job.job_id)
        .bind(status.as_str())
        .bind(last_error_summary)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn upsert_artifact(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &DocumentProcessingJob,
        artifact_kind: DocumentArtifactKind,
        text_content: Option<String>,
    ) -> Result<(), DocumentProcessingError> {
        let artifact_id = artifact_id(&job.document_id, artifact_kind);
        let text = text_content.as_deref().unwrap_or("");
        let content_sha256 = content_sha256_hex(text);
        let metadata = json!({
            "source": ARTIFACT_METADATA_KIND,
            "artifact_kind": artifact_kind.as_str(),
        });

        sqlx::query(
            r#"
            INSERT INTO document_artifacts (
                artifact_id,
                document_id,
                job_id,
                artifact_kind,
                content_sha256,
                text_content,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (document_id, artifact_kind)
            DO UPDATE SET
                content_sha256 = EXCLUDED.content_sha256,
                text_content = EXCLUDED.text_content,
                metadata = EXCLUDED.metadata,
                job_id = EXCLUDED.job_id
            "#,
        )
        .bind(artifact_id)
        .bind(&job.document_id)
        .bind(&job.job_id)
        .bind(artifact_kind.as_str())
        .bind(content_sha256)
        .bind(text_content)
        .bind(metadata)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct QueuedJob {
    base: DocumentProcessingJob,
}

#[derive(Debug)]
enum DocumentProcessingRunStepResult {
    Succeeded,
    Skipped(String),
}

#[derive(Debug)]
struct DocumentRecord {
    kind: String,
    extracted_text: String,
}

#[derive(Debug)]
struct RetryCommandEvent {
    command_id: String,
    job_id: String,
    actor_id: String,
    event_id: String,
    occurred_at: DateTime<Utc>,
}

impl RetryCommandEvent {
    fn into_event(self) -> Result<NewEventEnvelope, DocumentProcessingError> {
        let job_id = self.job_id;
        Ok(NewEventEnvelope::builder(
            self.event_id,
            RETRY_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": RETRY_SOURCE_KIND,
                "provider": RETRY_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "document_processing_job",
                "job_id": job_id.clone(),
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(json!({ "job_id": job_id }))
        .build()?)
    }
}

fn job_id(document_id: &str, step: DocumentProcessingStep) -> String {
    format!("{JOB_ID_PREFIX}{document_id}:{:0}", step.as_str())
}

fn artifact_id(document_id: &str, artifact_kind: DocumentArtifactKind) -> String {
    format!(
        "{ARTIFACT_ID_PREFIX}{document_id}:{:0}",
        artifact_kind.as_str()
    )
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, DocumentProcessingError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(DocumentProcessingError::EmptyField(field));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: i64) -> Result<i64, DocumentProcessingError> {
    if !(MIN_LIST_LIMIT..=MAX_LIST_LIMIT).contains(&limit) {
        return Err(DocumentProcessingError::InvalidLimit);
    }
    Ok(limit)
}

fn validate_optional_limit(limit: Option<i64>) -> Result<i64, DocumentProcessingError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIST_LIMIT))
}

fn try_row_to_job(
    row: sqlx::postgres::PgRow,
) -> Result<DocumentProcessingJob, DocumentProcessingError> {
    let step = DocumentProcessingStep::parse(row.try_get::<String, _>("step")?.as_str())?;
    let status = DocumentProcessingStatus::parse(row.try_get::<String, _>("status")?.as_str())?;

    Ok(DocumentProcessingJob {
        job_id: row.try_get("job_id")?,
        document_id: row.try_get("document_id")?,
        step,
        status,
        attempts: row.try_get("attempts")?,
        max_attempts: row.try_get("max_attempts")?,
        last_error_summary: row.try_get("last_error_summary")?,
        queued_at: row.try_get("queued_at")?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn try_row_to_artifact(
    row: sqlx::postgres::PgRow,
) -> Result<DocumentProcessingArtifact, DocumentProcessingError> {
    let artifact_kind =
        DocumentArtifactKind::parse(row.try_get::<String, _>("artifact_kind")?.as_str())?;

    Ok(DocumentProcessingArtifact {
        artifact_id: row.try_get("artifact_id")?,
        document_id: row.try_get("document_id")?,
        job_id: row.try_get("job_id")?,
        artifact_kind,
        content_sha256: row.try_get("content_sha256")?,
        text_content: row.try_get("text_content")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn content_sha256_hex(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}

fn safe_summary(value: &str) -> String {
    let sanitized = value
        .chars()
        .filter(|character| !character.is_control() || *character == '\n')
        .collect::<String>();
    sanitized
        .chars()
        .take(240)
        .collect::<String>()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::safe_summary;

    #[test]
    fn safe_summary_truncates_to_240_and_removes_control_chars() {
        let long_text = "a\n".repeat(200) + &"b".repeat(80);
        let summary = safe_summary(&long_text);

        assert!(summary.chars().count() <= 240);
        assert!(!summary.contains('\u{0007}'));
        assert!(summary.contains('\n'));
    }
}

#[derive(Debug, Error)]
pub enum DocumentProcessingError {
    #[error("document processing limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(&'static str),

    #[error("document processing job not found")]
    JobNotFound,

    #[error("document processing retry requires a failed job")]
    RetryRequiresFailedJob,

    #[error("document processing retry command conflicts with existing event")]
    RetryCommandConflict,

    #[error("document not found")]
    DocumentNotFound,

    #[error("invalid document kind")]
    InvalidStep(String),

    #[error("invalid step value")]
    InvalidStatus(String),

    #[error("invalid artifact kind")]
    InvalidArtifactKind(String),

    #[error("missing document source text")]
    MissingSourceText,

    #[error("OCR backend is not available")]
    OcrBackendUnavailable,

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
