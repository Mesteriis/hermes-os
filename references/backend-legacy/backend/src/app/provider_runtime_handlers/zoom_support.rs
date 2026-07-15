use super::*;

pub(super) async fn sync_zoom_signal_connection(
    state: &AppState,
    account_id: &str,
) -> Result<(), ApiError> {
    let account = provider_account_or_not_found(state, account_id).await?;
    sync_provider_account_signal_connection(state, &account, None).await
}

pub(super) async fn zoom_remote_transcript_download_enabled(
    state: &AppState,
) -> Result<bool, ApiError> {
    Ok(settings_store(state)?
        .setting(ZOOM_REMOTE_TRANSCRIPT_DOWNLOAD_ENABLED_SETTING_KEY)
        .await?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false))
}

pub(super) async fn zoom_remote_recording_download_enabled(
    state: &AppState,
) -> Result<bool, ApiError> {
    Ok(settings_store(state)?
        .setting(ZOOM_REMOTE_RECORDING_DOWNLOAD_ENABLED_SETTING_KEY)
        .await?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false))
}

pub(super) const fn default_zoom_recording_imports_limit() -> i64 {
    20
}

pub(super) const fn default_zoom_audit_events_limit() -> i64 {
    25
}

pub(super) fn validate_zoom_webhook_account_id(account_id: &str) -> Result<String, ZoomError> {
    let trimmed = account_id.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(
            "account_id query parameter is required for Zoom webhook ingestion".to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn require_zoom_unlocked_host_vault(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(ApiError::HostVault(HostVaultError::Locked)),
        VaultMode::Uninitialized => Err(ApiError::HostVault(HostVaultError::Uninitialized)),
    }
}

pub(super) async fn read_zoom_webhook_secret(
    state: &AppState,
    account_id: &str,
) -> Result<String, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let binding = CommunicationProviderSecretBindingStore::new(pool)
        .get_for_account(account_id, ProviderAccountSecretPurpose::ZoomWebhookSecret)
        .await
        .map_err(|error| ZoomError::InvalidRequest(error.to_string()))?
        .ok_or_else(|| {
            ZoomError::InvalidRequest(format!(
                "Zoom webhook secret is not configured for account `{account_id}`"
            ))
        })?;
    state
        .vault
        .read_secret(&binding.secret_ref)
        .map_err(ApiError::HostVault)
}

pub(super) fn zoom_webhook_envelope(body: &[u8]) -> Result<Value, ZoomError> {
    serde_json::from_slice::<Value>(body).map_err(|error| {
        ZoomError::InvalidRequest(format!("Zoom webhook body must be valid JSON: {error}"))
    })
}

pub(super) fn zoom_webhook_event(envelope: &Value) -> Result<&str, ZoomError> {
    envelope
        .get("event")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ZoomError::InvalidRequest("Zoom webhook event is required".to_owned()))
}

pub(super) fn zoom_webhook_plain_token(envelope: &Value) -> Result<String, ZoomError> {
    envelope
        .get("payload")
        .and_then(|payload| payload.get("plainToken"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "Zoom endpoint.url_validation payload.plainToken is required".to_owned(),
            )
        })
}

pub(super) fn verify_zoom_webhook_signature(
    webhook_secret: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(), ZoomError> {
    let timestamp = headers
        .get(ZOOM_TIMESTAMP_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "x-zm-request-timestamp is required for Zoom webhook ingestion".to_owned(),
            )
        })?;
    let timestamp_seconds = timestamp.parse::<i64>().map_err(|_| {
        ZoomError::InvalidRequest(
            "x-zm-request-timestamp must be a Unix timestamp in seconds".to_owned(),
        )
    })?;
    let now_seconds = Utc::now().timestamp();
    if (now_seconds - timestamp_seconds).abs() > ZOOM_WEBHOOK_SIGNATURE_TOLERANCE_SECONDS {
        return Err(ZoomError::InvalidRequest(
            "Zoom webhook timestamp is outside the allowed replay window".to_owned(),
        ));
    }

    let signature = headers
        .get(ZOOM_SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "x-zm-signature is required for Zoom webhook ingestion".to_owned(),
            )
        })?;
    let signature = signature.strip_prefix("v0=").ok_or_else(|| {
        ZoomError::InvalidRequest("x-zm-signature must use v0=<hex-digest>".to_owned())
    })?;
    let signature = decode_sha256_hex(signature).ok_or_else(|| {
        ZoomError::InvalidRequest("x-zm-signature must contain a 32-byte hex digest".to_owned())
    })?;

    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes()).map_err(|_| {
        ZoomError::InvalidRequest("Zoom webhook secret is invalid for HMAC".to_owned())
    })?;
    mac.update(b"v0:");
    mac.update(timestamp.as_bytes());
    mac.update(b":");
    mac.update(body);
    mac.verify_slice(&signature).map_err(|_| {
        ZoomError::InvalidRequest("Zoom webhook signature verification failed".to_owned())
    })
}

