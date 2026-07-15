use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use thiserror::Error;

use super::models::{ContextPackKind, ContextPackSourceKind, NewContextPack, NewContextPackSource};

const REVIEW_CONTEXT_PACK_OWNER: &str = "engines.context_packs.review";

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewContextPackInput {
    pub review_item: ReviewContextPackItem,
    pub evidence: Vec<ReviewContextPackEvidence>,
    pub trace_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewContextPackItem {
    pub review_item_id: String,
    pub item_kind: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub target_domain: Option<String>,
    pub target_entity_kind: Option<String>,
    pub target_entity_id: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewContextPackEvidence {
    pub observation_id: String,
    pub evidence_role: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewContextPackBuildResult {
    pub pack: NewContextPack,
    pub sources: Vec<NewContextPackSource>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ReviewContextPackBuildError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("review context pack evidence is required")]
    MissingEvidence,
}

pub fn build_review_context_pack(
    input: ReviewContextPackInput,
) -> Result<ReviewContextPackBuildResult, ReviewContextPackBuildError> {
    let ReviewContextPackInput {
        review_item,
        evidence,
        trace_id,
    } = input;

    let review_item_id = require_non_empty("review_item_id", &review_item.review_item_id)?;
    let title = require_non_empty("title", &review_item.title)?;
    let summary = require_non_empty("summary", &review_item.summary)?;
    let trace_id = require_non_empty("trace_id", &trace_id)?;
    let evidence = normalize_evidence(evidence)?;

    let content_evidence = evidence
        .iter()
        .map(|record| {
            json!({
                "observation_id": record.observation_id,
                "evidence_role": record.role,
                "metadata": record.metadata,
            })
        })
        .collect::<Vec<_>>();

    let mut timeline = Vec::with_capacity(evidence.len());
    let mut messages = Vec::new();
    let mut documents = Vec::new();
    let mut people = Vec::new();
    let mut organizations = Vec::new();

    for record in &evidence {
        timeline.push(review_timeline_entry(record));

        if let Some(message_context) = build_message_context(record) {
            messages.push(message_context);
        }

        if let Some(document_context) = build_document_context(record) {
            documents.push(document_context);
        }

        if let Some(person_context) = build_person_context(record) {
            people.push(person_context);
        }

        if let Some(organization_context) = build_organization_context(record) {
            organizations.push(organization_context);
        }
    }

    let timeline = dedup_values(timeline);
    let messages = dedup_values(messages);
    let documents = dedup_values(documents);
    let people = dedup_values(people);
    let organizations = dedup_values(organizations);

    let previous_reviews = item_previous_reviews(&review_item).unwrap_or_default();
    let suggested_actions = suggested_actions(&review_item.status);
    let content = json!({
        "summary": summary.clone(),
        "review_item": {
            "review_item_id": review_item_id.clone(),
            "item_kind": review_item.item_kind,
            "title": title,
            "summary": summary,
            "status": review_item.status,
            "target_domain": review_item.target_domain,
            "target_entity_kind": review_item.target_entity_kind,
            "target_entity_id": review_item.target_entity_id,
            "confidence": review_item.confidence,
            "metadata": review_item.metadata,
            "created_at": review_item.created_at,
            "updated_at": review_item.updated_at,
        },
        "timeline": timeline,
        "messages": messages,
        "documents": documents,
        "people": people,
        "organizations": organizations,
        "previous_reviews": previous_reviews,
        "open_tasks": collect_open_entities(&review_item, "task", "open_tasks"),
        "open_obligations": collect_open_entities(&review_item, "obligation", "open_obligations"),
        "trace": {
            "trace_id": trace_id.clone(),
        },
        "evidence": content_evidence,
        "suggested_actions": suggested_actions,
    });

    let pack = NewContextPack::new(ContextPackKind::Review, review_item_id.clone(), content)
        .metadata(json!({
            "owner": REVIEW_CONTEXT_PACK_OWNER,
            "review_item_id": review_item_id.clone(),
            "trace_id": trace_id.clone(),
        }))
        .rebuildable(true);

    let mut sources = Vec::with_capacity(evidence.len() + 1);
    sources.push(
        NewContextPackSource::new(ContextPackSourceKind::ReviewItem, review_item_id)
            .role("subject")
            .metadata(json!({
                "owner": REVIEW_CONTEXT_PACK_OWNER,
            })),
    );
    sources.extend(evidence.into_iter().map(|record| {
        NewContextPackSource::new(ContextPackSourceKind::Observation, record.observation_id)
            .role(record.role)
            .metadata(json!({
                "owner": REVIEW_CONTEXT_PACK_OWNER,
                "evidence_role": record.evidence_role,
                "review_item_evidence": record.metadata,
            }))
    }));

    Ok(ReviewContextPackBuildResult { pack, sources })
}

#[derive(Debug)]
struct NormalizedReviewContextPackEvidence {
    observation_id: String,
    evidence_role: String,
    role: String,
    metadata: Value,
}

fn normalize_evidence(
    evidence: Vec<ReviewContextPackEvidence>,
) -> Result<Vec<NormalizedReviewContextPackEvidence>, ReviewContextPackBuildError> {
    if evidence.is_empty() {
        return Err(ReviewContextPackBuildError::MissingEvidence);
    }

    evidence
        .into_iter()
        .map(|record| {
            let observation_id =
                require_non_empty("evidence_observation_id", &record.observation_id)?;
            let evidence_role = record.evidence_role.trim().to_owned();
            let role = if evidence_role.is_empty() {
                "evidence".to_owned()
            } else {
                evidence_role.clone()
            };

            Ok(NormalizedReviewContextPackEvidence {
                observation_id,
                evidence_role: role.clone(),
                role,
                metadata: record.metadata,
            })
        })
        .collect()
}

fn require_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, ReviewContextPackBuildError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ReviewContextPackBuildError::EmptyField(field_name));
    }

    Ok(value.to_owned())
}

