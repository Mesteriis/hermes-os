use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::DocumentImportError;
use super::models::ImportedDocument;

pub(super) fn row_to_imported_document(
    row: PgRow,
) -> Result<ImportedDocument, DocumentImportError> {
    Ok(ImportedDocument {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        extracted_text: row.try_get("extracted_text")?,
        imported_at: row.try_get("imported_at")?,
    })
}
