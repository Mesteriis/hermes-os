use chrono::{DateTime, Utc};

use super::errors::DocumentImportError;
use super::fingerprint::local_markdown_fingerprint;
use super::markdown::extract_markdown_text;
use super::validation::{ValidatedDocumentImport, validate_document_import};
use crate::domains::documents::processing::models::DocumentProcessingJob;

pub(super) const DOCUMENT_KIND_MARKDOWN: &str = "markdown";
pub(super) const DOCUMENT_KIND_PDF: &str = "pdf";

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

    pub(super) fn validate(&self) -> Result<ValidatedDocumentImport, DocumentImportError> {
        validate_document_import(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct ImportedDocumentWithProcessing {
    pub imported: ImportedDocument,
    pub jobs: Vec<DocumentProcessingJob>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedDocument {
    pub document_id: String,
    pub document_kind: String,
    pub observation_id: String,
    pub title: String,
    pub source_fingerprint: String,
    pub extracted_text: String,
    pub imported_at: DateTime<Utc>,
}
