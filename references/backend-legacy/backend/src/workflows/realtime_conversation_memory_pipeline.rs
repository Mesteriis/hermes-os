use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::engines::call_intelligence::{
    engine::CallIntelligenceEngine, models::CallIntelligencePipelinePlan,
};
use crate::platform::realtime_conversation::models::CallBundleManifest;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RealtimeConversationMemoryPipelinePlan {
    pub bundle_id: String,
    pub account_id: String,
    pub conference_id: Option<String>,
    pub provider_kind: String,
    pub stage: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub audio_path: String,
    pub call_intelligence: CallIntelligencePipelinePlan,
    pub follow_up_events: Vec<String>,
}

pub fn plan_memory_pipeline(
    manifest: &CallBundleManifest,
) -> RealtimeConversationMemoryPipelinePlan {
    let engine = CallIntelligenceEngine;
    RealtimeConversationMemoryPipelinePlan {
        bundle_id: manifest.bundle_id.clone(),
        account_id: manifest.account_id.clone(),
        conference_id: manifest.provider_conference_id.clone(),
        provider_kind: manifest.provider_kind.as_str().to_owned(),
        stage: "queued_after_local_recording".to_owned(),
        bundle_root: manifest.layout.root.clone(),
        manifest_path: Path::new(&manifest.layout.root)
            .join(&manifest.layout.manifest)
            .to_string_lossy()
            .into_owned(),
        audio_path: Path::new(&manifest.layout.root)
            .join(&manifest.layout.audio_mp3)
            .to_string_lossy()
            .into_owned(),
        call_intelligence: engine.plan_from_bundle(manifest),
        follow_up_events: vec![
            "realtime_conversation.transcript.requested".to_owned(),
            "realtime_conversation.knowledge.extracted".to_owned(),
            "realtime_conversation.radar_signals.detected".to_owned(),
        ],
    }
}
