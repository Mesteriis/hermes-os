use chrono::{DateTime, Utc};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use hermes_provider_api::{
    ProviderContractError, ProviderId, ProviderObservationEnvelope, ProviderObservationInput,
};

use crate::models::ZulipEvent;

pub mod zulip_raw_signal_event_types {
    pub const MESSAGE: &str = "signal.raw.zulip.message.observed";
    pub const REACTION: &str = "signal.raw.zulip.reaction.observed";
    pub const MESSAGE_UPDATE: &str = "signal.raw.zulip.message_update.observed";
    pub const MESSAGE_DELETE: &str = "signal.raw.zulip.message_delete.observed";
    pub const UNKNOWN: &str = "signal.raw.zulip.unknown.observed";
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipEventMappingContext {
    pub account_id: String,
    pub realm_url: String,
    pub received_at: DateTime<Utc>,
    pub import_batch_id: String,
    pub lab_correlation_id: Option<String>,
    pub scenario_run_id: Option<String>,
}

impl ZulipEventMappingContext {
    pub fn new(
        account_id: impl Into<String>,
        realm_url: impl Into<String>,
        received_at: DateTime<Utc>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            realm_url: realm_url.into(),
            received_at,
            import_batch_id: "zulip-event-queue".to_owned(),
            lab_correlation_id: None,
            scenario_run_id: None,
        }
    }

    pub fn with_import_batch_id(mut self, import_batch_id: impl Into<String>) -> Self {
        self.import_batch_id = import_batch_id.into();
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.lab_correlation_id = Some(correlation_id.into());
        self
    }

    pub fn with_scenario_id(mut self, scenario_run_id: impl Into<String>) -> Self {
        self.scenario_run_id = Some(scenario_run_id.into());
        self
    }
}

pub fn map_zulip_event_to_raw_record(
    event: &ZulipEvent,
    context: &ZulipEventMappingContext,
) -> Result<ProviderObservationEnvelope, ZulipEventMappingError> {
    let provider_record_id = provider_record_id(event);
    let provider_message_id = provider_message_id(event);
    let source_fingerprint = source_fingerprint(context, event, &provider_record_id);
    let mut payload = json!({
        "provider": "zulip",
        "provider_event_id": event.id,
        "provider_event_type": event.event_type,
        "provider_message_id": provider_message_id,
        "delivery_state": "received",
        "raw_event": event.data,
    });
    copy_message_field(event, &mut payload, "stream_id", &["stream_id"]);
    copy_message_field(
        event,
        &mut payload,
        "stream_name",
        &["display_recipient", "stream"],
    );
    copy_message_field(event, &mut payload, "topic", &["topic", "subject"]);
    copy_message_field(event, &mut payload, "sender_email", &["sender_email"]);
    copy_message_field(
        event,
        &mut payload,
        "sender_full_name",
        &["sender_full_name", "sender"],
    );
    copy_message_field(
        event,
        &mut payload,
        "message_type",
        &["type", "message_type"],
    );
    copy_message_field(event, &mut payload, "content", &["content"]);
    copy_message_direct_recipients(event, &mut payload);
    copy_event_field(event, &mut payload, "stream_id", &["stream_id"]);
    copy_event_field(event, &mut payload, "stream_name", &["stream"]);
    copy_event_field(event, &mut payload, "topic", &["topic", "subject"]);
    copy_event_field(event, &mut payload, "content", &["content"]);
    copy_event_field(event, &mut payload, "prev_content", &["prev_content"]);
    copy_event_field(event, &mut payload, "prev_topic", &["prev_topic"]);
    copy_event_field(event, &mut payload, "edit_timestamp", &["edit_timestamp"]);
    copy_event_field(event, &mut payload, "message_type", &["message_type"]);
    copy_event_field(event, &mut payload, "emoji_name", &["emoji_name"]);
    copy_event_field(event, &mut payload, "emoji_code", &["emoji_code"]);
    copy_event_field(event, &mut payload, "reaction_type", &["reaction_type"]);
    copy_event_field(event, &mut payload, "reaction_op", &["op"]);
    copy_event_field(event, &mut payload, "provider_actor_id", &["user_id"]);
    copy_event_field(
        event,
        &mut payload,
        "sender_display_name",
        &["sender_full_name", "sender"],
    );
    copy_event_field(event, &mut payload, "sender_email", &["sender_email"]);
    copy_user_field(event, &mut payload, "provider_actor_id", &["id", "user_id"]);
    copy_user_field(
        event,
        &mut payload,
        "sender_display_name",
        &["full_name", "name"],
    );
    copy_user_field(event, &mut payload, "sender_email", &["email"]);
    copy_message_attachments(event, &mut payload);

    let mut observation = ProviderObservationInput::new(
        raw_record_id(&source_fingerprint),
        ProviderId::parse("zulip")?,
        context.account_id.trim(),
        zulip_raw_record_kind(&event.event_type),
        provider_record_id,
        source_fingerprint,
        context.import_batch_id.trim(),
        context.received_at,
        context.received_at,
        event.id.to_string(),
        payload,
        json!({
            "provider": "zulip",
            "provider_kind": "zulip_bot",
            "account_id": context.account_id,
            "realm_url": context.realm_url,
            "provider_event_id": event.id,
            "provider_event_type": event.event_type,
            "provider_message_id": provider_message_id,
            "lab_correlation_id": context.lab_correlation_id,
            "scenario_run_id": context.scenario_run_id,
        }),
    );
    if let Some(correlation_id) = &context.lab_correlation_id {
        observation = observation.with_correlation_id(correlation_id);
    }

    ProviderObservationEnvelope::try_from(observation).map_err(ZulipEventMappingError::Contract)
}

