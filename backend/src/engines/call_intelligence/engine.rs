use crate::platform::realtime_conversation::CallBundleManifest;

use super::models::{
    CallIntelligenceArtifactRequirement, CallIntelligencePipelinePlan, CallIntelligenceStep,
};

#[derive(Clone, Debug, Default)]
pub struct CallIntelligenceEngine;

impl CallIntelligenceEngine {
    pub fn plan_from_bundle(&self, manifest: &CallBundleManifest) -> CallIntelligencePipelinePlan {
        CallIntelligencePipelinePlan {
            bundle_id: manifest.bundle_id.clone(),
            requirements: vec![
                CallIntelligenceArtifactRequirement {
                    kind: "audio.mp3".to_owned(),
                    required: true,
                    purpose: "transcription and diarization".to_owned(),
                },
                CallIntelligenceArtifactRequirement {
                    kind: "speaker-hints.jsonl".to_owned(),
                    required: false,
                    purpose: "warm-start speaker count and possible human labels".to_owned(),
                },
                CallIntelligenceArtifactRequirement {
                    kind: "screenshots".to_owned(),
                    required: false,
                    purpose: "screen intelligence, OCR and visual evidence".to_owned(),
                },
                CallIntelligenceArtifactRequirement {
                    kind: "chat.json".to_owned(),
                    required: false,
                    purpose: "meeting chat evidence and shared links/files".to_owned(),
                },
            ],
            steps: vec![
                step(
                    "transcribe",
                    "Transcribe MP3",
                    ["audio.mp3"],
                    ["transcript.json", "transcript.md"],
                    "audio_is_capture_artifact",
                ),
                step(
                    "diarize",
                    "Diarize speakers",
                    ["audio.mp3", "speaker-hints.jsonl"],
                    ["speaker-timeline.json"],
                    "speaker_hints_are_not_truth",
                ),
                step(
                    "identify_speakers",
                    "Merge speaker identities",
                    [
                        "speaker-timeline.json",
                        "participants.json",
                        "calendar_event",
                    ],
                    ["speaker-identities.json"],
                    "confidence_weighted_identity_merge",
                ),
                step(
                    "topics",
                    "Build topic timeline",
                    ["transcript.json"],
                    ["topics.json"],
                    "ai_candidate_with_evidence",
                ),
                step(
                    "decisions",
                    "Detect decisions",
                    ["transcript.json", "topics.json"],
                    ["decisions.json"],
                    "candidate_not_domain_truth",
                ),
                step(
                    "actions",
                    "Detect action items",
                    ["transcript.json", "speaker-identities.json"],
                    ["tasks.json"],
                    "radar_review_before_task",
                ),
                step(
                    "screen_intelligence",
                    "Analyze screenshots and OCR",
                    ["screenshots"],
                    ["ocr/", "visual-evidence.json"],
                    "screenshot_is_evidence_not_context_by_itself",
                ),
                step(
                    "knowledge",
                    "Extract meeting knowledge",
                    [
                        "transcript.json",
                        "decisions.json",
                        "tasks.json",
                        "visual-evidence.json",
                    ],
                    ["knowledge.json", "summary.md"],
                    "memory_candidate_requires_provenance",
                ),
                step(
                    "radar",
                    "Project important findings to Radar",
                    ["knowledge.json", "tasks.json", "decisions.json"],
                    ["radar-signals.json"],
                    "review_required_before_promotion",
                ),
            ],
        }
    }
}

fn step<const I: usize, const O: usize>(
    step_id: &'static str,
    title: &'static str,
    input_artifacts: [&'static str; I],
    output_artifacts: [&'static str; O],
    source_of_truth_policy: &'static str,
) -> CallIntelligenceStep {
    CallIntelligenceStep {
        step_id: step_id.to_owned(),
        title: title.to_owned(),
        input_artifacts: input_artifacts.into_iter().map(str::to_owned).collect(),
        output_artifacts: output_artifacts.into_iter().map(str::to_owned).collect(),
        source_of_truth_policy: source_of_truth_policy.to_owned(),
    }
}
