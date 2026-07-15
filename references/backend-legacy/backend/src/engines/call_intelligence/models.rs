use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallIntelligenceArtifactRequirement {
    pub kind: String,
    pub required: bool,
    pub purpose: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallIntelligenceStep {
    pub step_id: String,
    pub title: String,
    pub input_artifacts: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub source_of_truth_policy: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallIntelligencePipelinePlan {
    pub bundle_id: String,
    pub requirements: Vec<CallIntelligenceArtifactRequirement>,
    pub steps: Vec<CallIntelligenceStep>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CallIntelligenceOutputCandidate {
    pub candidate_kind: String,
    pub title: String,
    pub confidence: f32,
    pub evidence: Value,
}
