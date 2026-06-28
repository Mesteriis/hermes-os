use std::collections::BTreeMap;

use super::models::{
    SpeakerEvidence, SpeakerIdentityCandidate, SpeakerIdentityMergePlan, SpeakerIdentitySource,
};

#[derive(Clone, Debug, Default)]
pub struct SpeakerIdentityEngine;

impl SpeakerIdentityEngine {
    pub fn merge(&self, evidence: &[SpeakerEvidence]) -> SpeakerIdentityMergePlan {
        let mut grouped: BTreeMap<String, Vec<&SpeakerEvidence>> = BTreeMap::new();
        for item in evidence {
            let key = item
                .person_id
                .clone()
                .unwrap_or_else(|| normalize_label(&item.label));
            grouped.entry(key).or_default().push(item);
        }

        let candidates = grouped
            .into_iter()
            .map(|(key, items)| {
                let evidence_count = items.len();
                let weighted_sum: f32 = items
                    .iter()
                    .map(|item| source_weight(item.source) * item.confidence.clamp(0.0, 1.0))
                    .sum();
                let weight_total: f32 = items.iter().map(|item| source_weight(item.source)).sum();
                let confidence = if weight_total > 0.0 {
                    weighted_sum / weight_total
                } else {
                    0.0
                };
                let display_label = items
                    .iter()
                    .find(|item| !item.label.trim().is_empty())
                    .map(|item| item.label.trim().to_owned())
                    .unwrap_or_else(|| "Unknown speaker".to_owned());
                SpeakerIdentityCandidate {
                    speaker_key: key,
                    display_label,
                    person_id: items.iter().find_map(|item| item.person_id.clone()),
                    confidence,
                    evidence_count,
                    requires_review: confidence < 0.8,
                }
            })
            .collect::<Vec<_>>();
        let unknown_speaker_count = candidates
            .iter()
            .filter(|candidate| candidate.person_id.is_none())
            .count();
        SpeakerIdentityMergePlan {
            candidates,
            unknown_speaker_count,
            policy: "dom_webview_hints_are_supporting_evidence_not_truth".to_owned(),
        }
    }
}

fn source_weight(source: SpeakerIdentitySource) -> f32 {
    match source {
        SpeakerIdentitySource::ManualConfirmation => 1.0,
        SpeakerIdentitySource::VoiceEmbedding => 0.85,
        SpeakerIdentitySource::CalendarAttendee => 0.55,
        SpeakerIdentitySource::ProviderParticipant => 0.5,
        SpeakerIdentitySource::WhisperDiarization => 0.45,
        SpeakerIdentitySource::WebviewDomHint => 0.25,
    }
}

fn normalize_label(value: &str) -> String {
    let normalized = value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if normalized.is_empty() {
        "unknown-speaker".to_owned()
    } else {
        normalized
    }
}
