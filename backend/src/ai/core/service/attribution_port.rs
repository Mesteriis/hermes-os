use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use thiserror::Error;

pub type SharedAiPersonaAttributionPort = Arc<dyn AiPersonaAttributionPort>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiAgentPersonaAttribution {
    pub persona_id: String,
    pub persona_type: &'static str,
    pub persona_email: String,
}

#[derive(Debug, Error)]
pub enum AiPersonaAttributionError {
    #[error("AI persona attribution failed: {0}")]
    Store(String),
}

pub trait AiPersonaAttributionPort: Send + Sync {
    fn upsert_ai_agent_persona<'a>(
        &'a self,
        agent_id: &'a str,
        display_name: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AiAgentPersonaAttribution, AiPersonaAttributionError>>
                + Send
                + 'a,
        >,
    >;

    fn owner_persona_id<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, AiPersonaAttributionError>> + Send + 'a>>;
}
