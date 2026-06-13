use hermes_hub_backend::engines::memory::MemoryEngine;

#[test]
fn memory_engine_builds_persona_notes_memory_card_draft() {
    let draft = MemoryEngine::persona_notes_memory_card(
        "person:v1:email:alice@example.com",
        "  Met Alice at the local-first workshop.  ",
    )
    .expect("notes should create a memory card draft");

    assert_eq!(draft.title, "Compatibility notes");
    assert_eq!(draft.description, "Met Alice at the local-first workshop.");
    assert_eq!(
        draft.source,
        "persons.notes:person:v1:email:alice@example.com"
    );
    assert_eq!(draft.confidence, 1.0);
    assert_eq!(draft.importance, 5);
}

#[test]
fn memory_engine_ignores_empty_persona_notes() {
    let draft = MemoryEngine::persona_notes_memory_card("person:v1:email:alice@example.com", "  ");

    assert!(draft.is_none());
}

#[test]
fn memory_engine_builds_source_backed_persona_fact_draft() {
    let draft = MemoryEngine::persona_fact_memory(
        "person:v1:email:alice@example.com",
        " interest ",
        " local-first systems ",
        " communication_messages:message-1 ",
        0.84,
    )
    .expect("source-backed persona fact should be valid");

    assert_eq!(draft.affected_entity_kind, "persona");
    assert_eq!(draft.affected_entity_id, "person:v1:email:alice@example.com");
    assert_eq!(draft.fact_type, "interest");
    assert_eq!(draft.value, "local-first systems");
    assert_eq!(draft.source, "communication_messages:message-1");
    assert_eq!(draft.confidence, 0.84);
    assert_eq!(draft.review_state, "accepted");
    assert_eq!(draft.produced_by, "memory_engine");
}

#[test]
fn memory_engine_rejects_unsourced_persona_fact_draft() {
    let error = MemoryEngine::persona_fact_memory(
        "person:v1:email:alice@example.com",
        "interest",
        "local-first systems",
        " ",
        0.84,
    )
    .expect_err("fact source should be required");

    assert_eq!(error.to_string(), "memory source must not be empty");
}
