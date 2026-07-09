mod agents;
mod constants;
mod errors;
mod evidence;
mod helpers;
mod prompts;
mod runs;
mod semantic;
mod service;
mod types;

pub use agents::{AiAgentDescriptor, AiAgentListResponse, v3_agents};
pub use constants::AI_EMBEDDING_DIMENSION;
pub(crate) use constants::AI_PROMPT_TEMPLATE_VERSION;
pub use errors::AiError;
pub(crate) use helpers::{event_id_from_command, run_id_from_command, text_preview};
pub use runs::{AiAgentRun, AiRunStore, NewAiRun};
pub use semantic::{
    NewSemanticEmbedding, SemanticEmbedding, SemanticEmbeddingStore, SemanticIndexReport,
    SemanticSearchResult, SemanticSourceKind,
};
pub use service::{
    AiAgentPersonaAttribution, AiPersonaAttributionError, AiPersonaAttributionPort, AiService,
    SharedAiPersonaAttributionPort,
};
pub use types::{
    AiAnswerRequest, AiAnswerResponse, AiCitation, AiHubRequestAcceptedResponse,
    AiMeetingPrepRequest, AiMeetingPrepResponse, AiModelRouteTarget, AiModelRouting,
    AiStatusResponse, AiTaskCandidateRefreshRequest, AiTaskCandidateRefreshResponse,
};
