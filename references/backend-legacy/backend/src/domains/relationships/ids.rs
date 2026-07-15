use super::models::{RelationshipEntityKind, RelationshipEvidenceSourceKind};

pub fn relationship_id(
    source_entity_kind: RelationshipEntityKind,
    source_entity_id: &str,
    relationship_type: &str,
    target_entity_kind: RelationshipEntityKind,
    target_entity_id: &str,
) -> String {
    format!(
        "relationship:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        source_entity_kind.as_str().len(),
        source_entity_kind.as_str(),
        source_entity_id.len(),
        source_entity_id,
        relationship_type.len(),
        relationship_type,
        target_entity_kind.as_str().len(),
        target_entity_kind.as_str(),
        target_entity_id.len(),
        target_entity_id
    )
}

pub fn evidence_id(
    relationship_id: &str,
    source_kind: RelationshipEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "relationship:evidence:v1:{}:{}:{}:{}:{}:{}",
        relationship_id.len(),
        relationship_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}
