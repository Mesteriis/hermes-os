use hermes_communications_domain::CommunicationSummary;
use hermes_communications_persistence::{
    CommunicationsPersistence, CommunicationsPersistenceError,
};

#[test]
fn persists_and_rejects_duplicates() {
    let mut persistence = CommunicationsPersistence::new();
    let summary = CommunicationSummary {
        communication_id: "comm-op-1".to_owned(),
        operation_id: "op-1".to_owned(),
        source_id: "source-1".to_owned(),
        source_kind: "mail-imap".to_owned(),
        has_body: true,
        has_preview: false,
        is_final_window: true,
    };
    assert!(persistence.persist(&summary).is_ok());
    assert!(matches!(
        persistence.persist(&summary),
        Err(CommunicationsPersistenceError::DuplicateOperation)
    ));
    assert!(persistence.has("op-1"));
}