pub(super) fn zoom_webhook_validation_token(
    webhook_secret: &str,
    plain_token: &str,
) -> Result<String, ZoomError> {
    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes()).map_err(|_| {
        ZoomError::InvalidRequest("Zoom webhook secret is invalid for HMAC".to_owned())
    })?;
    mac.update(plain_token.as_bytes());
    Ok(bytes_to_lower_hex(&mac.finalize().into_bytes()))
}

pub(super) fn zoom_meeting_observation_from_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<ZoomMeetingObservationRequest, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    Ok(ZoomMeetingObservationRequest {
        observation_id: Some(zoom_webhook_observation_id(event, envelope, &meeting_id)),
        account_id: account_id.to_owned(),
        meeting_id,
        meeting_uuid: value_string(object, &["uuid", "meeting_uuid"]),
        topic: value_string(object, &["topic"]),
        host_email: value_string(object, &["host_email"]),
        join_url: value_string(object, &["join_url"]),
        started_at: value_datetime(object, &["start_time", "started_at"]),
        ended_at: value_datetime(object, &["end_time", "ended_at"]),
        duration_seconds: value_i64(object, &["duration_seconds"]),
        participants: Vec::new(),
        recording_refs: Vec::new(),
        transcript_ref: value_string(object, &["transcript_ref"]),
        metadata: json!({
            "webhook_event": event,
            "webhook_event_ts": envelope.get("event_ts"),
            "webhook_payload": envelope.get("payload"),
        }),
        causation_id: value_string(envelope, &["event_id"]),
        correlation_id: Some(format!("zoom-webhook:{account_id}")),
    })
}

pub(super) fn zoom_recording_observations_from_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<Vec<ZoomRecordingObservationRequest>, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    let Some(files) = object.get("recording_files").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut requests = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let recording_id = value_string(file, &["id", "file_id", "recording_id"])
            .unwrap_or_else(|| format!("{}:recording-file:{index}", meeting_id.trim()));
        let recording = ZoomRecordingRef {
            recording_id,
            recording_type: value_string(file, &["recording_type", "file_type"]),
            download_ref: value_string(file, &["download_url", "download_ref"]),
            file_extension: value_string(file, &["file_extension", "file_type"]),
            file_size_bytes: value_i64(file, &["file_size", "file_size_bytes"]),
            recorded_at: value_datetime(file, &["recording_start", "start_time", "recorded_at"]),
            metadata: json!({
                "webhook_event": event,
                "webhook_file": file,
            }),
        };
        requests.push(ZoomRecordingObservationRequest {
            observation_id: Some(zoom_webhook_observation_id(
                event,
                envelope,
                &format!("{}:{}", meeting_id.trim(), recording.recording_id.trim()),
            )),
            account_id: account_id.to_owned(),
            meeting_id: meeting_id.clone(),
            recording,
            metadata: json!({
                "webhook_event": event,
                "webhook_event_ts": envelope.get("event_ts"),
                "webhook_payload": envelope.get("payload"),
            }),
            causation_id: value_string(envelope, &["event_id"]),
            correlation_id: Some(format!("zoom-webhook:{account_id}")),
        });
    }
    Ok(requests)
}

