use std::time::Instant;

use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use crate::domains::persons::api::PersonProjectionStore;
use crate::integrations::ai_runtime::AiRuntimeClient;
use crate::platform::events::{EventStore, NewEventEnvelope};

use super::agents::{ai_agent_display_name, validate_agent};
use super::constants::{
    AI_EMBEDDING_DIMENSION, AI_PROMPT_TEMPLATE_VERSION, DEFAULT_RETRIEVAL_LIMIT,
};
use super::errors::AiError;
use super::helpers::{
    ai_task_candidate_id, elapsed_ms, event_id_from_command, merge_retrieval_results,
    run_id_from_command, text_preview, validate_non_empty,
};
use super::prompts::{
    AiTaskCandidateDraft, answer_prompt, citation_for_draft, meeting_prep_prompt,
    parse_task_candidate_drafts, scoped_meeting_query, task_candidate_prompt,
};
use super::runs::{AiRunStore, NewAiRun};
use super::semantic::SemanticEmbeddingStore;
use super::types::{
    AiAnswerRequest, AiAnswerResponse, AiCitation, AiMeetingPrepRequest, AiMeetingPrepResponse,
    AiModelRouting, AiStatusResponse, AiTaskCandidateRefreshRequest,
    AiTaskCandidateRefreshResponse,
};

#[derive(Clone)]
pub struct AiService {
    pool: PgPool,
    runtime: AiRuntimeClient,
    chat_model: String,
    embedding_model: String,
    model_routing: AiModelRouting,
}

struct AiRunEvent<'a> {
    event_id: &'a str,
    event_type: &'a str,
    run_id: &'a str,
    agent_id: &'a str,
    actor_id: &'a str,
    query: &'a str,
    payload: Value,
    correlation_id: Option<&'a str>,
}

