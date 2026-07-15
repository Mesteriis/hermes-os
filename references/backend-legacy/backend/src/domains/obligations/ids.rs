use super::models::entity_kind::ObligationEntityKind;
use super::models::obligation::NewObligation;

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

fn normalize_statement(statement: &str) -> String {
    statement
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
