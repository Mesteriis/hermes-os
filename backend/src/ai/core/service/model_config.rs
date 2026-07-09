use serde_json::{Value, json};

use super::super::constants::AI_EMBEDDING_DIMENSION;
use super::core::AiService;

impl AiService {
    pub(super) fn model_config(&self) -> Value {
        json!({
            "runtime": self.hub.runtime_name(),
            "chat_model": &self.model_routing.default_chat,
            "embedding_model": &self.model_routing.embeddings,
            "embedding_dimension": AI_EMBEDDING_DIMENSION,
            "routes": {
                "default_chat": &self.model_routing.default_chat,
                "reasoning": &self.model_routing.reasoning,
                "summarization": &self.model_routing.summarization,
                "mail_intelligence": &self.model_routing.mail_intelligence,
                "reply_draft": &self.model_routing.reply_draft,
                "extraction": &self.model_routing.extraction,
                "embeddings": &self.model_routing.embeddings,
                "meeting_prep": &self.model_routing.meeting_prep,
            }
        })
    }
}
