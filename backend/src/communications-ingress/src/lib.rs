//! Communications domain ingress contract for ADR-0239.

pub const PACKAGE: &str = "hermes-communications-ingress";

/// Stable operation of the neutral evidence path owned by communications.
pub const MAX_SOURCE_ID_LEN: usize = 128;
pub const MAX_PREVIEW_BYTES: usize = 2048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceEnvelope {
    pub source_kind: String,
    pub source_id: String,
}

#[derive(Clone, Debug)]
pub struct CommunicationObservationDraft {
    pub operation_id: String,
    pub source_id: String,
    pub source_kind: String,
    pub text_preview: Option<String>,
    pub has_body: bool,
    pub is_final_window: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IngressDraftError {
    EmptySourceId,
    SourceTooLong,
    PreviewTooLarge,
}

pub fn validate_source_id(source_id: &str) -> bool {
    !source_id.trim().is_empty() && source_id.len() <= MAX_SOURCE_ID_LEN
}

pub fn sanitize_text_preview(mut preview: String) -> String {
    if preview.len() > MAX_PREVIEW_BYTES {
        preview.truncate(MAX_PREVIEW_BYTES);
    }
    preview
}

pub fn new_communication_observation_draft(
    operation_id: impl Into<String>,
    source: SourceEnvelope,
    preview: Option<String>,
    has_body: bool,
    is_final_window: bool,
) -> Result<CommunicationObservationDraft, IngressDraftError> {
    if !validate_source_id(&source.source_id) {
        return Err(IngressDraftError::EmptySourceId);
    }
    if source.source_id.len() > MAX_SOURCE_ID_LEN {
        return Err(IngressDraftError::SourceTooLong);
    }
    let text_preview = preview.and_then(|value| {
        let sanitized = sanitize_text_preview(value);
        if sanitized.is_empty() {
            None
        } else {
            Some(sanitized)
        }
    });
    if text_preview
        .as_deref()
        .is_some_and(|preview| preview.len() > MAX_PREVIEW_BYTES)
    {
        return Err(IngressDraftError::PreviewTooLarge);
    }
    Ok(CommunicationObservationDraft {
        operation_id: operation_id.into(),
        source_id: source.source_id,
        source_kind: source.source_kind,
        text_preview,
        has_body,
        is_final_window,
    })
}
