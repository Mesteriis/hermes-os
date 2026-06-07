use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::documents::processing::{DocumentProcessingJob, DocumentProcessingStore};

const DOCUMENT_KIND_MARKDOWN: &str = "markdown";
const DOCUMENT_KIND_PDF: &str = "pdf";
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewDocumentImport {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub source_fingerprint: String,
    pub extracted_text: String,
}

impl NewDocumentImport {
    /// Creates a Markdown import with extracted text and a deterministic V1
    /// local fingerprint of that extracted text. The fingerprint is for local
    /// idempotence only and is not cryptographic evidence of source content.
    pub fn markdown(
        document_id: impl Into<String>,
        title: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        let extracted_text = extract_markdown_text(&markdown.into());
        let source_fingerprint = local_markdown_fingerprint(&extracted_text);

        Self {
            document_id: document_id.into(),
            document_kind: DOCUMENT_KIND_MARKDOWN.to_owned(),
            title: title.into(),
            source_fingerprint,
            extracted_text,
        }
    }

    pub fn pdf_metadata(
        document_id: impl Into<String>,
        title: impl Into<String>,
        source_fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            document_kind: DOCUMENT_KIND_PDF.to_owned(),
            title: title.into(),
            source_fingerprint: source_fingerprint.into(),
            extracted_text: String::new(),
        }
    }

    fn validate(&self) -> Result<ValidatedDocumentImport, DocumentImportError> {
        let document_id = validate_non_empty("document_id", &self.document_id)?;
        let document_kind = validate_non_empty("document_kind", &self.document_kind)?;
        let title = validate_non_empty("title", &self.title)?;
        let source_fingerprint =
            validate_non_empty("source_fingerprint", &self.source_fingerprint)?;

        match document_kind.as_str() {
            DOCUMENT_KIND_MARKDOWN => {
                let extracted_text = self.extracted_text.trim_end().to_owned();
                validate_non_empty("extracted_text", &extracted_text)?;
                Ok(ValidatedDocumentImport {
                    document_id,
                    document_kind,
                    title,
                    source_fingerprint,
                    extracted_text,
                })
            }
            DOCUMENT_KIND_PDF => Ok(ValidatedDocumentImport {
                document_id,
                document_kind,
                title,
                source_fingerprint,
                extracted_text: String::new(),
            }),
            _ => Err(DocumentImportError::InvalidDocumentKind(document_kind)),
        }
    }
}

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
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            return row_to_imported_document(row);
        }

        let existing_kind = sqlx::query_scalar::<_, String>(
            "SELECT document_kind FROM documents WHERE document_id = $1",
        )
        .bind(&document.document_id)
        .fetch_optional(&self.pool)
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

#[derive(Debug, PartialEq)]
pub struct ImportedDocumentWithProcessing {
    pub imported: ImportedDocument,
    pub jobs: Vec<DocumentProcessingJob>,
}

#[derive(Debug, Error)]
pub enum DocumentImportWithProcessingError {
    #[error(transparent)]
    DocumentImport(#[from] DocumentImportError),

    #[error(transparent)]
    Processing(#[from] crate::domains::documents::processing::DocumentProcessingError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedDocument {
    pub document_id: String,
    pub document_kind: String,
    pub title: String,
    pub source_fingerprint: String,
    pub extracted_text: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedDocumentImport {
    document_id: String,
    document_kind: String,
    title: String,
    source_fingerprint: String,
    extracted_text: String,
}

fn row_to_imported_document(row: PgRow) -> Result<ImportedDocument, DocumentImportError> {
    Ok(ImportedDocument {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        extracted_text: row.try_get("extracted_text")?,
        imported_at: row.try_get("imported_at")?,
    })
}

fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, DocumentImportError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(DocumentImportError::EmptyField(field_name));
    }

    Ok(normalized)
}

fn extract_markdown_text(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| match markdown_heading_text(line.trim_end()) {
            Some(heading_text) => heading_text,
            None => line.trim_end(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn markdown_heading_text(line: &str) -> Option<&str> {
    let mut hash_count = 0;
    for character in line.chars() {
        if character == '#' {
            hash_count += 1;
            continue;
        }
        break;
    }

    if !(1..=6).contains(&hash_count) {
        return None;
    }

    line.as_bytes()
        .get(hash_count)
        .filter(|byte| **byte == b' ')
        .map(|_| &line[hash_count + 1..])
}

// V1 local boundary fingerprint only. This is deterministic for idempotence but
// is not cryptographic evidence of source content.
fn local_markdown_fingerprint(extracted_text: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in extracted_text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("local-v1:markdown:{hash:016x}")
}

#[derive(Debug, Error)]
pub enum DocumentImportError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("document_kind must be markdown or pdf: {0}")]
    InvalidDocumentKind(String),

    #[error(
        "document_kind change rejected for document_id={document_id}: existing={existing_kind}, new={new_kind}"
    )]
    DocumentKindChange {
        document_id: String,
        existing_kind: String,
        new_kind: String,
    },

    #[error("document import upsert skipped unexpectedly for document_id={0}")]
    UpsertSkipped(String),
}
