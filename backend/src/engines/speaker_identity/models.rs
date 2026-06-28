use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerIdentitySource {
    WebviewDomHint,
    WhisperDiarization,
    VoiceEmbedding,
    CalendarAttendee,
    ProviderParticipant,
    ManualConfirmation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerEvidence {
    pub source: SpeakerIdentitySource,
    pub label: String,
    pub person_id: Option<String>,
    pub starts_at_ms: Option<i64>,
    pub ends_at_ms: Option<i64>,
    pub confidence: f32,
    pub evidence: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerIdentityCandidate {
    pub speaker_key: String,
    pub display_label: String,
    pub person_id: Option<String>,
    pub confidence: f32,
    pub evidence_count: usize,
    pub requires_review: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerIdentityMergePlan {
    pub candidates: Vec<SpeakerIdentityCandidate>,
    pub unknown_speaker_count: usize,
    pub policy: String,
}
