use serde_json::json;

use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn document_processing_job_retry(
        actor_id: impl Into<String>,
        job_id: impl Into<String>,
    ) -> Self {
        Self::new(
            actor_id,
            "document_processing.job.retry",
            "POST",
            "/api/v1/document-processing/jobs/{job_id}/retry",
            "document_processing_job",
            Some(job_id.into()),
            json!({}),
        )
    }
}
