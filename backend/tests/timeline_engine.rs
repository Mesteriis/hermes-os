use chrono::Utc;
use hermes_hub_backend::engines::timeline::{TimelineEngine, TimelineEventDraft};

#[test]
fn timeline_engine_bounds_entity_timeline_limits() {
    assert_eq!(TimelineEngine::bounded_entity_limit(0), 1);
    assert_eq!(TimelineEngine::bounded_entity_limit(25), 25);
    assert_eq!(TimelineEngine::bounded_entity_limit(250), 100);
}

#[test]
fn timeline_engine_rejects_unsourced_timeline_event() {
    let draft = TimelineEventDraft {
        entity_kind: "persona",
        entity_id: "persona:v1:human:alice",
        event_type: "first_message",
        title: "First message",
        occurred_at: Utc::now(),
        source: " ",
    };

    let error = TimelineEngine::validate_event(&draft).expect_err("event should be rejected");

    assert_eq!(error.to_string(), "timeline event source must not be empty");
}

#[test]
fn timeline_engine_accepts_source_backed_timeline_event() {
    let draft = TimelineEventDraft {
        entity_kind: "persona",
        entity_id: "persona:v1:human:alice",
        event_type: "first_message",
        title: "First message",
        occurred_at: Utc::now(),
        source: "communication_messages:message-1",
    };

    TimelineEngine::validate_event(&draft).expect("source-backed event should be valid");
}
