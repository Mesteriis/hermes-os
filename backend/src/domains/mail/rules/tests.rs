use chrono::Utc;
use serde_json::json;

use crate::domains::mail::messages::{LocalMessageState, ProjectedMessage, WorkflowState};

use super::evaluation::evaluate_conditions;
use super::mode::RuleMode;

fn test_message(subject: &str, sender: &str, body: &str) -> ProjectedMessage {
    ProjectedMessage {
        message_id: "msg:test:1".into(),
        raw_record_id: "raw:1".into(),
        account_id: "acct:1".into(),
        provider_record_id: "prov:1".into(),
        subject: subject.into(),
        sender: sender.into(),
        recipients: vec!["to@ex.com".into()],
        body_text: body.into(),
        occurred_at: Some(Utc::now()),
        projected_at: Utc::now(),
        channel_kind: "email".into(),
        conversation_id: None,
        sender_display_name: None,
        delivery_state: "received".into(),
        message_metadata: json!({}),
        workflow_state: WorkflowState::New,
        importance_score: None,
        ai_category: None,
        ai_summary: None,
        ai_summary_generated_at: None,
        local_state: LocalMessageState::Active,
        local_state_changed_at: None,
        local_state_reason: None,
    }
}

#[test]
fn evaluate_conditions_matches_subject() {
    let msg = test_message("Urgent: Project Update", "alice@ex.com", "Body text");
    let conditions = json!([
        {"field": "subject", "operator": "contains", "value": "urgent", "label": "urgent subject"}
    ]);
    let matched = evaluate_conditions(&conditions, &msg);
    assert_eq!(matched, vec!["urgent subject"]);
}

#[test]
fn evaluate_conditions_matches_sender() {
    let msg = test_message("Hello", "bob@company.com", "Body");
    let conditions = json!([
        {"field": "sender", "operator": "contains", "value": "company.com", "label": "company sender"}
    ]);
    let matched = evaluate_conditions(&conditions, &msg);
    assert_eq!(matched, vec!["company sender"]);
}

#[test]
fn evaluate_conditions_no_match() {
    let msg = test_message("Regular", "alice@ex.com", "Nothing special");
    let conditions = json!([
        {"field": "subject", "operator": "contains", "value": "urgent", "label": "urgent"}
    ]);
    let matched = evaluate_conditions(&conditions, &msg);
    assert!(matched.is_empty());
}

#[test]
fn rule_mode_parse_all() {
    assert_eq!(RuleMode::parse("suggest"), Some(RuleMode::Suggest));
    assert_eq!(RuleMode::parse("auto_execute"), Some(RuleMode::AutoExecute));
    assert_eq!(RuleMode::parse("dry_run"), Some(RuleMode::DryRun));
    assert_eq!(RuleMode::parse("invalid"), None);
}
