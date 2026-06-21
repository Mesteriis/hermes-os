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
}

impl AiModelRouting {
    pub fn fallback(chat_model: impl Into<String>, embedding_model: impl Into<String>) -> Self {
        let chat_model = chat_model.into();
        let embedding_model = embedding_model.into();
        Self {
            default_chat: chat_model.clone(),
            reasoning: chat_model.clone(),
            summarization: chat_model.clone(),
            mail_intelligence: chat_model.clone(),
            reply_draft: chat_model.clone(),
            extraction: chat_model.clone(),
            embeddings: embedding_model,
            meeting_prep: chat_model,
        }
    }
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
    pub person_id: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
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
