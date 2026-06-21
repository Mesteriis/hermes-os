use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct VersionResponse {
    pub version: String,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct TagsResponse {
    pub models: Vec<TaggedModel>,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct TaggedModel {
    pub name: String,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct ChatResponse {
    pub model: Option<String>,
    pub message: Option<ChatMessage>,
    pub total_duration: Option<u64>,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct ChatMessage {
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct EmbedResponse {
    pub model: Option<String>,
    pub embeddings: Option<Vec<Vec<f32>>>,
    pub embedding: Option<Vec<f32>>,
    pub total_duration: Option<u64>,
}
