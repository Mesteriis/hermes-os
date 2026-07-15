use super::*;

pub(super) fn provider_sync_user_id(
    account: &ZoomAccount,
    request: &ZoomRecordingSyncRequest,
) -> Result<String, ZoomError> {
    if let Some(user_id) = request
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(user_id.to_owned());
    }
    if account.auth_shape == ZoomAuthShape::OAuthUser.as_str() {
        return Ok("me".to_owned());
    }
    let external_account_id = account.external_account_id.trim();
    if !external_account_id.is_empty() {
        return Ok(external_account_id.to_owned());
    }
    Err(ZoomError::InvalidRequest(
        "user_id is required for server_to_server recording sync when external_account_id is empty"
            .to_owned(),
    ))
}

pub(super) fn zoom_recording_content_type(extension: Option<&str>) -> Option<String> {
    let normalized = extension
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_ascii_lowercase();
    let content_type = match normalized.as_str() {
        "mp4" => "video/mp4",
        "m4a" => "audio/mp4",
        "m4v" => "video/x-m4v",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "chat" => "text/plain",
        _ => "application/octet-stream",
    };
    Some(content_type.to_owned())
}

pub(super) fn zoom_recording_import_audit_item(
    item: ImportedAttachmentRecord,
) -> ZoomRecordingImportAuditItem {
    let retention_policy = item.metadata.get("retention_policy");
    ZoomRecordingImportAuditItem {
        attachment_id: item.attachment_id,
        account_id: item.account_id.unwrap_or_default(),
        meeting_id: item
            .metadata
            .get("meeting_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        meeting_uuid: item
            .metadata
            .get("meeting_uuid")
            .and_then(Value::as_str)
            .map(str::to_owned),
        recording_id: item
            .metadata
            .get("recording_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        filename: item.filename,
        content_type: item.content_type,
        size_bytes: item.size_bytes,
        sha256: item.sha256,
        source: item
            .metadata
            .get("source")
            .and_then(Value::as_str)
            .map(str::to_owned),
        scan_status: item.scan_status.as_str().to_owned(),
        scan_summary: item.scan_summary,
        storage_kind: item.storage_kind,
        storage_path: item.storage_path,
        retention_mode: retention_policy
            .and_then(|value| value.get("mode"))
            .and_then(Value::as_str)
            .unwrap_or("explicit_remove_only")
            .to_owned(),
        retention_days: retention_policy
            .and_then(|value| value.get("retention_days"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
        expires_at: retention_policy
            .and_then(|value| value.get("expires_at"))
            .and_then(Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc)),
        created_at: item.created_at,
    }
}

pub(super) fn json_string_or_number(field: &str, value: &Value) -> Result<String, ZoomError> {
    match value {
        Value::String(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(ZoomError::InvalidRequest(format!(
                    "{field} must not be empty"
                )))
            } else {
                Ok(trimmed.to_owned())
            }
        }
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(ZoomError::InvalidRequest(format!(
            "{field} must be a string or number"
        ))),
    }
}

pub(super) fn zoom_api_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    value
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .and_then(|candidate| {
            DateTime::parse_from_rfc3339(candidate)
                .ok()
                .map(|value| value.with_timezone(&Utc))
        })
}

pub(super) fn zoom_recording_file_is_transcript(recording: &ZoomApiRecordingFile) -> bool {
    [
        recording.file_extension.as_deref(),
        recording.file_type.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(|value| value.trim().to_ascii_lowercase())
    .any(|value| matches!(value.as_str(), "vtt" | "srt" | "txt" | "transcript" | "cc"))
        || recording
            .recording_type
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .is_some_and(|value| value.contains("transcript") || value == "audio_transcript")
}

pub(super) fn zoom_transcript_content_type(extension_or_type: Option<&str>) -> Option<String> {
    match extension_or_type
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_ascii_lowercase()
        .as_str()
    {
        "vtt" => Some("text/vtt".to_owned()),
        "srt" => Some("application/x-subrip".to_owned()),
        "txt" | "transcript" | "cc" => Some("text/plain".to_owned()),
        _ => None,
    }
}

pub(super) fn stable_zoom_transcript_id(
    account_id: &str,
    meeting_id: &str,
    recording_id: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(meeting_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(recording_id.trim().as_bytes());
    format!("zoom_transcript_{:x}", hasher.finalize())
}

pub(super) fn stable_zoom_call_id(account_id: &str, meeting_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(meeting_id.trim().as_bytes());
    format!("zoom_call_{:x}", hasher.finalize())
}

pub(super) fn zoom_event_id(
    kind: &str,
    account_id: &str,
    subject_id: &str,
    observation_id: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b":");
    hasher.update(account_id.trim().as_bytes());
    hasher.update(b":");
    hasher.update(subject_id.trim().as_bytes());
    if let Some(observation_id) = observation_id {
        hasher.update(b":");
        hasher.update(observation_id.trim().as_bytes());
    }
    format!("evt_zoom_{}_{:x}", kind, hasher.finalize())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn zoom_event(
    event_id: String,
    event_type: &str,
    occurred_at: DateTime<Utc>,
    source: Value,
    subject: Value,
    payload: Value,
    provenance: Value,
    causation_id: Option<String>,
    correlation_id: Option<String>,
) -> Result<NewEventEnvelope, ZoomError> {
    let mut builder = NewEventEnvelope::builder(event_id, event_type, occurred_at, source, subject)
        .payload(payload)
        .provenance(provenance);
    if let Some(causation_id) = causation_id {
        builder = builder.causation_id(causation_id);
    }
    if let Some(correlation_id) = correlation_id {
        builder = builder.correlation_id(correlation_id);
    }
    Ok(builder.build()?)
}
