use sqlx::postgres::PgPool;

use crate::integrations::ai_runtime::AiRuntimeClient;

use super::super::types::AiModelRouting;
use super::attribution_port::SharedAiPersonaAttributionPort;

#[derive(Clone)]
pub struct AiService {
    pub(super) pool: PgPool,
    pub(super) runtime: AiRuntimeClient,
    pub(super) chat_model: String,
    pub(super) embedding_model: String,
    pub(super) model_routing: AiModelRouting,
    pub(super) persona_attribution: Option<SharedAiPersonaAttributionPort>,
}

impl AiService {
    pub fn new(
        pool: PgPool,
        runtime: AiRuntimeClient,
        chat_model: impl Into<String>,
        embedding_model: impl Into<String>,
    ) -> Self {
        let chat_model = chat_model.into();
        let embedding_model = embedding_model.into();
        let model_routing = AiModelRouting::fallback(chat_model.clone(), embedding_model.clone());
        Self::new_with_routing(pool, runtime, model_routing)
    }

    pub fn new_with_routing(
        pool: PgPool,
        runtime: AiRuntimeClient,
        model_routing: AiModelRouting,
    ) -> Self {
        Self {
            pool,
            runtime,
            chat_model: model_routing.default_chat.clone(),
            embedding_model: model_routing.embeddings.clone(),
            model_routing,
            persona_attribution: None,
        }
    }

    pub fn with_persona_attribution(
        mut self,
        persona_attribution: SharedAiPersonaAttributionPort,
    ) -> Self {
        self.persona_attribution = Some(persona_attribution);
        self
    }
}
