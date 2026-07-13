use crate::ai::hub::{AiHub, AiModelRoute, LocalAiInspection, SharedAiHub};
use crate::domains::communications::messages::port::MessageProjectionPort;
use crate::domains::communications::messages::{ProjectedMessage, WorkflowState};
use crate::workflows::email_intelligence::errors::EmailIntelligenceError;
use crate::workflows::email_intelligence::heuristics;
use crate::workflows::email_intelligence::models::{EmailAnalysis, EmailSummaryContract};
use crate::workflows::email_intelligence::prompt::{
    EMAIL_INTELLIGENCE_PROMPT_VERSION, build_email_analysis_prompt,
};

#[derive(Clone)]
pub struct EmailIntelligenceService {
    hub: Option<SharedAiHub>,
}

impl EmailIntelligenceService {
    pub fn new(hub: Option<SharedAiHub>) -> Self {
        Self { hub }
    }

    pub async fn analyze_message(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Option<EmailAnalysis>, EmailIntelligenceError> {
        let inspection = AiHub::inspect_text(&message.body_text);
        self.analyze_message_with_context(message, "ru", &inspection)
            .await
    }

    pub async fn analyze_message_with_context(
        &self,
        message: &ProjectedMessage,
        target_language: &str,
        inspection: &LocalAiInspection,
    ) -> Result<Option<EmailAnalysis>, EmailIntelligenceError> {
        let Some(ref hub) = self.hub else {
            return Ok(None);
        };
        let prompt = build_email_analysis_prompt(message, target_language, inspection);
        let result = hub
            .chat_json(AiModelRoute::MailIntelligence, &prompt)
            .await?;
        let mut analysis = parse_email_analysis_response(&result.content)
            .map_err(|error| EmailIntelligenceError::ParseError(error.to_string()))?;

        analysis.model = result.model;
        analysis.prompt_version = EMAIL_INTELLIGENCE_PROMPT_VERSION.to_owned();
        if analysis.is_spam || analysis.is_phishing || is_spam_category(&analysis.category) {
            clear_candidate_arrays(&mut analysis);
        } else {
            attach_candidate_source_message_id(&mut analysis, &message.message_id);
        }

        Ok(Some(analysis))
    }

    pub async fn analyze_and_persist(
        &self,
        store: &MessageProjectionPort,
        message: &ProjectedMessage,
    ) -> Result<bool, EmailIntelligenceError> {
        let Some(analysis) = self.analyze_message(message).await? else {
            return Ok(false);
        };

        let workflow_hint = if analysis.importance_score >= 80 {
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
        let summary_contract = Self::summary_contract_for_analysis(&analysis, message);
        let mut metadata = message.message_metadata.clone();
        metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract)
            .map_err(|error| EmailIntelligenceError::ParseError(error.to_string()))?;
        metadata["mail_ai_pipeline"] = serde_json::json!({
            "model": analysis.model,
            "prompt_version": analysis.prompt_version,
            "category": analysis.category,
            "language": analysis.language,
            "is_spam": analysis.is_spam,
            "is_phishing": analysis.is_phishing,
            "spam_workflow_policy": "manual_decision_required",
        });
        store
            .set_message_metadata(&message.message_id, &metadata)
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

    pub fn heuristic_structured_summary(message: &ProjectedMessage) -> EmailSummaryContract {
        heuristics::structured_summary(message)
    }

    pub fn summary_contract_for_analysis(
        analysis: &EmailAnalysis,
        message: &ProjectedMessage,
    ) -> EmailSummaryContract {
        analysis_summary_contract(analysis, message)
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

fn parse_email_analysis_response(content: &str) -> Result<EmailAnalysis, serde_json::Error> {
    let cleaned = clean_json_response(content);
    let direct_error = match serde_json::from_str(cleaned) {
        Ok(analysis) => return Ok(analysis),
        Err(error) => error,
    };

    // Local models may prepend reasoning or a Markdown explanation despite the
    // output contract. Accept only an embedded object that fully satisfies the
    // typed EmailAnalysis schema; surrounding text is never persisted.
    for (offset, _) in cleaned.match_indices('{') {
        let mut values =
            serde_json::Deserializer::from_str(&cleaned[offset..]).into_iter::<serde_json::Value>();
        let Some(Ok(value)) = values.next() else {
            continue;
        };
        if let Ok(analysis) = serde_json::from_value(value) {
            return Ok(analysis);
        }
    }

    Err(direct_error)
}

fn analysis_summary_contract(
    analysis: &EmailAnalysis,
    message: &ProjectedMessage,
) -> EmailSummaryContract {
    let fallback = EmailIntelligenceService::heuristic_structured_summary(message);
    let allow_candidate_fallback =
        !(analysis.is_spam || analysis.is_phishing || is_spam_category(&analysis.category));
    EmailSummaryContract {
        key_points: non_empty_or(analysis.key_points.clone(), fallback.key_points),
        action_items: non_empty_or(analysis.action_items.clone(), fallback.action_items),
        risks: non_empty_or(analysis.risks.clone(), fallback.risks),
        deadlines: non_empty_or(analysis.deadlines.clone(), fallback.deadlines),
        event_candidates: non_empty_candidates_or(
            analysis.event_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.event_candidates),
        ),
        persona_candidates: non_empty_candidates_or(
            analysis.persona_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.persona_candidates),
        ),
        organization_candidates: non_empty_candidates_or(
            analysis.organization_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.organization_candidates),
        ),
        document_candidates: non_empty_candidates_or(
            analysis.document_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.document_candidates),
        ),
        agreement_candidates: non_empty_candidates_or(
            analysis.agreement_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.agreement_candidates),
        ),
        task_candidates: non_empty_candidates_or(
            analysis.task_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.task_candidates),
        ),
        decision_candidates: non_empty_candidates_or(
            analysis.decision_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.decision_candidates),
        ),
        obligation_candidates: non_empty_candidates_or(
            analysis.obligation_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.obligation_candidates),
        ),
        relationship_candidates: non_empty_candidates_or(
            analysis.relationship_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.relationship_candidates),
        ),
        fact_candidates: non_empty_candidates_or(
            analysis.fact_candidates.clone(),
            candidate_fallback(allow_candidate_fallback, fallback.fact_candidates),
        ),
    }
}

