use super::errors::DocumentImportError;
use super::models::{DOCUMENT_KIND_MARKDOWN, DOCUMENT_KIND_PDF, NewDocumentImport};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ValidatedDocumentImport {
    pub(super) document_id: String,
    pub(super) document_kind: String,
    pub(super) title: String,
    pub(super) source_fingerprint: String,
    pub(super) extracted_text: String,
}

pub(super) fn validate_document_import(
    document: &NewDocumentImport,
) -> Result<ValidatedDocumentImport, DocumentImportError> {
    let document_id = validate_non_empty("document_id", &document.document_id)?;
    let document_kind = validate_non_empty("document_kind", &document.document_kind)?;
    let title = validate_non_empty("title", &document.title)?;
    let source_fingerprint =
        validate_non_empty("source_fingerprint", &document.source_fingerprint)?;

    match document_kind.as_str() {
        DOCUMENT_KIND_MARKDOWN => {
            let extracted_text = document.extracted_text.trim_end().to_owned();
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
