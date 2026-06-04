use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

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
                document_kind = EXCLUDED.document_kind,
                title = EXCLUDED.title,
                source_fingerprint = EXCLUDED.source_fingerprint,
                extracted_text = EXCLUDED.extracted_text,
                imported_at = now()
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
        .fetch_one(&self.pool)
        .await?;

        row_to_imported_document(row)
    }
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
        .map(|line| {
            let line = line.trim_end();
            if !line.starts_with('#') {
                return line;
            }

            let stripped_heading = line.trim_start_matches('#');
            stripped_heading
                .strip_prefix(' ')
                .unwrap_or(stripped_heading)
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
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
}
