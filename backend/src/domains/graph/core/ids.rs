use super::models::{GraphEvidenceSourceKind, GraphNodeKind, RelationshipType};

pub fn node_id(kind: GraphNodeKind, stable_key: &str) -> String {
    format!("graph:node:v1:{}:{stable_key}", kind.as_str())
}

pub fn edge_id(
    source_node_id: &str,
    relationship_type: RelationshipType,
    target_node_id: &str,
) -> String {
    format!(
        "graph:edge:v1:{}:{}:{}:{}:{}:{}",
        source_node_id.len(),
        source_node_id,
        relationship_type.as_str().len(),
        relationship_type.as_str(),
        target_node_id.len(),
        target_node_id
    )
}

pub fn evidence_id(edge_id: &str, source_kind: GraphEvidenceSourceKind, source_id: &str) -> String {
    format!(
        "graph:evidence:v1:{}:{}:{}:{}:{}:{}",
        edge_id.len(),
        edge_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}
