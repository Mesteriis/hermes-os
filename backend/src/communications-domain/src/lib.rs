//! Communications domain implementation placeholder for the ADR-0239 slice.

use hermes_communications_api::CommunicationsClientError;
use hermes_communications_api::PACKAGE as API_PACKAGE;
use hermes_communications_ingress::CommunicationObservationDraft;
use hermes_communications_ingress::PACKAGE as INGRESS_PACKAGE;

pub const PACKAGE: &str = "hermes-communications-domain";

pub fn dependencies() -> (&'static str, &'static str) {
    (API_PACKAGE, INGRESS_PACKAGE)
}

#[derive(Clone, Debug)]
pub struct CanonicalCommunication {
    pub id: String,
    pub summary: CommunicationSummary,
}

pub use hermes_communications_api::CommunicationSummary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationsDomainError {
    InvalidDraft,
    AlreadyPromoted,
}

pub fn promote_draft_to_summary(
    draft: CommunicationObservationDraft,
) -> Result<CommunicationSummary, CommunicationsDomainError> {
    if draft.operation_id.trim().is_empty() || draft.source_id.trim().is_empty() {
        return Err(CommunicationsDomainError::InvalidDraft);
    }
    Ok(CommunicationSummary {
        communication_id: format!("comm-{}", draft.operation_id),
        operation_id: draft.operation_id,
        source_id: draft.source_id,
        source_kind: draft.source_kind,
        has_body: draft.has_body,
        has_preview: draft.text_preview.is_some(),
        is_final_window: draft.is_final_window,
    })
}

pub fn convert_client_query_error(error: CommunicationsDomainError) -> CommunicationsClientError {
    match error {
        CommunicationsDomainError::InvalidDraft => CommunicationsClientError::DraftValidationFailed,
        CommunicationsDomainError::AlreadyPromoted => {
            CommunicationsClientError::DuplicateObservation
        }
    }
}

pub fn canonicalize_communication(summary: &CommunicationSummary) -> CanonicalCommunication {
    CanonicalCommunication {
        id: summary.communication_id.clone(),
        summary: summary.clone(),
    }
}