pub fn map_zulip_event_to_observation(
    event: &ZulipEvent,
    context: &ZulipEventMappingContext,
) -> Result<ProviderObservationEnvelope, ZulipEventMappingError> {
    map_zulip_event_to_raw_record(event, context)
}

pub fn zulip_raw_signal_event_type(provider_event_type: &str) -> &'static str {
    match provider_event_type {
        "message" => zulip_raw_signal_event_types::MESSAGE,
        "reaction" => zulip_raw_signal_event_types::REACTION,
        "update_message" => zulip_raw_signal_event_types::MESSAGE_UPDATE,
        "delete_message" => zulip_raw_signal_event_types::MESSAGE_DELETE,
        _ => zulip_raw_signal_event_types::UNKNOWN,
    }
}

pub fn zulip_raw_record_kind(provider_event_type: &str) -> &'static str {
    match provider_event_type {
        "message" => "zulip_message",
        "reaction" => "zulip_reaction",
        "update_message" => "zulip_message_update",
        "delete_message" => "zulip_message_delete",
        _ => "zulip_unknown_event",
    }
}

fn raw_record_id(source_fingerprint: &str) -> String {
    format!(
        "raw_zulip_{}",
        source_fingerprint
            .strip_prefix("sha256:")
            .unwrap_or(source_fingerprint)
    )
}

