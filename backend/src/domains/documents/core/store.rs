use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::{DocumentImportError, DocumentImportWithProcessingError};
use super::models::{ImportedDocument, ImportedDocumentWithProcessing, NewDocumentImport};
use super::rows::row_to_imported_document;
use crate::domains::documents::processing::DocumentProcessingStore;

#[derive(Clone)]
pub struct DocumentImportStore {
    pool: PgPool,
}

impl DocumentImportStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn import_document_and_enqueue_processing(
        &self,
        document: &NewDocumentImport,
        processing_store: &DocumentProcessingStore,
    ) -> Result<ImportedDocumentWithProcessing, DocumentImportWithProcessingError> {
        let imported = self.import_document(document).await?;
        let jobs = processing_store
            .enqueue_for_document(&imported.document_id)
            .await?;

        Ok(ImportedDocumentWithProcessing { imported, jobs })
    }

    pub async fn import_document(
        &self,
        document: &NewDocumentImport,
    ) -> Result<ImportedDocument, DocumentImportError> {
        document.validate()?;
        let mut transaction = self.pool.begin().await?;
        let imported = Self::import_document_in_transaction(&mut transaction, document).await?;
        transaction.commit().await?;
        Ok(imported)
    }

    pub(crate) async fn import_document_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
    ) -> Result<ImportedDocument, DocumentImportError> {
        let document = document.validate()?;

        let row = sqlx::query(
            r#"
            INSERT INTO documents (
                document_id,
                document_kind,
                title,
                source_fingerprint,
                extracted_text
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (document_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                source_fingerprint = EXCLUDED.source_fingerprint,
                extracted_text = EXCLUDED.extracted_text
            WHERE documents.document_kind = EXCLUDED.document_kind
            RETURNING
                document_id,
                document_kind,
                title,
                source_fingerprint,
                extracted_text,
                imported_at
            "#,
        )
        .bind(&document.document_id)
        .bind(&document.document_kind)
        .bind(&document.title)
        .bind(&document.source_fingerprint)
        .bind(&document.extracted_text)
        .fetch_optional(&mut **transaction)
        .await?;

        if let Some(row) = row {
            return row_to_imported_document(row);
        }

        let existing_kind = sqlx::query_scalar::<_, String>(
            "SELECT document_kind FROM documents WHERE document_id = $1",
        )
        .bind(&document.document_id)
        .fetch_optional(&mut **transaction)
        .await?;

        match existing_kind {
            Some(existing_kind) => Err(DocumentImportError::DocumentKindChange {
                document_id: document.document_id,
                existing_kind,
                new_kind: document.document_kind,
            }),
            None => Err(DocumentImportError::UpsertSkipped(document.document_id)),
        }
    }
}