struct AiRunAttribution {
    agent_persona_id: String,
    owner_persona_id: Option<String>,
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
        }
    }

    pub async fn status(&self) -> AiStatusResponse {
        let version = self.runtime.version().await;
        let models = self.runtime.models().await;
        let chat_model_available = models
            .as_ref()
            .map(|models| {
                models
                    .iter()
                    .any(|model| model == &self.model_routing.default_chat)
            })
            .unwrap_or(false);
        let embedding_model_available = models
            .as_ref()
            .map(|models| {
                models
                    .iter()
                    .any(|model| model == &self.model_routing.embeddings)
            })
            .unwrap_or(false);

        AiStatusResponse {
            runtime: self.runtime.runtime_name().to_owned(),
            status: if version.is_ok()
                && models.is_ok()
                && chat_model_available
                && embedding_model_available
            {
                "ok"
            } else {
                "unavailable"
            }
            .to_owned(),
            version: version.ok().flatten(),
            chat_model: self.model_routing.default_chat.clone(),
            embedding_model: self.model_routing.embeddings.clone(),
            embedding_dimension: AI_EMBEDDING_DIMENSION,
            chat_model_available,
            embedding_model_available,
        }
    }

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

        let citations = self.retrieve_citations(&query).await?;
        let prompt = answer_prompt(&query, &citations);
        let chat = self.runtime.chat_with_model(&prompt, &chat_model).await?;
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
            run_id,
            agent_id,
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

    pub async fn refresh_task_candidates(
        &self,
        request: AiTaskCandidateRefreshRequest,
        actor_id: &str,
    ) -> Result<AiTaskCandidateRefreshResponse, AiError> {
        let command_id = validate_non_empty("command_id", &request.command_id)?;
        let query = validate_non_empty("query", &request.query)?;
        let agent_id = "HERMES".to_owned();
        let started_at = Instant::now();
        let run_id = run_id_from_command("task-refresh", &command_id);
        let requested_event_id = event_id_from_command("ai.run.requested", &command_id);
        let completed_event_id = event_id_from_command("ai.run.completed", &command_id);
        let extraction_event_id =
            event_id_from_command("ai.task_extraction.completed", &command_id);
        let run_store = AiRunStore::new(self.pool.clone());
        let chat_model = self.model_routing.extraction.clone();
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
            payload: json!({ "workflow": "task_candidates" }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        let citations = self.retrieve_citations(&query).await?;
        let prompt = task_candidate_prompt(&query, &citations);
        let chat = self.runtime.chat_with_model(&prompt, &chat_model).await?;
        let drafts = parse_task_candidate_drafts(&chat.content, &citations)?;
        let created_count = self
            .upsert_ai_task_candidates(&run_id, &drafts, &citations)
            .await?;
        let duration_ms = elapsed_ms(started_at);
        let answer = format!("Created {created_count} suggested task candidate(s).");
        let stored = run_store
            .complete_run(
                &run_id,
                &answer,
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
                "workflow": "task_candidates",
                "created_count": created_count,
                "duration_ms": duration_ms,
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;
        self.append_run_event(AiRunEvent {
            event_id: &extraction_event_id,
            event_type: "ai.task_extraction.completed",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({
                "created_count": created_count,
                "candidate_state": "suggested",
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        Ok(AiTaskCandidateRefreshResponse {
            run_id,
            agent_id,
            agent_persona_id: attribution.agent_persona_id,
            owner_persona_id: attribution.owner_persona_id,
            status: stored.status,
            created_count,
            citations,
            model: chat.model,
            embedding_model: self.model_routing.embeddings.clone(),
            created_at: stored.started_at,
            duration_ms,
        })
    }

    pub async fn meeting_prep(
        &self,
        request: AiMeetingPrepRequest,
        actor_id: &str,
    ) -> Result<AiMeetingPrepResponse, AiError> {
        let command_id = validate_non_empty("command_id", &request.command_id)?;
        let topic = validate_non_empty("topic", &request.topic)?;
        let agent_id = "HESTIA".to_owned();
        let started_at = Instant::now();
        let run_id = run_id_from_command("meeting-prep", &command_id);
        let requested_event_id = event_id_from_command("ai.run.requested", &command_id);
        let completed_event_id = event_id_from_command("ai.run.completed", &command_id);
        let run_store = AiRunStore::new(self.pool.clone());
        let chat_model = self.model_routing.meeting_prep.clone();
        let query = scoped_meeting_query(
            &topic,
            request.project_id.as_deref(),
            request.person_id.as_deref(),
        );
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
            payload: json!({
                "workflow": "meeting_prep",
                "project_id": request.project_id,
                "person_id": request.person_id,
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        let citations = self.retrieve_citations(&query).await?;
        let prompt = meeting_prep_prompt(&topic, &citations);
        let chat = self.runtime.chat_with_model(&prompt, &chat_model).await?;
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
                "workflow": "meeting_prep",
                "citation_count": citations.len(),
                "duration_ms": duration_ms,
            }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        Ok(AiMeetingPrepResponse {
            run_id,
            agent_id,
            agent_persona_id: attribution.agent_persona_id,
            owner_persona_id: attribution.owner_persona_id,
            status: stored.status,
            briefing: chat.content,
            citations,
            model: chat.model,
            embedding_model: self.model_routing.embeddings.clone(),
            created_at: stored.started_at,
            duration_ms,
        })
    }

    async fn run_attribution(&self, agent_id: &str) -> Result<AiRunAttribution, AiError> {
        let person_store = PersonProjectionStore::new(self.pool.clone());
        let agent_persona = person_store
            .upsert_ai_agent_persona(agent_id, ai_agent_display_name(agent_id)?)
            .await?;
        let owner_persona_id = person_store
            .owner_persona()
            .await?
            .map(|owner| owner.person_id);

        Ok(AiRunAttribution {
            agent_persona_id: agent_persona.person_id,
            owner_persona_id,
        })
    }

    async fn retrieve_citations(&self, query: &str) -> Result<Vec<AiCitation>, AiError> {
        let semantic_store = SemanticEmbeddingStore::new(self.pool.clone());
        let embedding_model = &self.model_routing.embeddings;
        semantic_store
            .index_canonical_sources(&self.runtime, embedding_model)
            .await?;
        let query_embedding = self
            .runtime
            .embed_with_model(query, embedding_model)
            .await?;
        if query_embedding.embedding.len() != AI_EMBEDDING_DIMENSION {
            return Err(AiError::InvalidEmbeddingDimension {
                expected: AI_EMBEDDING_DIMENSION,
                actual: query_embedding.embedding.len(),
            });
        }

        let vector_results = semantic_store
            .search(
                embedding_model,
                &query_embedding.embedding,
                DEFAULT_RETRIEVAL_LIMIT,
            )
            .await?;
        let text_results = semantic_store
            .text_search(embedding_model, query, DEFAULT_RETRIEVAL_LIMIT)
            .await?;
        let merged = merge_retrieval_results(vector_results, text_results);

        Ok(merged
            .into_iter()
            .take(DEFAULT_RETRIEVAL_LIMIT as usize)
            .map(AiCitation::from)
            .collect())
    }

    async fn upsert_ai_task_candidates(
        &self,
        run_id: &str,
        drafts: &[AiTaskCandidateDraft],
        citations: &[AiCitation],
    ) -> Result<i64, AiError> {
        let mut created_count = 0;
        for draft in drafts {
            let Some(citation) = citation_for_draft(draft, citations) else {
                continue;
            };
            if citation.source_kind != "message" && citation.source_kind != "document" {
                continue;
            }
            let title = validate_non_empty("title", &draft.title)?;
            let evidence_excerpt = draft
                .evidence_excerpt
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(citation.excerpt.as_str());
            let confidence = draft.confidence.unwrap_or(0.5).clamp(0.0, 1.0);
            let task_candidate_id =
                ai_task_candidate_id(&citation.source_kind, &citation.source_id, &title);

            let result = sqlx::query(
                r#"
                INSERT INTO task_candidates (
                    task_candidate_id,
                    source_kind,
                    source_id,
                    project_id,
                    title,
                    due_text,
                    assignee_label,
                    confidence,
                    review_state,
                    evidence_excerpt,
                    event_id,
                    actor_id,
                    reviewed_at,
                    agent_run_id
                )
                VALUES (
                    $1, $2, $3, NULL, $4, $5, $6, $7, 'suggested', $8, NULL, NULL, NULL, $9
                )
                ON CONFLICT (task_candidate_id)
                DO UPDATE SET
                    title = EXCLUDED.title,
                    due_text = COALESCE(EXCLUDED.due_text, task_candidates.due_text),
                    assignee_label = COALESCE(EXCLUDED.assignee_label, task_candidates.assignee_label),
                    confidence = EXCLUDED.confidence,
                    review_state = CASE
                        WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                            THEN task_candidates.review_state
                        ELSE 'suggested'
                    END,
                    evidence_excerpt = EXCLUDED.evidence_excerpt,
                    agent_run_id = CASE
                        WHEN task_candidates.review_state IN ('user_confirmed', 'user_rejected')
                            THEN task_candidates.agent_run_id
                        ELSE EXCLUDED.agent_run_id
                    END,
                    updated_at = now()
                "#,
            )
            .bind(task_candidate_id)
            .bind(&citation.source_kind)
            .bind(&citation.source_id)
            .bind(&title)
            .bind(&draft.due_text)
            .bind(&draft.assignee_label)
            .bind(confidence)
            .bind(evidence_excerpt)
            .bind(run_id)
            .execute(&self.pool)
            .await?;

            if result.rows_affected() > 0 {
                created_count += 1;
            }
        }

        Ok(created_count)
    }

    async fn append_run_event(&self, event: AiRunEvent<'_>) -> Result<(), AiError> {
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

    fn model_config(&self) -> Value {
        json!({
            "runtime": self.runtime.runtime_name(),
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
