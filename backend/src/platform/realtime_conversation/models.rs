use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeConversationProviderKind {
    YandexTelemost,
    Zoom,
    GoogleMeet,
    Jitsi,
    Discord,
    SignalCalls,
    Unknown,
}

impl RealtimeConversationProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::YandexTelemost => "yandex_telemost",
            Self::Zoom => "zoom",
            Self::GoogleMeet => "google_meet",
            Self::Jitsi => "jitsi",
            Self::Discord => "discord",
            Self::SignalCalls => "signal_calls",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RealtimeConversationProviderCapabilities {
    pub provider_kind: RealtimeConversationProviderKind,
    pub provider_shape: String,
    pub supports_conference_create: bool,
    pub supports_visible_webview: bool,
    pub supports_audio_capture: bool,
    pub supports_participant_events: bool,
    pub supports_speaker_hints: bool,
    pub supports_chat_capture: bool,
    pub supports_screen_share_detection: bool,
    pub supports_screenshot_hints: bool,
    pub supports_recording: bool,
    pub supports_provider_transcript: bool,
    pub supports_reactions: bool,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundleArtifact {
    pub kind: String,
    pub relative_path: String,
    pub source: String,
    pub truth_status: String,
    pub media_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundleLayout {
    pub root: String,
    pub manifest: String,
    pub meeting_json: String,
    pub provider_json: String,
    pub participants_json: String,
    pub audio_mp3: String,
    pub speaker_hints_jsonl: String,
    pub speaker_timeline_txt: String,
    pub event_track_jsonl: String,
    pub chat_json: String,
    pub transcript_json: String,
    pub transcript_markdown: String,
    pub summary_markdown: String,
    pub topics_json: String,
    pub entities_json: String,
    pub decisions_json: String,
    pub tasks_json: String,
    pub knowledge_json: String,
    pub metrics_json: String,
    pub radar_signals_json: String,
    pub screenshots_dir: String,
    pub attachments_dir: String,
    pub ocr_dir: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundlePipelineState {
    pub audio_capture: String,
    pub speaker_hints: String,
    pub transcription: String,
    pub diarization: String,
    pub speaker_identity: String,
    pub topic_timeline: String,
    pub decision_detection: String,
    pub action_detection: String,
    pub screen_intelligence: String,
    pub knowledge_extraction: String,
    pub radar_projection: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundlePrivacyPolicy {
    pub owner_visible_capture_only: bool,
    pub hidden_headless_capture: String,
    pub consent_required: bool,
    pub local_first: bool,
    pub provider_dom_truth_status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallBundleManifest {
    pub schema_version: u16,
    pub bundle_id: String,
    pub provider_kind: RealtimeConversationProviderKind,
    pub provider_shape: String,
    pub account_id: String,
    pub provider_conference_id: Option<String>,
    pub join_url: Option<String>,
    pub calendar_event_id: Option<String>,
    pub project_id: Option<String>,
    pub organization_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub layout: CallBundleLayout,
    pub artifacts: Vec<CallBundleArtifact>,
    pub pipeline_state: CallBundlePipelineState,
    pub privacy_policy: CallBundlePrivacyPolicy,
    pub provenance: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerTimelineHint {
    pub observed_at_ms: i64,
    pub speaker_label: String,
    pub source: String,
    pub confidence: f32,
    pub truth_status: String,
    pub provider_participant_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MeetingTimelineEvent {
    pub occurred_at_ms: i64,
    pub event_kind: String,
    pub label: String,
    pub confidence: f32,
    pub source: String,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TopicTimelineSegment {
    pub starts_at_ms: i64,
    pub ends_at_ms: Option<i64>,
    pub title: String,
    pub summary: String,
    pub confidence: f32,
    pub evidence: Value,
}

impl CallBundlePipelineState {
    pub fn queued_from_local_recording() -> Self {
        Self {
            audio_capture: "running_or_completed".to_owned(),
            speaker_hints: "collecting_hint_not_truth".to_owned(),
            transcription: "queued".to_owned(),
            diarization: "queued".to_owned(),
            speaker_identity: "queued".to_owned(),
            topic_timeline: "queued".to_owned(),
            decision_detection: "queued".to_owned(),
            action_detection: "queued".to_owned(),
            screen_intelligence: "queued".to_owned(),
            knowledge_extraction: "queued".to_owned(),
            radar_projection: "queued".to_owned(),
        }
    }
}

impl CallBundlePrivacyPolicy {
    pub fn local_visible_capture() -> Self {
        Self {
            owner_visible_capture_only: true,
            hidden_headless_capture: "forbidden".to_owned(),
            consent_required: true,
            local_first: true,
            provider_dom_truth_status: "hint_not_truth".to_owned(),
        }
    }
}

impl RealtimeConversationProviderCapabilities {
    pub fn evidence_source(source: &'static str) -> Value {
        json!({ "source": source, "confidence": "provider_contract_declared" })
    }
}