fn source_fingerprint(
    context: &ZulipEventMappingContext,
    event: &ZulipEvent,
    provider_record_id: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(context.account_id.trim().as_bytes());
    hasher.update(b"\0");
    hasher.update(event.id.to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(event.event_type.trim().as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_record_id.trim().as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn provider_record_id(event: &ZulipEvent) -> String {
    if event.event_type == "message" {
        return provider_message_id(event).unwrap_or_else(|| event.id.to_string());
    }

    format!("{}:{}", event.event_type.trim(), event.id)
}

fn provider_message_id(event: &ZulipEvent) -> Option<String> {
    message_value(event, &["id"])
        .or_else(|| event.data.get("message_id"))
        .and_then(value_to_string)
}

fn copy_message_field(event: &ZulipEvent, payload: &mut Value, target: &str, fields: &[&str]) {
    let Some(value) = message_value(event, fields).and_then(value_to_string) else {
        return;
    };
    payload[target] = json!(value);
}

fn copy_event_field(event: &ZulipEvent, payload: &mut Value, target: &str, fields: &[&str]) {
    if payload.get(target).is_some() {
        return;
    }
    let Some(value) = find_first(&event.data, fields).and_then(value_to_string) else {
        return;
    };
    payload[target] = json!(value);
}

fn copy_user_field(event: &ZulipEvent, payload: &mut Value, target: &str, fields: &[&str]) {
    if payload.get(target).is_some() {
        return;
    }
    let Some(user) = event.data.get("user").and_then(Value::as_object) else {
        return;
    };
    let Some(value) = find_first(user, fields).and_then(value_to_string) else {
        return;
    };
    payload[target] = json!(value);
}

fn copy_message_attachments(event: &ZulipEvent, payload: &mut Value) {
    let attachments = if let Some(attachments) =
        message_value(event, &["attachments"]).and_then(Value::as_array)
    {
        attachments
            .iter()
            .filter_map(zulip_attachment_metadata)
            .collect::<Vec<_>>()
    } else {
        message_value(event, &["content"])
            .and_then(Value::as_str)
            .map(zulip_attachment_metadata_from_content)
            .unwrap_or_default()
    };
    if attachments.is_empty() {
        return;
    }

    payload["attachments"] = Value::Array(attachments);
    payload["attachment_state"] = json!({
        "state": "metadata_only",
        "bytes_state": "not_transferred",
        "scan_status": "not_scanned",
        "materialization_state": "not_materialized",
        "reason": "zulip_attachment_bytes_not_transferred"
    });
}

fn zulip_attachment_metadata_from_content(content: &str) -> Vec<Value> {
    user_upload_paths(content)
        .into_iter()
        .map(zulip_attachment_metadata_from_user_upload_path)
        .collect()
}

fn user_upload_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut search_from = 0;
    while let Some(relative_start) = content[search_from..].find("/user_uploads/") {
        let start = search_from + relative_start;
        let tail = &content[start..];
        let end = tail
            .char_indices()
            .find_map(|(index, character)| {
                matches!(
                    character,
                    '"' | '\'' | ')' | '<' | '>' | ' ' | '\n' | '\r' | '\t'
                )
                .then_some(index)
            })
            .unwrap_or(tail.len());
        let path = &tail[..end];
        if !paths.iter().any(|existing| existing == path) {
            paths.push(path.to_owned());
        }
        search_from = start + end.max("/user_uploads/".len());
        if search_from >= content.len() {
            break;
        }
    }
    paths
}

fn zulip_attachment_metadata_from_user_upload_path(path: String) -> Value {
    let path_id = path
        .strip_prefix("/user_uploads/")
        .unwrap_or(path.as_str())
        .trim_matches('/')
        .to_owned();
    let filename = path_id
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("zulip-upload")
        .to_owned();
    json!({
        "provider": "zulip",
        "provider_attachment_id": path_id.clone(),
        "bytes_state": "not_transferred",
        "scan_status": "not_scanned",
        "materialization_state": "not_materialized",
        "filename": filename,
        "url": path.clone(),
        "path_id": path_id,
    })
}

fn copy_message_direct_recipients(event: &ZulipEvent, payload: &mut Value) {
    let Some(display_recipient) = message_value(event, &["display_recipient"]) else {
        return;
    };
    let Some(recipients) = zulip_direct_recipients(display_recipient) else {
        return;
    };
    if recipients.is_empty() {
        return;
    }

    payload["direct_recipients"] = Value::Array(recipients);
}

fn zulip_direct_recipients(value: &Value) -> Option<Vec<Value>> {
    let values = value.as_array()?;
    Some(
        values
            .iter()
            .filter_map(zulip_direct_recipient_metadata)
            .collect(),
    )
}

fn zulip_direct_recipient_metadata(value: &Value) -> Option<Value> {
    match value {
        Value::String(value) => {
            let value = value.trim();
            if value.is_empty() {
                return None;
            }
            Some(json!({ "display_name": value }))
        }
        Value::Object(object) => {
            let mut metadata = Map::new();
            if let Some(email) = attachment_string_field(object, &["email"]) {
                metadata.insert("email".to_owned(), json!(email));
            }
            if let Some(full_name) = attachment_string_field(object, &["full_name", "name"]) {
                metadata.insert("full_name".to_owned(), json!(full_name));
            }
            if let Some(user_id) = find_first(object, &["id", "user_id"]).and_then(value_to_string)
            {
                metadata.insert("provider_user_id".to_owned(), json!(user_id));
            }
            (!metadata.is_empty()).then_some(Value::Object(metadata))
        }
        _ => None,
    }
}

fn zulip_attachment_metadata(value: &Value) -> Option<Value> {
    let object = value.as_object()?;
    let provider_attachment_id = attachment_string_field(object, &["id", "path_id", "url"])
        .unwrap_or_else(|| attachment_fingerprint(value));
    let mut metadata = Map::new();
    metadata.insert("provider".to_owned(), json!("zulip"));
    metadata.insert(
        "provider_attachment_id".to_owned(),
        json!(provider_attachment_id),
    );
    metadata.insert("bytes_state".to_owned(), json!("not_transferred"));
    metadata.insert("scan_status".to_owned(), json!("not_scanned"));
    metadata.insert(
        "materialization_state".to_owned(),
        json!("not_materialized"),
    );

    if let Some(filename) = attachment_string_field(object, &["name", "filename"]) {
        metadata.insert("filename".to_owned(), json!(filename));
    }
    if let Some(url) = attachment_string_field(object, &["url"]) {
        metadata.insert("url".to_owned(), json!(url));
    }
    if let Some(path_id) = attachment_string_field(object, &["path_id"]) {
        metadata.insert("path_id".to_owned(), json!(path_id));
    }
    if let Some(content_type) = attachment_string_field(object, &["content_type", "mime_type"]) {
        metadata.insert("content_type".to_owned(), json!(content_type));
    }
    if let Some(size_bytes) = attachment_size_bytes(object, &["size_bytes", "size"]) {
        metadata.insert("size_bytes".to_owned(), json!(size_bytes));
    }

    Some(Value::Object(metadata))
}

fn attachment_string_field(object: &Map<String, Value>, fields: &[&str]) -> Option<String> {
    find_first(object, fields).and_then(value_to_string)
}

fn attachment_size_bytes(object: &Map<String, Value>, fields: &[&str]) -> Option<u64> {
    fields.iter().find_map(|field| {
        let value = object.get(*field)?;
        match value {
            Value::Number(number) => number
                .as_u64()
                .or_else(|| number.as_i64().and_then(|value| u64::try_from(value).ok())),
            Value::String(value) => value.trim().parse::<u64>().ok(),
            _ => None,
        }
    })
}

fn attachment_fingerprint(value: &Value) -> String {
    let mut hasher = Sha256::new();
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    hasher.update(bytes);
    format!("zulip-attachment-sha256:{:x}", hasher.finalize())
}

fn message_value<'a>(event: &'a ZulipEvent, fields: &[&str]) -> Option<&'a Value> {
    let message = event.data.get("message")?.as_object()?;
    find_first(message, fields)
}

