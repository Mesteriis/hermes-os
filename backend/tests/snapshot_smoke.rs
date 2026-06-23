use serde_json::json;

#[test]
fn event_payload_snapshot_remains_stable() {
    let payload = json!({
        "event_type": "signal.accepted.telegram.message",
        "source": {
            "kind": "integration",
            "provider": "telegram",
            "source_id": "msg-42"
        },
        "subject": {
            "kind": "communication",
            "entity_id": "thread-7"
        },
        "metadata": {
            "priority": "normal",
            "channel": "telegram"
        }
    });

    insta::assert_snapshot!(
        "event_payload_snapshot_remains_stable",
        serde_json::to_string_pretty(&payload).expect("snapshot payload must serialize"),
    );
}