fn non_empty_or(values: Vec<String>, fallback: Vec<String>) -> Vec<String> {
    if values.is_empty() { fallback } else { values }
}

fn non_empty_candidates_or<T>(values: Vec<T>, fallback: Vec<T>) -> Vec<T> {
    if values.is_empty() { fallback } else { values }
}

fn candidate_fallback<T>(allow: bool, fallback: Vec<T>) -> Vec<T> {
    if allow { fallback } else { Vec::new() }
}

fn is_spam_category(category: &str) -> bool {
    matches!(
        category.trim().to_ascii_lowercase().as_str(),
        "spam" | "scam" | "phishing"
    )
}

fn clear_candidate_arrays(analysis: &mut EmailAnalysis) {
    analysis.event_candidates.clear();
    analysis.persona_candidates.clear();
    analysis.organization_candidates.clear();
    analysis.document_candidates.clear();
    analysis.agreement_candidates.clear();
    analysis.task_candidates.clear();
    analysis.decision_candidates.clear();
    analysis.obligation_candidates.clear();
    analysis.relationship_candidates.clear();
    analysis.fact_candidates.clear();
}

fn attach_candidate_source_message_id(analysis: &mut EmailAnalysis, message_id: &str) {
    for candidate in analysis
        .event_candidates
        .iter_mut()
        .chain(analysis.persona_candidates.iter_mut())
        .chain(analysis.organization_candidates.iter_mut())
        .chain(analysis.document_candidates.iter_mut())
        .chain(analysis.agreement_candidates.iter_mut())
        .chain(analysis.task_candidates.iter_mut())
        .chain(analysis.decision_candidates.iter_mut())
        .chain(analysis.obligation_candidates.iter_mut())
        .chain(analysis.relationship_candidates.iter_mut())
        .chain(analysis.fact_candidates.iter_mut())
    {
        if candidate.source_message_id.is_none() {
            candidate.source_message_id = Some(message_id.to_owned());
        }
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    #[test]
    fn parses_analysis_after_local_model_reasoning() {
        let analysis = parse_email_analysis_response(
            "<think>Reasoning that must not become message data.</think>\n{\n\
                \"category\": \"work\",\n\
                \"summary\": \"Follow up.\",\n\
                \"key_points\": [], \"action_items\": [], \"risks\": [], \"deadlines\": [],\n\
                \"event_candidates\": [], \"persona_candidates\": [], \"organization_candidates\": [],\n\
                \"document_candidates\": [], \"agreement_candidates\": [], \"task_candidates\": [],\n\
                \"decision_candidates\": [], \"obligation_candidates\": [], \"relationship_candidates\": [],\n\
                \"fact_candidates\": [], \"importance_score\": 20, \"is_spam\": false,\n\
                \"is_phishing\": false, \"suggested_action\": null, \"extracted_deadline\": null,\n\
                \"language\": \"en\"\n\
            }\nAdditional model commentary",
        )
        .expect("embedded typed JSON should be parsed");

        assert_eq!(analysis.category, "work");
        assert_eq!(analysis.summary, "Follow up.");
    }

    #[test]
    fn parses_fenced_analysis_response() {
        let analysis = parse_email_analysis_response(
            "```json\n{\n\
                \"category\": \"notification\",\n\
                \"summary\": \"Notice.\",\n\
                \"key_points\": [], \"action_items\": [], \"risks\": [], \"deadlines\": [],\n\
                \"event_candidates\": [], \"persona_candidates\": [], \"organization_candidates\": [],\n\
                \"document_candidates\": [], \"agreement_candidates\": [], \"task_candidates\": [],\n\
                \"decision_candidates\": [], \"obligation_candidates\": [], \"relationship_candidates\": [],\n\
                \"fact_candidates\": [], \"importance_score\": 5, \"is_spam\": false,\n\
                \"is_phishing\": false, \"suggested_action\": null, \"extracted_deadline\": null,\n\
                \"language\": \"en\"\n\
            }\n```",
        )
        .expect("fenced JSON should be parsed");

        assert_eq!(analysis.category, "notification");
    }

    #[test]
    fn spam_category_clears_candidates() {
        let mut analysis = EmailAnalysis {
            category: "spam".to_owned(),
            summary: "spam".to_owned(),
            key_points: Vec::new(),
            action_items: Vec::new(),
            risks: Vec::new(),
            deadlines: Vec::new(),
            event_candidates: Vec::new(),
            persona_candidates: vec![
                crate::workflows::email_intelligence::models::EmailKnowledgeCandidate::new(
                    "Bad sender",
                    "evidence",
                ),
            ],
            organization_candidates: Vec::new(),
            document_candidates: Vec::new(),
            agreement_candidates: Vec::new(),
            task_candidates: Vec::new(),
            decision_candidates: Vec::new(),
            obligation_candidates: Vec::new(),
            relationship_candidates: Vec::new(),
            fact_candidates: Vec::new(),
            importance_score: 0,
            is_spam: true,
            is_phishing: false,
            suggested_action: None,
            extracted_deadline: None,
            language: Some("en".to_owned()),
            model: "test".to_owned(),
            prompt_version: "test".to_owned(),
        };

        clear_candidate_arrays(&mut analysis);

        assert!(analysis.persona_candidates.is_empty());
    }
}