fn find_first<'a>(message: &'a Map<String, Value>, fields: &[&str]) -> Option<&'a Value> {
    fields.iter().find_map(|field| message.get(*field))
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_owned())
        }
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

#[derive(Debug, Error)]
pub enum ZulipEventMappingError {
    #[error("invalid Zulip raw event mapping: {0}")]
    Invalid(String),
    #[error("invalid Zulip provider observation: {0}")]
    Contract(#[from] ProviderContractError),
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde_json::json;

    use super::*;

    #[test]
    fn maps_message_event_to_raw_signal_ready_record() {
        let event: ZulipEvent = serde_json::from_value(json!({
            "id": 42,
            "type": "message",
            "message": {
                "id": 7001,
                "content": "Надо проверить backup retention до пятницы.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Hermes Bot",
                "stream_id": 10,
                "display_recipient": "Hermes Lab",
                "subject": "Tasks"
            }
        }))
        .expect("valid Zulip event fixture");
        let received_at = DateTime::parse_from_rfc3339("2026-06-29T08:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc);
        let context = ZulipEventMappingContext::new(
            "lab-zulip-account",
            "http://localhost:8080",
            received_at,
        )
        .with_import_batch_id("zulip-lab-batch")
        .with_correlation_id("lab-zulip-task-001")
        .with_scenario_id("zulip-message-to-task-run");

        let raw_record = map_zulip_event_to_raw_record(&event, &context)
            .expect("message event maps to raw record");

        assert_eq!(raw_record.account_id, "lab-zulip-account");
        assert_eq!(raw_record.record_kind, "zulip_message");
        assert_eq!(raw_record.provider_record_id, "7001");
        assert_eq!(raw_record.import_batch_id, "zulip-lab-batch");
        assert_eq!(raw_record.payload["provider_message_id"], json!("7001"));
        assert_eq!(raw_record.payload["stream_name"], json!("Hermes Lab"));
        assert_eq!(raw_record.payload["topic"], json!("Tasks"));
        assert_eq!(
            raw_record.payload["content"],
            json!("Надо проверить backup retention до пятницы.")
        );
        assert_eq!(raw_record.provenance["provider_kind"], json!("zulip_bot"));
        assert_eq!(
            raw_record.provenance["scenario_run_id"],
            json!("zulip-message-to-task-run")
        );
        assert!(raw_record.source_fingerprint.starts_with("sha256:"));
        assert!(raw_record.observation_id.starts_with("raw_zulip_"));
    }

    #[test]
    fn maps_message_attachment_metadata_without_materializing_bytes() {
        let event: ZulipEvent = serde_json::from_value(json!({
            "id": 46,
            "type": "message",
            "message": {
                "id": 7002,
                "content": "См. вложение.",
                "sender_email": "bot@example.test",
                "sender_full_name": "Hermes Bot",
                "stream_id": 10,
                "display_recipient": "Hermes Lab",
                "topic": "Evidence",
                "attachments": [
                    {
                        "id": "zulip-file-1",
                        "name": "evidence.pdf",
                        "content_type": "application/pdf",
                        "size": 2048,
                        "url": "/user_uploads/1/evidence.pdf",
                        "path_id": "1/evidence.pdf"
                    }
                ]
            }
        }))
        .expect("valid Zulip event fixture");
        let received_at = DateTime::parse_from_rfc3339("2026-06-29T08:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc);
        let context = ZulipEventMappingContext::new(
            "lab-zulip-account",
            "http://localhost:8080",
            received_at,
        );

        let raw_record = map_zulip_event_to_raw_record(&event, &context)
            .expect("message event maps to raw record");

        assert_eq!(
            raw_record.payload["attachments"][0]["provider"],
            json!("zulip")
        );
        assert_eq!(
            raw_record.payload["attachments"][0]["provider_attachment_id"],
            json!("zulip-file-1")
        );
        assert_eq!(
            raw_record.payload["attachments"][0]["filename"],
            json!("evidence.pdf")
        );
        assert_eq!(
            raw_record.payload["attachments"][0]["bytes_state"],
            json!("not_transferred")
        );
        assert_eq!(
            raw_record.payload["attachments"][0]["scan_status"],
            json!("not_scanned")
        );
        assert_eq!(
            raw_record.payload["attachment_state"]["materialization_state"],
            json!("not_materialized")
        );
        assert_eq!(
            raw_record.payload["raw_event"]["message"]["attachments"][0]["url"],
            json!("/user_uploads/1/evidence.pdf")
        );
    }

    #[test]
    fn maps_user_upload_links_from_message_content_as_attachment_evidence() {
        let event: ZulipEvent = serde_json::from_value(json!({
            "id": 47,
            "type": "message",
            "message": {
                "id": 7003,
                "content": "<p><a href=\"/user_uploads/2/ab/hermes-fact.txt\">hermes-fact.txt</a></p>",
                "sender_email": "bot@example.test",
                "sender_full_name": "Hermes Bot",
                "stream_id": 10,
                "display_recipient": "Hermes Lab",
                "topic": "Evidence"
            }
        }))
        .expect("valid Zulip event fixture");
        let received_at = DateTime::parse_from_rfc3339("2026-06-29T08:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc);
        let context = ZulipEventMappingContext::new(
            "lab-zulip-account",
            "http://localhost:8080",
            received_at,
        );

        let raw_record = map_zulip_event_to_raw_record(&event, &context)
            .expect("message event maps to raw record");

        assert_eq!(
            raw_record.payload["attachments"][0]["provider_attachment_id"],
            json!("2/ab/hermes-fact.txt")
        );
        assert_eq!(
            raw_record.payload["attachments"][0]["filename"],
            json!("hermes-fact.txt")
        );
        assert_eq!(
            raw_record.payload["attachments"][0]["url"],
            json!("/user_uploads/2/ab/hermes-fact.txt")
        );
        assert_eq!(
            raw_record.payload["attachment_state"]["bytes_state"],
            json!("not_transferred")
        );
        assert_eq!(
            raw_record.payload["attachment_state"]["materialization_state"],
            json!("not_materialized")
        );
    }

    #[test]
    fn raw_signal_event_types_match_signal_hub_contract() {
        assert_eq!(
            zulip_raw_signal_event_type("message"),
            zulip_raw_signal_event_types::MESSAGE
        );
        assert_eq!(
            zulip_raw_signal_event_type("reaction"),
            zulip_raw_signal_event_types::REACTION
        );
        assert_eq!(
            zulip_raw_signal_event_type("update_message"),
            zulip_raw_signal_event_types::MESSAGE_UPDATE
        );
        assert_eq!(
            zulip_raw_signal_event_type("delete_message"),
            zulip_raw_signal_event_types::MESSAGE_DELETE
        );
        assert_eq!(
            zulip_raw_signal_event_type("realm_emoji"),
            zulip_raw_signal_event_types::UNKNOWN
        );
    }

    #[test]
    fn maps_non_message_events_to_target_message_payloads() {
        let received_at = DateTime::parse_from_rfc3339("2026-06-29T08:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc);
        let context = ZulipEventMappingContext::new(
            "lab-zulip-account",
            "http://localhost:8080",
            received_at,
        );

        let reaction_event: ZulipEvent = serde_json::from_value(json!({
            "id": 43,
            "type": "reaction",
            "message_id": 7001,
            "emoji_name": "+1",
            "emoji_code": "1f44d",
            "reaction_type": "unicode_emoji",
            "op": "add",
            "user_id": 55,
            "user": {
                "full_name": "Zulip Reactor",
                "email": "reactor@example.test"
            }
        }))
        .expect("valid reaction event fixture");
        let reaction_raw = map_zulip_event_to_raw_record(&reaction_event, &context)
            .expect("reaction event maps to raw record");
        assert_eq!(reaction_raw.record_kind, "zulip_reaction");
        assert_eq!(reaction_raw.provider_record_id, "reaction:43");
        assert_eq!(reaction_raw.payload["provider_message_id"], json!("7001"));
        assert_eq!(reaction_raw.payload["emoji_name"], json!("+1"));
        assert_eq!(reaction_raw.payload["reaction_op"], json!("add"));
        assert_eq!(reaction_raw.payload["provider_actor_id"], json!("55"));
        assert_eq!(
            reaction_raw.payload["sender_display_name"],
            json!("Zulip Reactor")
        );

        let update_event: ZulipEvent = serde_json::from_value(json!({
            "id": 44,
            "type": "update_message",
            "message_id": 7001,
            "content": "Updated Zulip content",
            "prev_content": "Original Zulip content",
            "topic": "Tasks",
            "edit_timestamp": 1782720060
        }))
        .expect("valid message update event fixture");
        let update_raw = map_zulip_event_to_raw_record(&update_event, &context)
            .expect("message update event maps to raw record");
        assert_eq!(update_raw.record_kind, "zulip_message_update");
        assert_eq!(update_raw.provider_record_id, "update_message:44");
        assert_eq!(update_raw.payload["provider_message_id"], json!("7001"));
        assert_eq!(
            update_raw.payload["content"],
            json!("Updated Zulip content")
        );
        assert_eq!(
            update_raw.payload["prev_content"],
            json!("Original Zulip content")
        );
        assert_eq!(update_raw.payload["topic"], json!("Tasks"));
        assert_eq!(update_raw.payload["edit_timestamp"], json!("1782720060"));

        let delete_event: ZulipEvent = serde_json::from_value(json!({
            "id": 45,
            "type": "delete_message",
            "message_id": 7001,
            "message_type": "stream",
            "stream_id": 10,
            "topic": "Tasks"
        }))
        .expect("valid message delete event fixture");
        let delete_raw = map_zulip_event_to_raw_record(&delete_event, &context)
            .expect("message delete event maps to raw record");
        assert_eq!(delete_raw.record_kind, "zulip_message_delete");
        assert_eq!(delete_raw.provider_record_id, "delete_message:45");
        assert_eq!(delete_raw.payload["provider_message_id"], json!("7001"));
        assert_eq!(delete_raw.payload["message_type"], json!("stream"));
        assert_eq!(delete_raw.payload["stream_id"], json!("10"));
        assert_eq!(delete_raw.payload["topic"], json!("Tasks"));
    }
}
