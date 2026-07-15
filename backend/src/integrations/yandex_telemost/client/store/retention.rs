use super::super::errors::YandexTelemostError;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn retention_cleanup_candidate_from_event(
    payload: &Value,
    occurred_at: DateTime<Utc>,
) -> Option<TelemostRetentionCleanupCandidate> {
    let account_id = payload.get("account_id")?.as_str()?.trim();
    let bundle_id = payload
        .get("bundle_id")
        .or_else(|| payload.get("recording_session_id"))?
        .as_str()?
        .trim();
    let bundle_root = PathBuf::from(payload.get("bundle_root")?.as_str()?.trim());
    let manifest_path = PathBuf::from(payload.get("manifest_path")?.as_str()?.trim());
    let audio_path = PathBuf::from(payload.get("audio_path")?.as_str()?.trim());
    let speaker_jsonl_path = PathBuf::from(payload.get("speaker_jsonl_path")?.as_str()?.trim());
    let speaker_txt_path = PathBuf::from(payload.get("speaker_txt_path")?.as_str()?.trim());
    if account_id.is_empty()
        || bundle_id.is_empty()
        || !bundle_root.is_absolute()
        || !manifest_path.is_absolute()
        || !audio_path.is_absolute()
        || !speaker_jsonl_path.is_absolute()
        || !speaker_txt_path.is_absolute()
        || occurred_at.timestamp() <= 0
    {
        return None;
    }
    if !audio_path.starts_with(&bundle_root)
        || !speaker_jsonl_path.starts_with(&bundle_root)
        || !speaker_txt_path.starts_with(&bundle_root)
        || !manifest_path.starts_with(&bundle_root)
    {
        return None;
    }

    Some(TelemostRetentionCleanupCandidate {
        account_id: account_id.to_owned(),
        bundle_id: bundle_id.to_owned(),
        conference_id: payload
            .get("conference_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        bundle_root,
        manifest_path,
        audio_path,
        speaker_jsonl_path,
        speaker_txt_path,
    })
}

pub(super) fn local_recording_retention_policy_from_manifest(
    manifest: &Value,
) -> Option<LocalRecordingRetentionPolicy> {
    let retention = manifest
        .get("provenance")?
        .get("retention_policy")?
        .get("local_recording")?;
    Some(LocalRecordingRetentionPolicy {
        recording_retention_days: retention
            .get("recording_retention_days")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0),
        speaker_hint_retention_days: retention
            .get("speaker_hint_retention_days")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0),
        audio_expires_at: parse_optional_datetime(retention.get("audio_expires_at")),
        speaker_hints_expires_at: parse_optional_datetime(
            retention.get("speaker_hints_expires_at"),
        ),
    })
}

pub(super) fn local_recording_retention_policy_from_days(
    observed_at: DateTime<Utc>,
    recording_retention_days: i64,
    speaker_hint_retention_days: i64,
) -> LocalRecordingRetentionPolicy {
    LocalRecordingRetentionPolicy {
        recording_retention_days,
        speaker_hint_retention_days,
        audio_expires_at: if recording_retention_days > 0 {
            Some(observed_at + chrono::TimeDelta::days(recording_retention_days))
        } else {
            None
        },
        speaker_hints_expires_at: if speaker_hint_retention_days > 0 {
            Some(observed_at + chrono::TimeDelta::days(speaker_hint_retention_days))
        } else {
            None
        },
    }
}

fn parse_optional_datetime(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
        .map(|value| value.with_timezone(&Utc))
}

pub(super) fn remove_local_file_if_exists(path: &Path) -> Result<bool, YandexTelemostError> {
    if !path.exists() || !path.is_file() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

pub(super) fn record_retention_cleanup_in_manifest(
    manifest_path: &Path,
    policy: &LocalRecordingRetentionPolicy,
    audio_removed: bool,
    speaker_removed: bool,
    removed_at: DateTime<Utc>,
) -> Result<(), YandexTelemostError> {
    if !manifest_path.exists() {
        return Ok(());
    }
    let mut manifest: Value = serde_json::from_str(&fs::read_to_string(manifest_path)?)?;
    let Some(provenance) = manifest
        .get_mut("provenance")
        .and_then(Value::as_object_mut)
    else {
        return Ok(());
    };
    provenance.insert(
        "retention_cleanup".to_owned(),
        json!({
            "audio_removed": audio_removed,
            "speaker_hints_removed": speaker_removed,
            "removed_at": removed_at,
            "recording_retention_days": policy.recording_retention_days,
            "speaker_hint_retention_days": policy.speaker_hint_retention_days,
        }),
    );
    fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

pub(super) struct TelemostRetentionCleanupCandidate {
    pub(super) account_id: String,
    pub(super) bundle_id: String,
    pub(super) conference_id: Option<String>,
    pub(super) bundle_root: PathBuf,
    pub(super) manifest_path: PathBuf,
    pub(super) audio_path: PathBuf,
    pub(super) speaker_jsonl_path: PathBuf,
    pub(super) speaker_txt_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LocalRecordingRetentionPolicy {
    pub(super) recording_retention_days: i64,
    pub(super) speaker_hint_retention_days: i64,
    pub(super) audio_expires_at: Option<DateTime<Utc>>,
    pub(super) speaker_hints_expires_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_cleanup_candidate_rejects_relative_paths() {
        let candidate = retention_cleanup_candidate_from_event(
            &json!({
                "account_id": "telemost-main",
                "bundle_id": "bundle-1",
                "bundle_root": "relative/root",
                "manifest_path": "/tmp/manifest.json",
                "audio_path": "/tmp/audio.mp3",
                "speaker_jsonl_path": "/tmp/speaker.jsonl",
                "speaker_txt_path": "/tmp/speaker.txt"
            }),
            Utc::now(),
        );

        assert!(candidate.is_none());
    }

    #[test]
    fn local_recording_retention_policy_from_days_sets_expiry_for_positive_values() {
        let observed_at = Utc::now();
        let policy = local_recording_retention_policy_from_days(observed_at, 7, 3);

        assert_eq!(policy.recording_retention_days, 7);
        assert_eq!(policy.speaker_hint_retention_days, 3);
        assert!(policy.audio_expires_at.is_some());
        assert!(policy.speaker_hints_expires_at.is_some());
    }
}
