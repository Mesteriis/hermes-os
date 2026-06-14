use crate::domains::mail::messages::{MessageProjectionStore, ProjectedMessage, WorkflowState};
use crate::integrations::ai_runtime::AiRuntimeClient;
use crate::workflows::email_intelligence::errors::EmailIntelligenceError;
use crate::workflows::email_intelligence::heuristics;
use crate::workflows::email_intelligence::models::EmailAnalysis;
use crate::workflows::email_intelligence::prompt::{
    EMAIL_INTELLIGENCE_PROMPT_VERSION, build_email_analysis_prompt,
};

#[derive(Clone)]
pub struct EmailIntelligenceService {
    runtime: Option<AiRuntimeClient>,
}

impl EmailIntelligenceService {
    pub fn new(runtime: Option<AiRuntimeClient>) -> Self {
        Self { runtime }
    }

    pub async fn analyze_message(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Option<EmailAnalysis>, EmailIntelligenceError> {
        let Some(ref runtime) = self.runtime else {
            return Ok(None);
        };

        let prompt = build_email_analysis_prompt(message);
        let result = runtime.chat(&prompt).await?;
        let mut analysis: EmailAnalysis =
            serde_json::from_str(clean_json_response(&result.content))
                .map_err(|e| EmailIntelligenceError::ParseError(e.to_string()))?;

        analysis.model = result.model;
        analysis.prompt_version = EMAIL_INTELLIGENCE_PROMPT_VERSION.to_owned();

        Ok(Some(analysis))
    }

    pub async fn analyze_and_persist(
        &self,
        store: &MessageProjectionStore,
        message: &ProjectedMessage,
    ) -> Result<bool, EmailIntelligenceError> {
        let Some(analysis) = self.analyze_message(message).await? else {
            return Ok(false);
        };

        let workflow_hint = if analysis.is_spam || analysis.is_phishing {
            Some(WorkflowState::Spam)
        } else if analysis.importance_score >= 80 {
            Some(WorkflowState::NeedsAction)
        } else {
            None
        };

        store
            .set_ai_analysis(
                &message.message_id,
                Some(&analysis.category),
                Some(&analysis.summary),
                Some(analysis.importance_score),
            )
            .await?;

        if let Some(state) = workflow_hint {
            let _ = store
                .transition_workflow_state(&message.message_id, state)
                .await;
        }

        Ok(true)
    }

    pub fn heuristic_score(message: &ProjectedMessage) -> i16 {
        heuristics::heuristic_score(message)
    }

    pub fn heuristic_category(message: &ProjectedMessage) -> Option<String> {
        heuristics::heuristic_category(message)
    }
}

fn clean_json_response(content: &str) -> &str {
    content
        .trim()
        .strip_prefix("```json")
        .and_then(|value| value.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(content.trim())
}