fn suggested_actions(status: &str) -> Value {
    match status {
        "new" => json!([
            {"action": "take_review", "label": "Take review"},
            {"action": "approve", "label": "Approve"},
            {"action": "dismiss", "label": "Dismiss"},
        ]),
        "in_review" => json!([
            {"action": "approve", "label": "Approve"},
            {"action": "dismiss", "label": "Dismiss"},
        ]),
        "approved" => json!([
            {"action": "promote", "label": "Promote"},
            {"action": "dismiss", "label": "Dismiss"},
        ]),
        "promoted" | "dismissed" | "archived" => {
            json!([{"action": "archive", "label": "Archive"}])
        }
        _ => json!([]),
    }
}

fn review_timeline_entry(record: &NormalizedReviewContextPackEvidence) -> Value {
    json!({
        "observation_id": record.observation_id,
        "role": record.role,
        "evidence_role": record.evidence_role,
    })
}

fn collect_open_entities(
    review_item: &ReviewContextPackItem,
    entity_kind: &str,
    section: &str,
) -> Vec<Value> {
    let Some(target_domain) = review_item.target_domain.as_ref() else {
        return Vec::new();
    };
    let Some(target_entity_id) = review_item.target_entity_id.as_ref() else {
        return Vec::new();
    };

    let target_matches =
        target_domain == entity_kind || target_domain == format!("{entity_kind}s").as_str();
    if !target_matches {
        return Vec::new();
    }

    vec![json!({
        "entity_kind": entity_kind,
        "entity_id": target_entity_id,
        "status": "candidate",
        "source": section,
        "metadata": json!({
            "target_domain": target_domain,
        }),
    })]
}

