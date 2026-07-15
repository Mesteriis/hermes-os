#[derive(Clone, Debug, PartialEq)]
pub struct OllamaChatResult {
    pub model: String,
    pub content: String,
    pub total_duration_ns: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OllamaEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
    pub total_duration_ns: Option<u64>,
}
