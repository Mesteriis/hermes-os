use hermes_communications_ingress::{
    MAX_PREVIEW_BYTES, SourceEnvelope, new_communication_observation_draft, sanitize_text_preview,
    validate_source_id,
};

#[test]
fn validation_and_preview_sanitization() {
    assert!(validate_source_id("abc-123"));
    assert!(!validate_source_id(""));
    assert!(!validate_source_id("   "));
    let preview = sanitize_text_preview("a".repeat(9));
    assert_eq!(preview, "a".repeat(9));

    let source = SourceEnvelope {
        source_kind: "mail-imap".to_owned(),
        source_id: "source-1".to_owned(),
    };
    let draft = new_communication_observation_draft(
        "op-1",
        source,
        Some("short preview".to_owned()),
        true,
        false,
    )
    .expect("draft");
    assert_eq!(draft.source_id, "source-1");
    assert_eq!(draft.text_preview, Some("short preview".to_owned()));
}

#[test]
fn rejects_empty_source_and_overlong_preview() {
    let source = SourceEnvelope {
        source_kind: "mail-imap".to_owned(),
        source_id: "".to_owned(),
    };
    assert!(new_communication_observation_draft("op-1", source, None, true, false).is_err());

    let too_large = "x".repeat(MAX_PREVIEW_BYTES + 1);
    let source = SourceEnvelope {
        source_kind: "mail-imap".to_owned(),
        source_id: "source-2".to_owned(),
    };
    assert!(
        new_communication_observation_draft("op-2", source, Some(too_large), true, false).is_ok()
    );
}