fn item_previous_reviews(item: &ReviewContextPackItem) -> Option<Vec<Value>> {
    let Value::Array(values) = item.metadata.get("previous_reviews")? else {
        return None;
    };

    let previous_reviews = values
        .iter()
        .filter_map(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    if previous_reviews.is_empty() {
        return None;
    }

    Some(
        previous_reviews
            .into_iter()
            .map(|review_id| {
                json!({
                    "review_item_id": review_id,
                    "type": "review",
                    "metadata": json!({}),
                })
            })
            .collect(),
    )
}

fn build_message_context(record: &NormalizedReviewContextPackEvidence) -> Option<Value> {
    let message_id = first_string(
        &record.metadata,
        &[
            "message_id",
            "thread_id",
            "channel_message_id",
            "message_ref",
        ],
    );
    let title = first_string(
        &record.metadata,
        &["title", "subject", "summary", "headline"],
    );
    let body = first_string(&record.metadata, &["body", "text", "content", "excerpt"]);

    if message_id.is_none() && title.is_none() && body.is_none() {
        return None;
    }

    Some(json!({
        "observation_id": record.observation_id,
        "message_id": message_id,
        "role": record.role,
        "evidence_role": record.evidence_role,
        "title": title,
        "body": body,
    }))
}

fn build_document_context(record: &NormalizedReviewContextPackEvidence) -> Option<Value> {
    let document_id = first_string(&record.metadata, &["document_id", "doc_id", "document_ref"])?;

    let title = first_string(
        &record.metadata,
        &["document_title", "title", "name", "summary"],
    );

    Some(json!({
        "observation_id": record.observation_id,
        "document_id": document_id,
        "role": record.role,
        "evidence_role": record.evidence_role,
        "title": title,
    }))
}

fn build_person_context(record: &NormalizedReviewContextPackEvidence) -> Option<Value> {
    let legacy_persona_id_key = ["person", "id"].join("_");
    let persona_id = first_string(&record.metadata, &["persona_id", "author_id", "actor_id"])
        .or_else(|| first_string(&record.metadata, &[legacy_persona_id_key.as_str()]))?;

    let name = first_string(&record.metadata, &["person_name", "author_name", "name"]);

    Some(json!({
        "observation_id": record.observation_id,
        "persona_id": persona_id,
        "role": record.role,
        "evidence_role": record.evidence_role,
        "name": name,
    }))
}

fn build_organization_context(record: &NormalizedReviewContextPackEvidence) -> Option<Value> {
    let organization_id = first_string(
        &record.metadata,
        &["organization_id", "org_id", "organization_ref"],
    )?;

    let name = first_string(
        &record.metadata,
        &["organization_name", "org_name", "organization", "name"],
    );

    Some(json!({
        "observation_id": record.observation_id,
        "organization_id": organization_id,
        "role": record.role,
        "evidence_role": record.evidence_role,
        "name": name,
    }))
}

fn dedup_values(values: Vec<Value>) -> Vec<Value> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.iter().any(|item| item == &value) {
            unique.push(value);
        }
    }
    unique
}