pub(super) fn zoom_transcript_downloads_from_recording_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<Vec<ZoomWebhookTranscriptDownload>, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    let meeting_uuid = value_string(object, &["uuid"]);
    let Some(files) = object.get("recording_files").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    let mut requests = Vec::new();
    for (index, file) in files.iter().enumerate() {
        if !is_zoom_transcript_recording_file(file) {
            continue;
        }
        let Some(download_url) = value_string(file, &["download_url", "download_ref"]) else {
            continue;
        };
        let recording_id = value_string(file, &["id", "file_id", "recording_id"])
            .unwrap_or_else(|| format!("{}:transcript-file:{index}", meeting_id.trim()));
        let file_name = value_string(file, &["file_name"]).or_else(|| {
            transcript_file_name(
                &recording_id,
                value_string(file, &["file_extension", "file_type"]).as_deref(),
            )
        });
        let content_type = transcript_content_type(
            value_string(file, &["file_extension", "file_type"]).as_deref(),
            file_name.as_deref(),
        );
        requests.push(ZoomWebhookTranscriptDownload {
            request: ZoomTranscriptFileImportRequest {
                observation_id: Some(zoom_webhook_observation_id(
                    event,
                    envelope,
                    &format!("{}:{}:transcript", meeting_id.trim(), recording_id.trim()),
                )),
                transcript_id: format!(
                    "zoom-transcript-download:{}:{}",
                    meeting_id.trim(),
                    recording_id.trim()
                ),
                account_id: account_id.to_owned(),
                meeting_id: meeting_id.clone(),
                meeting_uuid: meeting_uuid.clone(),
                source_recording_ref: Some(recording_id),
                language_code: value_string(file, &["file_language", "language"]),
                file_name,
                content_type,
                file_text: String::new(),
                metadata: json!({
                    "webhook_transcript_download": {
                        "event": event,
                        "source": "recording_webhook",
                        "file_type": value_string(file, &["file_type"]),
                        "file_extension": value_string(file, &["file_extension"]),
                    }
                }),
                causation_id: value_string(envelope, &["event_id"]),
                correlation_id: Some(format!("zoom-webhook:{account_id}")),
            },
            download_url,
            download_token: value_string(file, &["download_token"]),
        });
    }

    Ok(requests)
}

pub(super) fn zoom_recording_media_downloads_from_recording_webhook(
    account_id: &str,
    event: &str,
    envelope: &Value,
) -> Result<Vec<ZoomWebhookRecordingMediaDownload>, ZoomError> {
    let object = zoom_webhook_object(envelope)?;
    let meeting_id = required_value_string(object, &["id", "meeting_id"], "payload.object.id")?;
    let meeting_uuid = value_string(object, &["uuid"]);
    let Some(files) = object.get("recording_files").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    let mut requests = Vec::new();
    for (index, file) in files.iter().enumerate() {
        if is_zoom_transcript_recording_file(file) {
            continue;
        }
        let Some(download_url) = value_string(file, &["download_url", "download_ref"]) else {
            continue;
        };
        let recording_id = value_string(file, &["id", "file_id", "recording_id"])
            .unwrap_or_else(|| format!("{}:recording-file:{index}", meeting_id.trim()));
        let file_extension = value_string(file, &["file_extension", "file_type"]);
        let file_name = value_string(file, &["file_name"])
            .or_else(|| recording_media_file_name(&recording_id, file_extension.as_deref()));
        requests.push(ZoomWebhookRecordingMediaDownload {
            request: ZoomRecordingMediaDownloadRequest {
                observation_id: Some(zoom_webhook_observation_id(
                    event,
                    envelope,
                    &format!(
                        "{}:{}:recording-download",
                        meeting_id.trim(),
                        recording_id.trim()
                    ),
                )),
                account_id: account_id.to_owned(),
                meeting_id: meeting_id.clone(),
                meeting_uuid: meeting_uuid.clone(),
                recording: ZoomRecordingRef {
                    recording_id,
                    recording_type: value_string(file, &["recording_type", "file_type"]),
                    download_ref: Some(download_url.clone()),
                    file_extension: file_extension.clone(),
                    file_size_bytes: value_i64(file, &["file_size", "file_size_bytes"]),
                    recorded_at: value_datetime(
                        file,
                        &["recording_start", "start_time", "recorded_at"],
                    ),
                    metadata: json!({
                        "webhook_event": event,
                        "webhook_file": file,
                        "file_type": value_string(file, &["file_type"]),
                    }),
                },
                file_name,
                content_type: None,
                download_url,
                metadata: json!({
                    "source": "zoom_recording_webhook",
                    "webhook_event": event,
                    "webhook_event_ts": envelope.get("event_ts"),
                    "webhook_payload": envelope.get("payload"),
                }),
                causation_id: value_string(envelope, &["event_id"]),
                correlation_id: Some(format!("zoom-webhook:{account_id}")),
            },
            download_token: value_string(file, &["download_token"]),
        });
    }

    Ok(requests)
}

