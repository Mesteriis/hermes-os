use hermes_communications_api::CommunicationsClientError;
use hermes_communications_domain::{
    CommunicationsDomainError, canonicalize_communication, convert_client_query_error,
    promote_draft_to_summary,
};
use hermes_communications_ingress::CommunicationObservationDraft;

#[test]
fn promotes_valid_draft() {
    let draft = CommunicationObservationDraft {
        operation_id: "op-1".to_owned(),
        source_id: "source-1".to_owned(),
        source_kind: "mail-imap".to_owned(),
        text_preview: None,
        has_body: true,
        is_final_window: true,
    };
    let summary = promote_draft_to_summary(draft).unwrap();
    assert_eq!(summary.communication_id, "comm-op-1");
    assert_eq!(summary.operation_id, "op-1");
}

#[test]
fn rejects_invalid_draft_and_maps_errors() {
    let draft = CommunicationObservationDraft {
        operation_id: "".to_owned(),
        source_id: "source-1".to_owned(),
        source_kind: "mail-imap".to_owned(),
        text_preview: None,
        has_body: false,
        is_final_window: true,
    };
    assert!(matches!(
        promote_draft_to_summary(draft),
        Err(CommunicationsDomainError::InvalidDraft)
    ));
    assert!(matches!(
        convert_client_query_error(CommunicationsDomainError::InvalidDraft),
        CommunicationsClientError::DraftValidationFailed
    ));
}

#[test]
fn canonicalization_wraps_summary() {
    let summary = hermes_communications_api::CommunicationSummary {
        communication_id: "comm-op-1".to_owned(),
        operation_id: "op-1".to_owned(),
        source_id: "source-1".to_owned(),
        source_kind: "mail-imap".to_owned(),
        has_body: true,
        has_preview: false,
        is_final_window: true,
    };
    let canonical = canonicalize_communication(&summary);
    assert_eq!(canonical.id, summary.communication_id);
}
