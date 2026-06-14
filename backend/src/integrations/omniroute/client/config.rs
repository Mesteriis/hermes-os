use crate::platform::secrets::ResolvedSecret;

#[derive(Clone)]
pub struct OmniRouteClientConfig {
    pub(super) base_url: String,
    pub(super) chat_model: String,
    pub(super) embed_model: String,
    pub(super) api_key: ResolvedSecret,
    pub(super) timeout_seconds: u64,
}

impl OmniRouteClientConfig {
    pub fn new(
        base_url: impl Into<String>,
        chat_model: impl Into<String>,
        embed_model: impl Into<String>,
        api_key: ResolvedSecret,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            chat_model: chat_model.into(),
            embed_model: embed_model.into(),
            api_key,
            timeout_seconds: 120,
        }
    }

    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}
