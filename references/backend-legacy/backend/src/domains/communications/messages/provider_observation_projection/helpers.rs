use super::*;

pub(super) fn merged_zulip_message_metadata(
    current: &Value,
    patch: Value,
) -> Result<Value, CommunicationSignalProjectionError> {
    let Some(current) = current.as_object() else {
        return Err(MessageProjectionError::InvalidMessageMetadata.into());
    };
    let Some(patch) = patch.as_object() else {
        return Err(MessageProjectionError::InvalidMessageMetadata.into());
    };
    let mut merged = current.clone();
    for (key, value) in patch {
        if !value.is_null() {
            merged.insert(key.clone(), value.clone());
        }
    }
    Ok(Value::Object(merged))
}

pub(super) fn zulip_observed_at(
    raw_record: &StoredRawCommunicationRecord,
    event: &EventEnvelope,
) -> DateTime<Utc> {
    raw_record.occurred_at.unwrap_or(event.occurred_at)
}

pub(super) fn zulip_content_diff(previous_text: Option<&str>, new_text: &str) -> Value {
    json!({
        "changed": previous_text != Some(new_text),
        "previous_text_length": previous_text.map(|text| text.chars().count()),
        "new_text_length": new_text.chars().count(),
    })
}

pub(super) fn parse_observed_at(
    payload: &Value,
) -> Result<DateTime<Utc>, ProviderCommunicationMessagePortError> {
    let Some(value) = payload.get("observed_at") else {
        return Ok(Utc::now());
    };
    let Some(value) = value.as_str() else {
        return Err(ProviderCommunicationMessagePortError::InvalidRequest(
            "observed_at must be an RFC3339 string".to_owned(),
        ));
    };
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "invalid observed_at: {error}"
            ))
        })
}

pub(super) fn required_str<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a str, ProviderCommunicationMessagePortError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "{field} must be a non-empty string"
            ))
        })
}

pub(super) fn optional_str<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) fn required_i64(
    value: &Value,
    field: &str,
) -> Result<i64, ProviderCommunicationMessagePortError> {
    value.get(field).and_then(Value::as_i64).ok_or_else(|| {
        ProviderCommunicationMessagePortError::InvalidRequest(format!("{field} must be an integer"))
    })
}

pub(super) fn optional_i64(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

pub(super) fn required_payload_str(
    value: &Value,
    field: &'static str,
) -> Result<String, CommunicationSignalProjectionError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or(CommunicationSignalProjectionError::Message(
            MessageProjectionError::MissingPayloadField(field),
        ))
}

pub(super) fn optional_payload_str(value: &Value, field: &'static str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn required_subject_str<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, CommunicationSignalProjectionError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(CommunicationSignalProjectionError::MissingSubjectField(
            field,
        ))
}

pub(super) fn mail_blob_root_from_event(event: &EventEnvelope) -> &Path {
    event
        .provenance
        .get("raw_event_provenance")
        .and_then(|value| value.get("blob_root"))
        .and_then(Value::as_str)
        .map(Path::new)
        .unwrap_or_else(|| Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT))
}

pub(super) fn whatsapp_web_message_id(account_id: &str, provider_message_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_message_id.as_bytes());
    format!("message:v5:whatsapp_web:{:x}", hasher.finalize())
}

pub(super) fn telegram_message_id(account_id: &str, provider_message_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_message_id.as_bytes());
    format!("message:v4:telegram:{:x}", hasher.finalize())
}

pub(super) fn zulip_message_id(account_id: &str, provider_message_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_message_id.as_bytes());
    format!("message:v1:zulip:{:x}", hasher.finalize())
}

