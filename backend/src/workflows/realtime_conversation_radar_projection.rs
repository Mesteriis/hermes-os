use crate::platform::realtime_conversation::CallBundleManifest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RealtimeConversationRadarSignalCandidate {
    pub signal_kind: String,
    pub title: String,
    pub confidence: f32,
    pub evidence: Value,
    pub promotion_policy: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RealtimeConversationRadarProjectionContext {
    pub live_stream_watch_url: Option<String>,
    pub unknown_cohost_emails: Vec<String>,
    pub recording_session_id: Option<String>,
}

pub fn call_bundle_radar_candidates(
    manifest: &CallBundleManifest,
    projection_context: &RealtimeConversationRadarProjectionContext,
) -> Vec<RealtimeConversationRadarSignalCandidate> {
    let mut candidates = Vec::new();
    let base_evidence = serde_json::json!({
        "bundle_id": manifest.bundle_id,
        "provider_kind": manifest.provider_kind.as_str(),
        "provider_conference_id": manifest.provider_conference_id,
        "join_url": manifest.join_url,
        "calendar_event_id": manifest.calendar_event_id,
        "project_id": manifest.project_id,
        "organization_id": manifest.organization_id,
    });

    if manifest.join_url.is_some()
        && manifest.calendar_event_id.is_none()
        && manifest.project_id.is_none()
        && manifest.organization_id.is_none()
    {
        candidates.push(RealtimeConversationRadarSignalCandidate {
            signal_kind: "unmatched_meeting_link".to_owned(),
            title: "Review unmatched Telemost meeting link".to_owned(),
            confidence: 0.72,
            evidence: serde_json::json!({
                "bundle": base_evidence,
                "reason": "provider_conference_has_no_calendar_project_or_organization_binding",
            }),
            promotion_policy: "radar_review_required_before_calendar_or_project_link".to_owned(),
        });
    }

    if let Some(watch_url) = projection_context.live_stream_watch_url.as_deref() {
        candidates.push(RealtimeConversationRadarSignalCandidate {
            signal_kind: "live_stream_reference".to_owned(),
            title: "Review Telemost live stream reference".to_owned(),
            confidence: 0.78,
            evidence: serde_json::json!({
                "bundle": base_evidence,
                "watch_url": watch_url,
            }),
            promotion_policy: "radar_review_required_before_live_stream_promotion".to_owned(),
        });
    }

    if !projection_context.unknown_cohost_emails.is_empty() {
        candidates.push(RealtimeConversationRadarSignalCandidate {
            signal_kind: "unknown_cohosts".to_owned(),
            title: "Review Telemost cohosts without confirmed persona mapping".to_owned(),
            confidence: 0.68,
            evidence: serde_json::json!({
                "bundle": base_evidence,
                "unknown_cohost_emails": projection_context.unknown_cohost_emails,
            }),
            promotion_policy: "radar_review_required_before_relationship_or_persona_link"
                .to_owned(),
        });
    }

    candidates.push(RealtimeConversationRadarSignalCandidate {
        signal_kind: "local_recording_artifact".to_owned(),
        title: "Review local Telemost recording artifact".to_owned(),
        confidence: 0.88,
        evidence: serde_json::json!({
            "bundle": base_evidence,
            "recording_session_id": projection_context.recording_session_id,
            "artifacts": {
                "audio": manifest.layout.audio_mp3,
                "speaker_hints": manifest.layout.speaker_hints_jsonl,
                "event_track": manifest.layout.event_track_jsonl,
            }
        }),
        promotion_policy: "radar_review_required_before_document_or_memory_promotion".to_owned(),
    });

    candidates
}
