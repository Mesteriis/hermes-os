use std::time::Instant;

use serde_json::json;

use super::super::agents::validate_agent;
use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::{
    elapsed_ms, event_id_from_command, run_id_from_command, validate_non_empty,
};
use super::super::prompts::answer_prompt;
use super::super::runs::{AiRunStore, NewAiRun};
use super::super::types::{AiAnswerRequest, AiAnswerResponse};
use super::core::AiService;
use super::events::AiRunEvent;
use crate::ai::hub::AiModelRoute;

impl AiService {
    pub async fn answer(
        &self,
        request: AiAnswerRequest,
        actor_id: &str,
    ) -> Result<AiAnswerResponse, AiError> {
        let command_id = validate_non_empty("command_id", &request.command_id)?;
        let query = validate_non_empty("query", &request.query)?;
        let agent_id = request.agent_id.unwrap_or_else(|| "MNEMOSYNE".to_owned());
        validate_agent(&agent_id)?;
        let started_at = Instant::now();
        let run_id = run_id_from_command("answer", &command_id);
        let requested_event_id = event_id_from_command("ai.run.requested", &command_id);
        let completed_event_id = event_id_from_command("ai.run.completed", &command_id);
        let run_store = AiRunStore::new(self.pool.clone());
        let chat_model = self.model_routing.default_chat.clone();
        let attribution = self.run_attribution(&agent_id).await?;

        run_store
            .start_run(&NewAiRun {
                run_id: run_id.clone(),
                agent_id: agent_id.clone(),
                chat_model: chat_model.clone(),
                embedding_model: self.model_routing.embeddings.clone(),
                prompt_template_version: AI_PROMPT_TEMPLATE_VERSION.to_owned(),
                model_config: self.model_config(),
                query: query.clone(),
                actor_id: actor_id.to_owned(),
                agent_persona_id: Some(attribution.agent_persona_id.clone()),
                owner_persona_id: attribution.owner_persona_id.clone(),
                causation_id: request.causation_id.clone(),
                correlation_id: request.correlation_id.clone(),
                requested_event_id: requested_event_id.clone(),
            })
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &requested_event_id,
            event_type: "ai.run.requested",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({}),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        let result: Result<AiAnswerResponse, AiError> = async {
            let citations = self.retrieve_citations(&query).await?;
            let prompt = answer_prompt(&query, &citations);
            let chat = self.hub.chat(AiModelRoute::DefaultChat, &prompt).await?;
            let duration_ms = elapsed_ms(started_at);
            let stored = run_store
                .complete_run(
                    &run_id,
                    &chat.content,
                    &citations,
                    duration_ms,
                    &completed_event_id,
                )
                .await?;
            self.append_run_event(AiRunEvent {
                event_id: &completed_event_id,
                event_type: "ai.run.completed",
                run_id: &run_id,
                agent_id: &agent_id,
                actor_id,
                query: &query,
                payload: json!({
                    "citation_count": citations.len(),
                    "duration_ms": duration_ms,
                }),
                correlation_id: request.correlation_id.as_deref(),
            })
            .await?;

            Ok(AiAnswerResponse {
                run_id: run_id.clone(),
                agent_id: agent_id.clone(),
                agent_persona_id: attribution.agent_persona_id,
                owner_persona_id: attribution.owner_persona_id,
                status: stored.status,
                answer: chat.content,
                citations,
                model: chat.model,
                embedding_model: self.model_routing.embeddings.clone(),
                created_at: stored.started_at,
                duration_ms,
            })
        }
        .await;

        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                let duration_ms = elapsed_ms(started_at);
                let failed_event_id = event_id_from_command("ai.run.failed", &command_id);
                let error_summary = error.to_string();
                run_store
                    .fail_run(&run_id, &error_summary, duration_ms, &failed_event_id)
                    .await?;
                self.append_run_event(AiRunEvent {
                    event_id: &failed_event_id,
                    event_type: "ai.run.failed",
                    run_id: &run_id,
                    agent_id: &agent_id,
                    actor_id,
                    query: &query,
                    payload: json!({
                        "duration_ms": duration_ms,
                        "reason": error_summary,
                    }),
                    correlation_id: request.correlation_id.as_deref(),
                })
                .await?;
                Err(error)
            }
        }
    }
}
