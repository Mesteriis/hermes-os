use super::models::{NewObligation, ObligationEntityKind, ObligationEvidenceSourceKind};

pub fn obligation_id(obligation: &NewObligation) -> String {
    let beneficiary_kind = obligation
        .beneficiary_entity_kind
        .map(ObligationEntityKind::as_str)
        .unwrap_or("");
    let beneficiary_id = obligation.beneficiary_entity_id.as_deref().unwrap_or("");
    let statement = normalize_statement(&obligation.statement);

    format!(
        "obligation:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        obligation.obligated_entity_kind.as_str().len(),
        obligation.obligated_entity_kind.as_str(),
        obligation.obligated_entity_id.len(),
        obligation.obligated_entity_id,
        beneficiary_kind.len(),
        beneficiary_kind,
        beneficiary_id.len(),
        beneficiary_id,
        statement.len(),
        statement
    )
}

pub fn evidence_id(
    obligation_id: &str,
    source_kind: ObligationEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "obligation:evidence:v1:{}:{}:{}:{}:{}:{}",
        obligation_id.len(),
        obligation_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn normalize_statement(statement: &str) -> String {
    statement
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