fn first_string(metadata: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = metadata.get(key).and_then(Value::as_str) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    use super::*;

    #[test]
    fn review_context_pack_builder_rejects_missing_or_empty_evidence() {
        let error = build_review_context_pack(ReviewContextPackInput {
            review_item: review_item(),
            evidence: Vec::new(),
            trace_id: "trace:v1:missing-evidence".to_owned(),
        })
        .expect_err("review context pack without evidence must fail");

        assert_eq!(error, ReviewContextPackBuildError::MissingEvidence);

        let error = build_review_context_pack(ReviewContextPackInput {
            review_item: review_item(),
            evidence: vec![ReviewContextPackEvidence {
                observation_id: "  ".to_owned(),
                evidence_role: "primary".to_owned(),
                metadata: json!({}),
            }],
            trace_id: "trace:v1:empty-evidence".to_owned(),
        })
        .expect_err("review context pack with empty evidence observation id must fail");

        assert_eq!(
            error,
            ReviewContextPackBuildError::EmptyField("evidence_observation_id")
        );
    }

    #[test]
    fn review_context_pack_builder_rejects_empty_required_fields() {
        let mut input = review_context_pack_input();
        input.review_item.review_item_id = " ".to_owned();
        assert_empty_field(input, "review_item_id");

        let mut input = review_context_pack_input();
        input.review_item.title = " ".to_owned();
        assert_empty_field(input, "title");

        let mut input = review_context_pack_input();
        input.review_item.summary = " ".to_owned();
        assert_empty_field(input, "summary");

        let mut input = review_context_pack_input();
        input.trace_id = " ".to_owned();
        assert_empty_field(input, "trace_id");
    }

    #[test]
    fn review_context_pack_builder_preserves_review_trace_and_sources() {
        let result = build_review_context_pack(review_context_pack_input())
            .expect("build review context pack");

        assert_eq!(result.pack.kind, ContextPackKind::Review);
        assert_eq!(result.pack.subject_id, "review_item:v1:test");
        assert!(result.pack.rebuildable);
        assert_eq!(
            result.pack.metadata["owner"],
            json!("engines.context_packs.review")
        );
        for key in [
            "summary",
            "review_item",
            "timeline",
            "messages",
            "documents",
            "people",
            "organizations",
            "previous_reviews",
            "open_tasks",
            "open_obligations",
            "trace",
            "evidence",
            "suggested_actions",
        ] {
            assert!(
                result.pack.content.get(key).is_some(),
                "{key} must be present in review context pack content"
            );
        }
        assert_eq!(
            result.pack.content["summary"],
            json!("Evidence-backed summary.")
        );
        assert_eq!(
            result.pack.content["review_item"]["review_item_id"],
            json!("review_item:v1:test")
        );
        assert_eq!(
            result.pack.content["trace"]["trace_id"],
            json!("trace:v1:review-context")
        );
        assert!(
            !result.pack.content["timeline"]
                .as_array()
                .expect("timeline array")
                .is_empty(),
            "timeline must include evidence-derived context"
        );
        assert!(
            !result.pack.content["messages"]
                .as_array()
                .expect("messages array")
                .is_empty(),
            "messages must include evidence-derived context"
        );
        assert!(
            !result.pack.content["documents"]
                .as_array()
                .expect("documents array")
                .is_empty(),
            "documents must include evidence-derived context"
        );
        assert!(
            !result.pack.content["people"]
                .as_array()
                .expect("people array")
                .is_empty(),
            "people must include evidence-derived context"
        );
        assert!(
            !result.pack.content["organizations"]
                .as_array()
                .expect("organizations array")
                .is_empty(),
            "organizations must include evidence-derived context"
        );
        assert_eq!(
            result.pack.content["previous_reviews"]
                .as_array()
                .expect("previous_reviews array"),
            &Vec::<serde_json::Value>::new(),
        );
        assert_eq!(
            result.pack.content["open_tasks"]
                .as_array()
                .expect("open_tasks array"),
            &Vec::<serde_json::Value>::new(),
        );
        assert_eq!(
            result.pack.content["open_obligations"]
                .as_array()
                .expect("open_obligations array"),
            &Vec::<serde_json::Value>::new(),
        );

        let evidence = result.pack.content["evidence"]
            .as_array()
            .expect("evidence array");
        let evidence_ids = evidence
            .iter()
            .map(|record| record["observation_id"].as_str().expect("observation id"))
            .collect::<Vec<_>>();
        assert_eq!(
            evidence_ids,
            vec!["observation:v1:first", "observation:v1:second"]
        );

        assert_eq!(result.sources.len(), 3);
        assert!(result.sources.iter().any(|source| {
            source.source_kind == ContextPackSourceKind::ReviewItem
                && source.source_id == "review_item:v1:test"
                && source.role == "subject"
        }));
        assert!(result.sources.iter().any(|source| {
            source.source_kind == ContextPackSourceKind::Observation
                && source.source_id == "observation:v1:first"
                && source.role == "primary"
        }));
        assert!(result.sources.iter().any(|source| {
            source.source_kind == ContextPackSourceKind::Observation
                && source.source_id == "observation:v1:second"
                && source.role == "evidence"
        }));
    }

    #[test]
    fn person_context_reads_legacy_persona_identity_only_as_the_last_fallback() {
        let legacy_persona_id_key = ["person", "id"].join("_");
        let mut legacy_metadata = serde_json::Map::new();
        legacy_metadata.insert(
            legacy_persona_id_key.clone(),
            Value::String("persona:legacy".to_owned()),
        );

        let legacy_context =
            build_person_context(&normalized_evidence(Value::Object(legacy_metadata.clone())))
                .expect("legacy persona identity must remain readable");
        assert_eq!(legacy_context["persona_id"], json!("persona:legacy"));

        legacy_metadata.insert(
            "actor_id".to_owned(),
            Value::String("persona:actor".to_owned()),
        );
        let actor_context =
            build_person_context(&normalized_evidence(Value::Object(legacy_metadata.clone())))
                .expect("actor identity must be readable");
        assert_eq!(actor_context["persona_id"], json!("persona:actor"));

        legacy_metadata.insert(
            "persona_id".to_owned(),
            Value::String("persona:canonical".to_owned()),
        );
        let canonical_context =
            build_person_context(&normalized_evidence(Value::Object(legacy_metadata)))
                .expect("canonical persona identity must be readable");
        assert_eq!(canonical_context["persona_id"], json!("persona:canonical"));
    }

    fn normalized_evidence(metadata: Value) -> NormalizedReviewContextPackEvidence {
        NormalizedReviewContextPackEvidence {
            observation_id: "observation:v1:persona".to_owned(),
            evidence_role: "primary".to_owned(),
            role: "primary".to_owned(),
            metadata,
        }
    }

    fn assert_empty_field(input: ReviewContextPackInput, field_name: &'static str) {
        let error = build_review_context_pack(input)
            .expect_err("empty required field must fail validation");
        assert_eq!(error, ReviewContextPackBuildError::EmptyField(field_name));
    }

    fn review_context_pack_input() -> ReviewContextPackInput {
        ReviewContextPackInput {
            review_item: review_item(),
            evidence: vec![
                ReviewContextPackEvidence {
                    observation_id: "observation:v1:first".to_owned(),
                    evidence_role: "primary".to_owned(),
                    metadata: json!({
                        "message_id": "message:v1:first",
                        "title": "Potential task in review",
                        "body": "First evidence message body.",
                        "persona_id": "persona:first",
                        "person_name": "Alice",
                        "organization_id": "org:v1:first",
                        "organization_name": "Acme Labs",
                    }),
                },
                ReviewContextPackEvidence {
                    observation_id: "observation:v1:second".to_owned(),
                    evidence_role: "  ".to_owned(),
                    metadata: json!({
                        "document_id": "doc:v1:context",
                        "document_title": "Decision Log",
                        "summary": "Second evidence supports document linkage.",
                    }),
                },
            ],
            trace_id: "trace:v1:review-context".to_owned(),
        }
    }

    fn review_item() -> ReviewContextPackItem {
        let timestamp = Utc.with_ymd_and_hms(2026, 6, 18, 12, 0, 0).unwrap();
        ReviewContextPackItem {
            review_item_id: "review_item:v1:test".to_owned(),
            item_kind: "potential_task".to_owned(),
            title: "Review context test".to_owned(),
            summary: "Evidence-backed summary.".to_owned(),
            status: "new".to_owned(),
            target_domain: None,
            target_entity_kind: None,
            target_entity_id: None,
            confidence: 0.87,
            metadata: json!({"attention_group_key": "review-context"}),
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
}
