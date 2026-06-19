use super::super::errors::AiError;
use super::super::helpers::{ai_task_candidate_id, validate_non_empty};
use super::super::prompts::{AiTaskCandidateDraft, citation_for_draft};
use super::super::types::AiCitation;
use super::core::AiService;

impl AiService {
    pub(super) async fn upsert_ai_task_candidates(
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
            let observation_id = task_candidate_observation_id(self, citation).await?;
            let Some(observation_id) = observation_id else {
                continue;
            };
            let task_candidate_id = ai_task_candidate_id("observation", &observation_id, &title);

            let result = sqlx::query(
                r#"
                INSERT INTO task_candidates (
                    task_candidate_id,
                    source_kind,
                    source_id,
                    observation_id,
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
                    $1, $2, $3, $4, NULL, $5, $6, $7, $8, 'suggested', $9, NULL, NULL, NULL, $10
                )
                ON CONFLICT (source_kind, source_id, lower(title))
                DO UPDATE SET
                    observation_id = COALESCE(EXCLUDED.observation_id, task_candidates.observation_id),
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
            .bind("observation")
            .bind(&observation_id)
            .bind(&observation_id)
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
}

async fn task_candidate_observation_id(
    service: &AiService,
    citation: &AiCitation,
) -> Result<Option<String>, AiError> {
    match citation.source_kind.as_str() {
        "message" => {
            let observation_id = sqlx::query_scalar::<_, Option<String>>(
                r#"
                SELECT observation_id
                FROM communication_messages
                WHERE message_id = $1
                "#,
            )
            .bind(&citation.source_id)
            .fetch_optional(&service.pool)
            .await?
            .flatten();
            Ok(observation_id)
        }
        "document" => {
            let observation_id = sqlx::query_scalar::<_, Option<String>>(
                r#"
                SELECT observation_id
                FROM documents
                WHERE document_id = $1
                "#,
            )
            .bind(&citation.source_id)
            .fetch_optional(&service.pool)
            .await?
            .flatten();
            Ok(observation_id)
        }
        _ => Ok(None),
    }
}