pub(super) fn zoom_webhook_object(envelope: &Value) -> Result<&Value, ZoomError> {
    envelope
        .get("payload")
        .and_then(|payload| payload.get("object"))
        .filter(|value| value.is_object())
        .ok_or_else(|| {
            ZoomError::InvalidRequest("Zoom webhook payload.object is required".to_owned())
        })
}

pub(super) fn zoom_webhook_observation_id(
    event: &str,
    envelope: &Value,
    subject_id: &str,
) -> String {
    if let Some(event_id) = value_string(envelope, &["event_id"]) {
        return event_id;
    }
    format!(
        "zoom-webhook:{}:{}:{}",
        event,
        envelope
            .get("event_ts")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
        subject_id.trim()
    )
}

pub(super) fn is_zoom_transcript_recording_file(file: &Value) -> bool {
    let file_type = value_string(file, &["file_type"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    let recording_type = value_string(file, &["recording_type"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_extension = value_string(file, &["file_extension"])
        .or_else(|| {
            value_string(file, &["file_name"]).and_then(|name| {
                name.rsplit_once('.')
                    .map(|(_, extension)| extension.trim().to_owned())
            })
        })
        .unwrap_or_default()
        .to_ascii_lowercase();

    file_type.contains("transcript")
        || recording_type.contains("transcript")
        || matches!(file_extension.as_str(), "vtt" | "srt" | "txt")
        || file_type == "cc"
        || file_type == "vtt"
        || file_type == "srt"
}

pub(super) fn transcript_file_name(
    recording_id: &str,
    extension_hint: Option<&str>,
) -> Option<String> {
    let extension = extension_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches('.').to_ascii_lowercase())?;
    Some(format!("{recording_id}.{extension}"))
}

pub(super) fn recording_media_file_name(
    recording_id: &str,
    extension_hint: Option<&str>,
) -> Option<String> {
    let extension = extension_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_ascii_lowercase();
    Some(format!("{recording_id}.{extension}"))
}

pub(super) fn transcript_content_type(
    extension_hint: Option<&str>,
    file_name: Option<&str>,
) -> Option<String> {
    let extension = extension_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches('.').to_ascii_lowercase())
        .or_else(|| {
            file_name.and_then(|value| {
                value
                    .rsplit_once('.')
                    .map(|(_, extension)| extension.trim().to_ascii_lowercase())
            })
        })?;
    Some(
        match extension.as_str() {
            "vtt" => "text/vtt",
            "srt" => "application/x-subrip",
            "txt" => "text/plain",
            _ => return None,
        }
        .to_owned(),
    )
}

pub(super) fn required_value_string(
    object: &Value,
    keys: &[&str],
    field: &'static str,
) -> Result<String, ZoomError> {
    value_string(object, keys)
        .ok_or_else(|| ZoomError::InvalidRequest(format!("Zoom webhook {field} must not be empty")))
}

pub(super) fn value_string(object: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| {
            object.get(*key).and_then(|value| match value {
                Value::String(raw) => Some(raw.trim().to_owned()),
                Value::Number(number) => Some(number.to_string()),
                _ => None,
            })
        })
        .filter(|value| !value.trim().is_empty())
}

pub(super) fn value_i64(object: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| {
        object.get(*key).and_then(|value| match value {
            Value::Number(number) => number.as_i64(),
            Value::String(raw) => raw.trim().parse::<i64>().ok(),
            _ => None,
        })
    })
}

pub(super) fn value_datetime(object: &Value, keys: &[&str]) -> Option<DateTime<Utc>> {
    keys.iter().find_map(|key| {
        object
            .get(*key)
            .and_then(Value::as_str)
            .and_then(|raw| DateTime::parse_from_rfc3339(raw.trim()).ok())
            .map(|value| value.with_timezone(&Utc))
    })
}

pub(super) fn decode_sha256_hex(value: &str) -> Option<[u8; 32]> {
    let value = value.trim();
    if value.len() != 64 {
        return None;
    }
    let mut output = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_hex_nibble(chunk[0])?;
        let low = decode_hex_nibble(chunk[1])?;
        output[index] = (high << 4) | low;
    }
    Some(output)
}

pub(super) fn decode_hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

pub(super) fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
