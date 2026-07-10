use super::models::MemoryCardDraft;

pub(super) fn persona_notes_memory_card(persona_id: &str, notes: &str) -> Option<MemoryCardDraft> {
    let description = notes.trim();
    if description.is_empty() {
        return None;
    }

    Some(MemoryCardDraft {
        title: "Compatibility notes".to_owned(),
        description: description.to_owned(),
        source: format!("personas.notes:{persona_id}"),
        confidence: 1.0,
        importance: 5,
    })
}
