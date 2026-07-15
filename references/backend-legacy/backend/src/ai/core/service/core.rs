use sqlx::postgres::PgPool;

use crate::ai::hub::{AiHub, SharedAiHub};
use crate::platform::ai_runtime::SharedAiRuntimePort;

use super::super::types::AiModelRouting;
use super::attribution_port::SharedAiPersonaAttributionPort;

#[derive(Clone)]
pub struct AiService {
    pub(super) pool: PgPool,
    pub(super) hub: SharedAiHub,
    pub(super) chat_model: String,
    pub(super) embedding_model: String,
    pub(super) model_routing: AiModelRouting,
    pub(super) persona_attribution: Option<SharedAiPersonaAttributionPort>,
}

impl AiService {
    pub fn new_with_routing(
        pool: PgPool,
        runtime: SharedAiRuntimePort,
        model_routing: AiModelRouting,
    ) -> Self {
        Self::new_with_hub(pool, AiHub::shared(runtime, model_routing))
    }

    pub fn new_with_hub(pool: PgPool, hub: SharedAiHub) -> Self {
        let model_routing = hub.model_routing().clone();
        Self {
            pool,
            hub,
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
