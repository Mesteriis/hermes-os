use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::errors::AttentionEngineError;
use super::models::{
    AttentionCandidate, AttentionCard, AttentionConfidenceExplanation, AttentionEvidenceRef,
    AttentionExplainability, AttentionImportance, AttentionRelatedEntity, AttentionReviewStatus,
    AttentionSuggestedAction,
};

pub struct AttentionEngine;

impl AttentionEngine {
    pub fn build_cards(
        candidates: &[AttentionCandidate],
    ) -> Result<Vec<AttentionCard>, AttentionEngineError> {
        let mut groups: BTreeMap<String, Vec<AttentionCandidate>> = BTreeMap::new();

        for candidate in candidates {
            candidate.validate()?;
            if !candidate.review_status()?.is_active() {
                continue;
            }

            groups
                .entry(candidate.normalized_group_key()?)
                .or_default()
                .push(candidate.clone());
        }

        let mut cards = groups
            .into_iter()
            .map(|(group_key, candidates)| build_group_card(&group_key, candidates))
            .collect::<Result<Vec<_>, _>>()?;

        cards.sort_by(compare_cards);
        Ok(cards)
    }
}

fn build_group_card(
    group_key: &str,
    candidates: Vec<AttentionCandidate>,
) -> Result<AttentionCard, AttentionEngineError> {
    let primary = primary_candidate(&candidates);
    let evidence = unique_evidence(&candidates);
    let related_entities = unique_related_entities(&candidates);
    let suggested_actions = unique_suggested_actions(&candidates);
    let review_item_ids = review_item_ids(&candidates);
    let importance = candidates
        .iter()
        .map(candidate_importance)
        .max_by_key(|importance| importance.rank())
        .unwrap_or(AttentionImportance::Low);
    let confidence = candidates
        .iter()
        .map(|candidate| candidate.confidence)
        .fold(0.0, f64::max);
    let source_summary = source_summary(primary, candidates.len());
    let why_this_matters = why_this_matters(primary, evidence.len());
    let confidence_explanation = AttentionConfidenceExplanation {
        score: confidence,
        rationale: format!(
            "{:.0}% confidence from {} evidence source(s).",
            confidence * 100.0,
            evidence.len()
        ),
    };

    Ok(AttentionCard {
        id: format!("attention:review:{group_key}"),
        title: primary.title.trim().to_owned(),
        summary: primary.summary.trim().to_owned(),
        importance,
        confidence,
        evidence_count: evidence.len(),
        related_entities: related_entities.clone(),
        trace_id: primary.trace_id.trim().to_owned(),
        review_item_ids,
        suggested_actions: suggested_actions.clone(),
        source_summary,
        explainability: AttentionExplainability {
            why_this_matters,
            evidence,
            confidence: confidence_explanation,
            related_objects: related_entities,
            suggested_actions,
        },
    })
}

fn primary_candidate(candidates: &[AttentionCandidate]) -> &AttentionCandidate {
    candidates
        .iter()
        .max_by(|left, right| compare_candidates(left, right))
        .expect("attention group cannot be empty")
}

