use chrono::{DateTime, Utc};
use serde_json::Value;

pub fn supports_observation_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.zulip.message"
            | "signal.accepted.zulip.reaction"
            | "signal.accepted.zulip.message_update"
            | "signal.accepted.zulip.message_delete"
    )
}

pub fn command_kinds_for_observation(event_type: &str, payload: &Value) -> Vec<&'static str> {
    match event_type {
        "signal.accepted.zulip.message" => vec![
            "send_stream_message",
            "send_direct_message",
            "send_stream_message_with_upload",
            "send_direct_message_with_upload",
        ],
        "signal.accepted.zulip.message_update" => vec!["update_message"],
        "signal.accepted.zulip.message_delete" => vec!["delete_message"],
        "signal.accepted.zulip.reaction" => match payload
            .get("reaction_op")
            .and_then(Value::as_str)
            .map(str::trim)
        {
            Some("add") => vec!["add_reaction"],
            Some("remove") => vec!["remove_reaction"],
            _ => vec!["add_reaction", "remove_reaction"],
        },
        _ => Vec::new(),
    }
}

pub fn provider_state(
    accepted_event_id: &str,
    accepted_event_type: &str,
    raw_record_id: &str,
    raw_payload: &Value,
    observed_at: DateTime<Utc>,
) -> Value {
    serde_json::json!({
        "provider": "zulip",
        "observed_via": "signal_hub_accepted_event",
        "accepted_event_id": accepted_event_id,
        "accepted_event_type": accepted_event_type,
        "raw_record_id": raw_record_id,
        "provider_event_id": raw_payload.get("provider_event_id").cloned().unwrap_or(Value::Null),
        "provider_event_type": raw_payload.get("provider_event_type").cloned().unwrap_or(Value::Null),
        "provider_message_id": raw_payload.get("provider_message_id").cloned().unwrap_or(Value::Null),
        "provider_observed_at": observed_at,
    })
}

#[cfg(test)]
mod tests {
    use super::{command_kinds_for_observation, provider_state, supports_observation_event};
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn accepts_only_supported_zulip_observation_events() {
        assert!(supports_observation_event("signal.accepted.zulip.message"));
        assert!(supports_observation_event("signal.accepted.zulip.reaction"));
        assert!(!supports_observation_event(
            "signal.accepted.telegram.message"
        ));
    }

    #[test]
    fn maps_reaction_operation_to_exact_command_kind() {
        assert_eq!(
            command_kinds_for_observation(
                "signal.accepted.zulip.reaction",
                &json!({"reaction_op": "remove"}),
            ),
            vec!["remove_reaction"]
        );
        assert_eq!(
            command_kinds_for_observation("signal.accepted.zulip.reaction", &json!({})),
            vec!["add_reaction", "remove_reaction"]
        );
    }

    #[test]
    fn provider_state_keeps_provenance_without_secret_content() {
        let observed_at = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let state = provider_state(
            "accepted-1",
            "signal.accepted.zulip.message",
            "raw-1",
            &json!({
                "provider_event_id": "event-1",
                "provider_message_id": "message-1",
                "secret": "must-not-be-copied",
            }),
            observed_at,
        );

        assert_eq!(state["provider"], "zulip");
        assert_eq!(state["provider_event_id"], "event-1");
        assert_eq!(state["provider_message_id"], "message-1");
        assert!(state.get("secret").is_none());
    }
}
