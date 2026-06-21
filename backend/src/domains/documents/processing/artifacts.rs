use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Transaction};

use super::constants::ARTIFACT_METADATA_KIND;
use super::errors::DocumentProcessingError;
use super::ids::artifact_id;
use super::models::{DocumentArtifactKind, DocumentProcessingJob};
use super::store::DocumentProcessingStore;

impl DocumentProcessingStore {
    pub(super) async fn upsert_artifact(
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

fn content_sha256_hex(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}
