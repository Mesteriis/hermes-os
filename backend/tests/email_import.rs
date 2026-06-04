use chrono::{TimeZone, Utc};
use serde_json::json;

use hermes_hub_backend::email_sources::{FixtureEmailMessage, parse_fixture_email_messages};

#[test]
fn fixture_email_source_parses_account_scoped_messages() {
    let input = json!([
        {
            "provider_record_id": "gmail-msg-1",
            "subject": "Budget review",
            "from": "alice@example.com",
            "to": ["bob@example.com"],
            "sent_at": "2026-06-04T10:00:00Z",
            "body_text": "Please review the Q2 budget.",
            "source_fingerprint": "sha256:gmail-msg-1"
        }
    ])
    .to_string();

    let messages = parse_fixture_email_messages(&input).expect("parse fixture messages");

    assert_eq!(
        messages,
        vec![FixtureEmailMessage {
            provider_record_id: "gmail-msg-1".to_owned(),
            subject: "Budget review".to_owned(),
            from: "alice@example.com".to_owned(),
            to: vec!["bob@example.com".to_owned()],
            sent_at: Utc.with_ymd_and_hms(2026, 6, 4, 10, 0, 0).single(),
            body_text: "Please review the Q2 budget.".to_owned(),
            source_fingerprint: "sha256:gmail-msg-1".to_owned(),
        }]
    );
}