fn compare_candidates(left: &AttentionCandidate, right: &AttentionCandidate) -> Ordering {
    candidate_importance(left)
        .rank()
        .cmp(&candidate_importance(right).rank())
        .then_with(|| {
            left.confidence
                .partial_cmp(&right.confidence)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| left.evidence.len().cmp(&right.evidence.len()))
        .then_with(|| right.review_item_id.cmp(&left.review_item_id))
}

fn compare_cards(left: &AttentionCard, right: &AttentionCard) -> Ordering {
    right
        .importance
        .rank()
        .cmp(&left.importance.rank())
        .then_with(|| {
            right
                .confidence
                .partial_cmp(&left.confidence)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| left.id.cmp(&right.id))
}

fn candidate_importance(candidate: &AttentionCandidate) -> AttentionImportance {
    let kind = candidate.candidate_kind.trim();
    if kind == "contradiction_candidate" && candidate.confidence >= 0.8 {
        return AttentionImportance::Critical;
    }

    if candidate.review_status().ok() == Some(AttentionReviewStatus::Approved) {
        return AttentionImportance::High;
    }

    if matches!(
        kind,
        "potential_obligation" | "potential_task" | "potential_decision" | "project_link_candidate"
    ) && candidate.confidence >= 0.75
    {
        return AttentionImportance::High;
    }

    if candidate.confidence >= 0.55 || candidate.evidence.len() > 1 {
        AttentionImportance::Medium
    } else {
        AttentionImportance::Low
    }
}

fn unique_evidence(candidates: &[AttentionCandidate]) -> Vec<AttentionEvidenceRef> {
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::new();

    for candidate in candidates {
        for item in &candidate.evidence {
            let key = (
                item.observation_id.trim().to_owned(),
                item.role.trim().to_owned(),
            );
            if seen.insert(key) {
                evidence.push(AttentionEvidenceRef {
                    observation_id: item.observation_id.trim().to_owned(),
                    role: item.role.trim().to_owned(),
                });
            }
        }
    }

    evidence
}

fn unique_related_entities(candidates: &[AttentionCandidate]) -> Vec<AttentionRelatedEntity> {
    let mut seen = BTreeSet::new();
    let mut entities = Vec::new();

    for candidate in candidates {
        for entity in &candidate.related_entities {
            let key = (
                entity.entity_kind.trim().to_owned(),
                entity.entity_id.trim().to_owned(),
            );
            if seen.insert(key) {
                entities.push(AttentionRelatedEntity {
                    entity_kind: entity.entity_kind.trim().to_owned(),
                    entity_id: entity.entity_id.trim().to_owned(),
                    label: entity.label.as_ref().map(|label| label.trim().to_owned()),
                });
            }
        }
    }

    entities
}

fn unique_suggested_actions(candidates: &[AttentionCandidate]) -> Vec<AttentionSuggestedAction> {
    let mut seen = BTreeSet::new();
    let mut actions = Vec::new();

    for candidate in candidates {
        for action in &candidate.suggested_actions {
            let key = (
                action.action_kind.trim().to_owned(),
                action.label.trim().to_owned(),
                action
                    .target_domain
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default()
                    .to_owned(),
                action
                    .target_entity_kind
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default()
                    .to_owned(),
            );
            if seen.insert(key) {
                actions.push(AttentionSuggestedAction {
                    action_kind: action.action_kind.trim().to_owned(),
                    label: action.label.trim().to_owned(),
                    target_domain: action
                        .target_domain
                        .as_ref()
                        .map(|target| target.trim().to_owned()),
                    target_entity_kind: action
                        .target_entity_kind
                        .as_ref()
                        .map(|target| target.trim().to_owned()),
                });
            }
        }
    }

    actions
}

fn review_item_ids(candidates: &[AttentionCandidate]) -> Vec<String> {
    let mut ids = candidates
        .iter()
        .map(|candidate| candidate.review_item_id.trim().to_owned())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

fn source_summary(primary: &AttentionCandidate, group_size: usize) -> String {
    let source_summary = primary.source_summary.trim();
    if group_size == 1 {
        return source_summary.to_owned();
    }

    format!(
        "{source_summary} Grouped with {} related review item(s).",
        group_size - 1
    )
}

fn why_this_matters(primary: &AttentionCandidate, evidence_count: usize) -> String {
    format!(
        "{} remains in Review as {} with {} evidence source(s).",
        primary.title.trim(),
        primary.candidate_kind.trim().replace('_', " "),
        evidence_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attention_engine_builds_explainable_cards_from_review_candidates() {
        let card = AttentionEngine::build_cards(&[task_candidate("review:item:task", 0.82)])
            .expect("build attention cards")
            .pop()
            .expect("one attention card");

        assert_eq!(card.id, "attention:review:review:item:task");
        assert_eq!(card.importance, AttentionImportance::High);
        assert_eq!(card.confidence, 0.82);
        assert_eq!(card.evidence_count, 1);
        assert_eq!(card.trace_id, "trace:task");
        assert_eq!(card.review_item_ids, vec!["review:item:task"]);
        assert_eq!(card.suggested_actions[0].action_kind, "promote");
        assert_eq!(
            card.explainability.why_this_matters,
            "Review contract remains in Review as potential task with 1 evidence source(s)."
        );
        assert_eq!(card.explainability.evidence.len(), 1);
        assert_eq!(card.explainability.related_objects.len(), 1);
    }

    #[test]
    fn attention_engine_groups_duplicate_review_items_and_preserves_all_evidence() {
        let first = task_candidate("review:item:first", 0.78).group_key("contract:deadline");
        let second = task_candidate("review:item:second", 0.91)
            .group_key("contract:deadline")
            .evidence(vec![
                AttentionEvidenceRef::new("observation:task:1"),
                AttentionEvidenceRef::new("observation:task:2").role("supporting"),
            ])
            .source_summary("Second Zulip message repeats the same deadline.");

        let cards =
            AttentionEngine::build_cards(&[first, second]).expect("build grouped attention cards");

        assert_eq!(cards.len(), 1);
        let card = &cards[0];
        assert_eq!(card.id, "attention:review:contract:deadline");
        assert_eq!(card.confidence, 0.91);
        assert_eq!(card.evidence_count, 2);
        assert_eq!(
            card.review_item_ids,
            vec!["review:item:first", "review:item:second"]
        );
        assert!(
            card.source_summary
                .contains("Grouped with 1 related review item(s).")
        );
    }

    #[test]
    fn attention_engine_rejects_cards_without_evidence() {
        let error = AttentionEngine::build_cards(&[AttentionCandidate::new(
            "review:item:missing-evidence",
            "potential_task",
            "Missing evidence",
            "This card should be rejected.",
            "trace:missing-evidence",
        )
        .confidence(0.8)
        .source_summary("No evidence source was supplied.")
        .suggested_actions(vec![AttentionSuggestedAction::new("dismiss", "Dismiss")])])
        .expect_err("missing evidence must be rejected");

        assert_eq!(
            error,
            AttentionEngineError::MissingEvidence("review:item:missing-evidence".to_owned())
        );
    }

    #[test]
    fn attention_engine_excludes_closed_review_items() {
        let cards = AttentionEngine::build_cards(&[
            task_candidate("review:item:active", 0.72),
            task_candidate("review:item:dismissed", 0.95).status("dismissed"),
        ])
        .expect("build attention cards");

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].review_item_ids, vec!["review:item:active"]);
    }

    fn task_candidate(review_item_id: &str, confidence: f64) -> AttentionCandidate {
        AttentionCandidate::new(
            review_item_id,
            "potential_task",
            "Review contract",
            "Potential action from a source-backed communication.",
            "trace:task",
        )
        .confidence(confidence)
        .evidence(vec![AttentionEvidenceRef::new("observation:task:1")])
        .related_entities(vec![
            AttentionRelatedEntity::new("organization", "org:v1:acme").label("Acme"),
        ])
        .source_summary("Zulip message asks for contract review.")
        .suggested_actions(vec![
            AttentionSuggestedAction::new("promote", "Create task").target("tasks", "task"),
            AttentionSuggestedAction::new("dismiss", "Dismiss"),
        ])
    }
}
