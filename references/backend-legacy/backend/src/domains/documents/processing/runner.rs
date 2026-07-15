use sqlx::{Postgres, Transaction};

use super::errors::DocumentProcessingError;
use super::jobs::QueuedJob;
use super::models::{
    DocumentArtifactKind, DocumentProcessingJob, DocumentProcessingRunReport,
    DocumentProcessingStatus, DocumentProcessingStep,
};
use super::store::DocumentProcessingStore;
use super::validation::validate_limit;

impl DocumentProcessingStore {
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
}

#[derive(Debug)]
enum DocumentProcessingRunStepResult {
    Succeeded,
    Skipped(String),
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
