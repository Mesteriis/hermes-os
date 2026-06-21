#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaClientConfig {
    pub(in crate::integrations::ollama::client) base_url: String,
    pub(in crate::integrations::ollama::client) chat_model: String,
    pub(in crate::integrations::ollama::client) embed_model: String,
    pub(in crate::integrations::ollama::client) timeout_seconds: u64,
}

impl OllamaClientConfig {
    pub fn new(
        base_url: impl Into<String>,
        chat_model: impl Into<String>,
        embed_model: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            chat_model: chat_model.into(),
            embed_model: embed_model.into(),
            timeout_seconds: 120,
        }
    }

    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}
