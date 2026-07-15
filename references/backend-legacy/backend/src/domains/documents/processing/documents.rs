use sqlx::postgres::Postgres;
use sqlx::{Row, Transaction};

use super::errors::DocumentProcessingError;
use super::store::DocumentProcessingStore;

impl DocumentProcessingStore {
    pub(super) async fn ensure_document_exists(
        &self,
        document_id: &str,
    ) -> Result<(), DocumentProcessingError> {
        if self.document_exists(document_id).await? {
            Ok(())
        } else {
            Err(DocumentProcessingError::DocumentNotFound)
        }
    }

    pub(super) async fn document_for_id(
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

    pub(super) async fn document_record_by_id(
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
}

#[derive(Debug)]
pub(super) struct DocumentRecord {
    pub(super) kind: String,
    pub(super) extracted_text: String,
}
