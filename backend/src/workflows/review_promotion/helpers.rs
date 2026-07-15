use super::*;

pub(super) fn review_keywords(item: &ReviewItem) -> Vec<String> {
    let mut keywords = vec![item.title.trim().to_owned()];
    for word in item.summary.split_whitespace().take(3) {
        let cleaned = word
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect::<String>();
        if !cleaned.is_empty() {
            keywords.push(cleaned);
        }
    }
    keywords.sort();
    keywords.dedup();
    keywords
}

pub(super) fn choose_target_id(target: &ReviewPromotionTarget, prefix: &str, seed: &str) -> String {
    let candidate = target.target_entity_id.trim();
    if !candidate.is_empty() {
        return candidate.to_owned();
    }
    format!("{prefix}:v1:{}", stable_short_hash(seed))
}

pub(super) fn stable_short_hash(seed: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(seed.as_bytes());
    format!("{:x}", digest.finalize())[..16].to_owned()
}

pub(super) fn knowledge_document_title(item: &ReviewItem) -> String {
    let title = item.title.trim();
    if title.ends_with(".md") {
        title.to_owned()
    } else {
        format!("{title}.md")
    }
}

pub(super) fn primary_observation_id(evidence: &[ReviewItemEvidenceRecord]) -> Option<String> {
    evidence
        .iter()
        .find(|record| record.evidence_role == "primary")
        .or_else(|| evidence.first())
        .map(|record| record.observation_id.clone())
}

pub(super) fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn observation_excerpt(observation: &Observation) -> Option<String> {
    for key in [
        "quote",
        "evidence",
        "body",
        "transcript",
        "subject",
        "title",
        "extracted_text",
    ] {
        if let Some(value) = observation.payload.get(key).and_then(Value::as_str) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_owned());
            }
        }
    }

    None
}

pub(super) fn decision_evidence(evidence: &[ReviewItemEvidenceRecord]) -> Vec<NewDecisionEvidence> {
    evidence
        .iter()
        .map(|record| {
            NewDecisionEvidence::observation(record.observation_id.clone()).metadata(json!({
                "evidence_role": record.evidence_role,
                "review_metadata": record.metadata
            }))
        })
        .collect()
}

pub(super) fn obligation_evidence(
    evidence: &[ReviewItemEvidenceRecord],
) -> Vec<NewObligationEvidence> {
    evidence
        .iter()
        .map(|record| {
            NewObligationEvidence::observation(record.observation_id.clone()).metadata(json!({
                "evidence_role": record.evidence_role,
                "review_metadata": record.metadata
            }))
        })
        .collect()
}

pub(super) fn relationship_evidence(
    evidence: &[ReviewItemEvidenceRecord],
) -> Vec<NewRelationshipEvidence> {
    evidence
        .iter()
        .map(|record| {
            NewRelationshipEvidence::observation(record.observation_id.clone()).metadata(json!({
                "evidence_role": record.evidence_role,
                "review_metadata": record.metadata
            }))
        })
        .collect()
}
use crate::domains::decisions::models::evidence::NewDecisionEvidence;
