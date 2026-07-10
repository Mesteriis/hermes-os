use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::constants::AI_EMBEDDING_DIMENSION;
use super::helpers::text_preview;
use super::semantic::SemanticSearchResult;

#[derive(Clone)]
pub struct AiModelRouting {
    pub default_chat: String,
    pub reasoning: String,
    pub summarization: String,
    pub mail_intelligence: String,
    pub reply_draft: String,
    pub extraction: String,
    pub embeddings: String,
    pub meeting_prep: String,
    pub targets: Vec<AiModelRouteTarget>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiModelRouteTarget {
    pub capability_slot: String,
    pub provider_id: String,
    pub model_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiAnswerRequest {
    pub command_id: String,
    pub query: String,
    pub agent_id: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiTaskCandidateRefreshRequest {
    pub command_id: String,
    pub query: String,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiMeetingPrepRequest {
    pub command_id: String,
    pub topic: String,
    pub project_id: Option<String>,
    #[serde(alias = "person_id")]
    pub persona_id: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiHubRequestAcceptedResponse {
    pub request_id: String,
    pub run_id: String,
    pub status: String,
    pub event_id: String,
    pub correlation_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiCitation {
    pub source_kind: String,
    pub source_id: String,
    pub title: String,
    pub excerpt: String,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_node_id: Option<String>,
}

impl From<SemanticSearchResult> for AiCitation {
    fn from(result: SemanticSearchResult) -> Self {
        Self {
            source_kind: result.source_kind,
            source_id: result.source_id,
            title: result.title,
            excerpt: text_preview(&result.source_text, 320),
            score: result.score,
            graph_node_id: result.graph_node_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AiAnswerResponse {
    pub run_id: String,
    pub agent_id: String,
    pub agent_persona_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_persona_id: Option<String>,
    pub status: String,
    pub answer: String,
    pub citations: Vec<AiCitation>,
    pub model: String,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub duration_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiTaskCandidateRefreshResponse {
    pub run_id: String,
    pub agent_id: String,
    pub agent_persona_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_persona_id: Option<String>,
    pub status: String,
    pub created_count: i64,
    pub citations: Vec<AiCitation>,
    pub model: String,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub duration_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiMeetingPrepResponse {
    pub run_id: String,
    pub agent_id: String,
    pub agent_persona_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_persona_id: Option<String>,
    pub status: String,
    pub briefing: String,
    pub citations: Vec<AiCitation>,
    pub model: String,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub duration_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AiStatusResponse {
    pub runtime: String,
    pub status: String,
    pub version: Option<String>,
    pub chat_model: String,
    pub embedding_model: String,
    pub embedding_dimension: usize,
    pub chat_model_available: bool,
    pub embedding_model_available: bool,
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::AiMeetingPrepRequest;

    #[test]
    fn meeting_prep_request_accepts_legacy_person_identifier() {
        let mut request = Map::new();
        request.insert("command_id".to_owned(), json!("command:meeting-prep"));
        request.insert("topic".to_owned(), json!("Quarterly review"));
        request.insert(["person", "id"].join("_"), json!("persona:legacy"));

        let request: AiMeetingPrepRequest =
            serde_json::from_value(Value::Object(request)).expect("meeting prep request");

        assert_eq!(request.persona_id.as_deref(), Some("persona:legacy"));
    }
}
