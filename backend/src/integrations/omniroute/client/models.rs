#[derive(Clone, Debug, PartialEq)]
pub struct OmniRouteChatResult {
    pub model: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OmniRouteEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
}
