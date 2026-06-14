use chrono::Utc;
use serde_json::{Value, json};

use crate::platform::events::{EventStore, NewEventEnvelope};

use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::text_preview;
use super::core::AiService;

pub(super) struct AiRunEvent<'a> {
    pub(super) event_id: &'a str,
    pub(super) event_type: &'a str,
    pub(super) run_id: &'a str,
    pub(super) agent_id: &'a str,
    pub(super) actor_id: &'a str,
    pub(super) query: &'a str,
    pub(super) payload: Value,
    pub(super) correlation_id: Option<&'a str>,
}

impl AiService {
    pub(super) async fn append_run_event(&self, event: AiRunEvent<'_>) -> Result<(), AiError> {
        let builder = NewEventEnvelope::builder(
            event.event_id,
            event.event_type,
            Utc::now(),
            json!({
                "kind": "ai_run",
                "source_id": event.run_id,
            }),
            json!({
                "kind": "ai_run",
                "run_id": event.run_id,
                "agent_id": event.agent_id,
            }),
        )
        .actor(json!({ "actor_id": event.actor_id }))
        .payload(json!({
            "agent_id": event.agent_id,
            "query_preview": text_preview(event.query, 160),
            "details": event.payload,
        }))
        .provenance(json!({
            "runtime": self.runtime.runtime_name(),
            "chat_model": self.chat_model,
            "embedding_model": self.embedding_model,
            "prompt_template_version": AI_PROMPT_TEMPLATE_VERSION,
        }));
        let builder = if let Some(correlation_id) = event.correlation_id {
            builder.correlation_id(correlation_id)
        } else {
            builder
        };
        EventStore::new(self.pool.clone())
            .append(&builder.build()?)
            .await?;
        Ok(())
    }
}
