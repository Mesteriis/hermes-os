use super::models::{DecisionEntityKind, DecisionEvidenceSourceKind, NewDecision};

pub fn decision_id(decision: &NewDecision) -> String {
    let title = normalize_text(&decision.title);
    let decider_kind = decision
        .decided_by_entity_kind
        .map(DecisionEntityKind::as_str)
        .unwrap_or("");
    let decider_id = decision.decided_by_entity_id.as_deref().unwrap_or("");
    let decided_at = decision
        .decided_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_default();

    format!(
        "decision:v1:{}:{}:{}:{}:{}:{}:{}:{}",
        title.len(),
        title,
        decider_kind.len(),
        decider_kind,
        decider_id.len(),
        decider_id,
        decided_at.len(),
        decided_at
    )
}

pub fn evidence_id(
    decision_id: &str,
    source_kind: DecisionEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "decision:evidence:v1:{}:{}:{}:{}:{}:{}",
        decision_id.len(),
        decision_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