pub(super) fn zulip_conversation_id(account_id: &str, stream_name: &str, topic: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(stream_name.as_bytes());
    hasher.update(b"\0");
    hasher.update(topic.as_bytes());
    format!("zulip:conversation:{:x}", hasher.finalize())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ZulipMessageTarget {
    pub(super) subject: String,
    pub(super) recipients: Vec<String>,
    pub(super) conversation_id: String,
}

pub(super) fn zulip_message_target(account_id: &str, payload: &Value) -> ZulipMessageTarget {
    let direct_recipients = zulip_direct_recipient_refs(payload);
    let message_type = optional_payload_str(payload, "message_type").unwrap_or_default();
    if is_zulip_direct_message(&message_type) || !direct_recipients.is_empty() {
        let display_names = zulip_direct_recipient_display_names(payload);
        let recipients = if direct_recipients.is_empty() {
            vec!["Zulip direct".to_owned()]
        } else {
            direct_recipients
        };
        let subject_suffix = if display_names.is_empty() {
            recipients.join(", ")
        } else {
            display_names.join(", ")
        };
        return ZulipMessageTarget {
            subject: if subject_suffix.trim().is_empty() {
                "Direct message".to_owned()
            } else {
                format!("Direct / {subject_suffix}")
            },
            conversation_id: zulip_direct_conversation_id(account_id, &recipients),
            recipients,
        };
    }

    let stream_name = optional_payload_str(payload, "stream_name")
        .or_else(|| optional_payload_str(payload, "stream_id"))
        .unwrap_or_else(|| "Zulip".to_owned());
    let topic = optional_payload_str(payload, "topic").unwrap_or_else(|| "message".to_owned());
    ZulipMessageTarget {
        subject: format!("{stream_name} / {topic}"),
        recipients: vec![stream_name.clone()],
        conversation_id: zulip_conversation_id(account_id, &stream_name, &topic),
    }
}

pub(super) fn is_zulip_direct_message(message_type: &str) -> bool {
    matches!(message_type.trim(), "private" | "direct")
}

pub(super) fn zulip_direct_recipient_refs(payload: &Value) -> Vec<String> {
    zulip_direct_recipient_values(
        payload,
        &["email", "full_name", "display_name", "provider_user_id"],
    )
}

pub(super) fn zulip_direct_recipient_display_names(payload: &Value) -> Vec<String> {
    zulip_direct_recipient_values(
        payload,
        &["full_name", "display_name", "email", "provider_user_id"],
    )
}

pub(super) fn zulip_direct_recipient_values(payload: &Value, fields: &[&str]) -> Vec<String> {
    let Some(recipients) = payload.get("direct_recipients").and_then(Value::as_array) else {
        return Vec::new();
    };

    recipients
        .iter()
        .filter_map(|recipient| {
            if let Some(value) = recipient
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                return Some(value.to_owned());
            }
            let object = recipient.as_object()?;
            fields.iter().find_map(|field| {
                object
                    .get(*field)
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
            })
        })
        .collect()
}

pub(super) fn zulip_direct_conversation_id(account_id: &str, recipients: &[String]) -> String {
    let mut recipient_refs = recipients
        .iter()
        .map(|recipient| recipient.trim())
        .filter(|recipient| !recipient.is_empty())
        .collect::<Vec<_>>();
    recipient_refs.sort_unstable();

    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0direct\0");
    for recipient in recipient_refs {
        hasher.update(recipient.as_bytes());
        hasher.update(b"\0");
    }
    format!("zulip:direct_conversation:{:x}", hasher.finalize())
}

pub(super) fn zulip_reaction_id(
    account_id: &str,
    provider_message_id: &str,
    provider_actor_id: &str,
    reaction: &str,
) -> String {
    stable_zulip_id(
        "message_reaction:v1:zulip",
        &[account_id, provider_message_id, provider_actor_id, reaction],
    )
}

pub(super) fn zulip_message_version_id(accepted_event_id: &str) -> String {
    stable_zulip_id("message_version:v1:zulip", &[accepted_event_id])
}

pub(super) fn zulip_message_tombstone_id(accepted_event_id: &str) -> String {
    stable_zulip_id("message_tombstone:v1:zulip", &[accepted_event_id])
}

pub(super) fn stable_zulip_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.trim().as_bytes());
        hasher.update(b"\0");
    }
    format!("{prefix}:{:x}", hasher.finalize())
}
