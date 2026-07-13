use crate::domains::communications::messages::port::MessageProjectionPort;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use std::sync::Arc;

use crate::ai::hub::{AiHub, LocalAiInspection, SensitivityLevel, SharedAiHub};
use crate::domains::communications::ai_state::{
    CommunicationAiState, CommunicationAiStatePort, CommunicationAiStateTransitionRequest,
};
use crate::domains::communications::messages::{ProjectedMessage, WorkflowState};
use crate::domains::communications::sensitive_forwarding::SensitiveForwardingCommandPort;
use crate::domains::communications::spam_reputation::{
    SenderReputationClassification, SenderReputationDecision, SenderReputationPort,
};
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxPort, ReviewItemKind,
};
use crate::domains::signal_hub::ai::dispatch_ai_helper_signal_best_effort;
use crate::workflows::email_intelligence::errors::EmailIntelligenceError;
use crate::workflows::email_intelligence::models::{EmailAnalysis, EmailKnowledgeCandidate};
use crate::workflows::email_intelligence::service::EmailIntelligenceService;

// A single local model invocation can take longer than a provider sync tick.
// Claim one message until this worker has bounded concurrent processing.
const MAX_IN_FLIGHT_MAIL_AI_CLAIMS: i64 = 1;

#[derive(Clone)]
pub struct MailAiPipelineService {
    pool: PgPool,
    hub: Option<SharedAiHub>,
    target_language: String,
    external_body_egress_required: bool,
    sensitive_forwarding: Arc<dyn SensitiveForwardingCommandPort>,
}

impl MailAiPipelineService {
    pub fn new(
        pool: PgPool,
        hub: Option<SharedAiHub>,
        target_language: impl Into<String>,
        sensitive_forwarding: Arc<dyn SensitiveForwardingCommandPort>,
    ) -> Self {
        Self {
            pool,
            hub,
            target_language: normalize_target_language(target_language.into()),
            external_body_egress_required: false,
            sensitive_forwarding,
        }
    }

    pub fn requiring_external_body_egress(mut self, required: bool) -> Self {
        self.external_body_egress_required = required;
        self
    }

    pub async fn process_next_batch(
        &self,
        limit: i64,
    ) -> Result<MailAiPipelineReport, EmailIntelligenceError> {
        let state_store = CommunicationAiStatePort::new(self.pool.clone());
        let now = chrono::Utc::now();
        let recovered = state_store.recover_expired_mail_processing(now).await?;
        let message_ids = state_store
            .claim_due_mail_messages(mail_ai_claim_limit(limit), now)
            .await?;
        let mut report = MailAiPipelineReport {
            claimed: message_ids.len(),
            recovered,
            ..MailAiPipelineReport::default()
        };
        let message_store = MessageProjectionPort::new(self.pool.clone());

        for message_id in message_ids {
            let Some(message) = message_store.message(&message_id).await? else {
                continue;
            };
            match self.process_message(&message).await {
                Ok(outcome) => {
                    complete_mail_ai_outcome(&mut report, &state_store, &message, outcome).await?;
                }
                Err(EmailIntelligenceError::ParseError(_)) => {
                    match self
                        .process_unstructured_model_response(&message, &message_store)
                        .await
                    {
                        Ok(outcome) => {
                            complete_mail_ai_outcome(&mut report, &state_store, &message, outcome)
                                .await?;
                        }
                        Err(error) => {
                            record_mail_ai_failure(
                                &mut report,
                                &state_store,
                                &self.pool,
                                &message,
                                &error,
                            )
                            .await;
                        }
                    }
                }
                Err(error) => {
                    record_mail_ai_failure(&mut report, &state_store, &self.pool, &message, &error)
                        .await;
                }
            }
        }

        Ok(report)
    }

    async fn process_message(
        &self,
        message: &ProjectedMessage,
    ) -> Result<MailAiPipelineMessageOutcome, EmailIntelligenceError> {
        let message_store = MessageProjectionPort::new(self.pool.clone());
        let reputation_store = SenderReputationPort::new(self.pool.clone());
        let reputation = reputation_store.evaluate_message(message).await?;
        if reputation.suppressed {
            return self
                .process_reputation_suppressed_message(message, &message_store, &reputation)
                .await;
        }

        if self.external_body_egress_required
            && !self.external_body_egress_allowed(&message.account_id).await
        {
            return self
                .process_egress_suppressed_message(message, &message_store)
                .await;
        }

        let inspection = AiHub::inspect_text(&message.body_text);
        let Some(hub) = self.hub.clone() else {
            return Err(EmailIntelligenceError::RouteNotConfigured(
                "mail_intelligence".to_owned(),
            ));
        };
        let service = EmailIntelligenceService::new(Some(hub));
        let analysis = service
            .analyze_message_with_context(message, &self.target_language, &inspection)
            .await?
            .ok_or_else(|| {
                EmailIntelligenceError::RouteNotConfigured("mail_intelligence".to_owned())
            })?;
        persist_analysis(&message_store, message, &analysis, &inspection, &reputation).await?;

        let classification = reputation_classification(&analysis);
        reputation_store
            .record_analysis(message, classification, analysis.category.as_str())
            .await?;

        let review_candidates = if classification == SenderReputationClassification::Spam {
            0
        } else {
            emit_review_candidates(&self.pool, message, &analysis).await?
        };
        let sensitive_forwarding_queued = if classification == SenderReputationClassification::Spam
            || inspection.sensitivity == SensitivityLevel::Public
        {
            0
        } else {
            match self
                .sensitive_forwarding
                .enqueue_for_message(
                    &message.account_id,
                    &message.message_id,
                    sensitivity_label(inspection.sensitivity),
                    chrono::Utc::now(),
                )
                .await
            {
                Ok(report) => report.queued,
                Err(error) => {
                    tracing::warn!(
                        message_id = %message.message_id,
                        error = %error,
                        "sensitive forwarding policy evaluation failed"
                    );
                    0
                }
            }
        };
        dispatch_mail_ai_signal(
            self.pool.clone(),
            message,
            Some(&analysis),
            Some(&inspection),
            None,
            Some(&reputation),
            review_candidates,
        )
        .await;

        Ok(MailAiPipelineMessageOutcome {
            suppressed: false,
            review_candidates,
            sensitive_forwarding_queued,
            reason: Some("mail_ai_processed".to_owned()),
            final_ai_state: CommunicationAiState::Processed,
        })
    }

    async fn external_body_egress_allowed(&self, account_id: &str) -> bool {
        match self
            .sensitive_forwarding
            .content_egress_permissions(account_id)
            .await
        {
            Ok(permissions) => permissions.body,
            Err(error) => {
                tracing::warn!(
                    account_id,
                    error = %error,
                    "mail AI pipeline could not load content-egress policy; denying external body egress"
                );
                false
            }
        }
    }

    async fn process_egress_suppressed_message(
        &self,
        message: &ProjectedMessage,
        message_store: &MessageProjectionPort,
    ) -> Result<MailAiPipelineMessageOutcome, EmailIntelligenceError> {
        let mut metadata = message.message_metadata.clone();
        metadata["mail_ai_pipeline"] = json!({
            "status": "review_required",
            "reason": "body_egress_denied",
            "llm_used": false,
            "body_included": false,
        });
        message_store
            .set_message_metadata(&message.message_id, &metadata)
            .await?;
        dispatch_mail_ai_signal(
            self.pool.clone(),
            message,
            None,
            None,
            Some("body_egress_denied"),
            None,
            0,
        )
        .await;

        Ok(MailAiPipelineMessageOutcome {
            suppressed: true,
            review_candidates: 0,
            sensitive_forwarding_queued: 0,
            reason: Some("body_egress_denied".to_owned()),
            final_ai_state: CommunicationAiState::ReviewRequired,
        })
    }

    async fn process_reputation_suppressed_message(
        &self,
        message: &ProjectedMessage,
        message_store: &MessageProjectionPort,
        reputation: &SenderReputationDecision,
    ) -> Result<MailAiPipelineMessageOutcome, EmailIntelligenceError> {
        SenderReputationPort::new(self.pool.clone())
            .record_suppressed_message(message, "sender_reputation_zero")
            .await?;
        message_store
            .set_ai_analysis(
                &message.message_id,
                Some("spam"),
                Some("Suppressed by sender reputation; no LLM was used."),
                Some(0),
            )
            .await?;
        let mut metadata = message.message_metadata.clone();
        metadata["mail_ai_pipeline"] = json!({
            "status": "suppressed",
            "reason": "sender_reputation_zero",
            "reputation": reputation,
            "llm_used": false,
        });
        metadata["ai_summary_contract"] = json!({
            "key_points": [],
            "action_items": [],
            "risks": ["Sender reputation reached zero; message suppressed before LLM analysis."],
            "deadlines": [],
            "event_candidates": [],
            "persona_candidates": [],
            "organization_candidates": [],
            "document_candidates": [],
            "agreement_candidates": [],
            "task_candidates": [],
            "decision_candidates": [],
            "obligation_candidates": [],
            "relationship_candidates": [],
            "fact_candidates": []
        });
        message_store
            .set_message_metadata(&message.message_id, &metadata)
            .await?;
        dispatch_mail_ai_signal(
            self.pool.clone(),
            message,
            None,
            None,
            Some("sender_reputation_zero"),
            Some(reputation),
            0,
        )
        .await;

        Ok(MailAiPipelineMessageOutcome {
            suppressed: true,
            review_candidates: 0,
            sensitive_forwarding_queued: 0,
            reason: Some("sender_reputation_zero".to_owned()),
            final_ai_state: CommunicationAiState::Processed,
        })
    }

    async fn process_unstructured_model_response(
        &self,
        message: &ProjectedMessage,
        message_store: &MessageProjectionPort,
    ) -> Result<MailAiPipelineMessageOutcome, EmailIntelligenceError> {
        let category = EmailIntelligenceService::heuristic_category(message)
            .unwrap_or_else(|| "unclassified".to_owned());
        let importance_score = EmailIntelligenceService::heuristic_score(message);
        let summary_contract = EmailIntelligenceService::heuristic_structured_summary(message);
        let mut metadata = message.message_metadata.clone();
        metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract)
            .map_err(|error| EmailIntelligenceError::ParseError(error.to_string()))?;
        metadata["mail_ai_pipeline"] = json!({
            "status": "review_required",
            "reason": "model_response_invalid",
            "llm_used": true,
            "raw_model_output_persisted": false,
            "summary_source": "deterministic_heuristic",
        });
        message_store
            .set_ai_analysis(
                &message.message_id,
                Some(&category),
                Some("AI returned an unstructured response; review the original message."),
                Some(importance_score),
            )
            .await?;
        message_store
            .set_message_metadata(&message.message_id, &metadata)
            .await?;
        dispatch_mail_ai_signal(
            self.pool.clone(),
            message,
            None,
            None,
            Some("model_response_invalid"),
            None,
            0,
        )
        .await;

        Ok(MailAiPipelineMessageOutcome {
            suppressed: false,
            review_candidates: 0,
            sensitive_forwarding_queued: 0,
            reason: Some("model_response_invalid".to_owned()),
            final_ai_state: CommunicationAiState::ReviewRequired,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MailAiPipelineReport {
    pub claimed: usize,
    pub recovered: usize,
    pub processed: usize,
    pub suppressed: usize,
    pub failed: usize,
    pub retrying: usize,
    pub review_candidates: usize,
    pub sensitive_forwarding_queued: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MailAiPipelineMessageOutcome {
    suppressed: bool,
    review_candidates: usize,
    sensitive_forwarding_queued: usize,
    reason: Option<String>,
    final_ai_state: CommunicationAiState,
}

async fn complete_mail_ai_outcome(
    report: &mut MailAiPipelineReport,
    state_store: &CommunicationAiStatePort,
    message: &ProjectedMessage,
    outcome: MailAiPipelineMessageOutcome,
) -> Result<(), EmailIntelligenceError> {
    report.processed += 1;
    if outcome.suppressed {
        report.suppressed += 1;
    }
    report.review_candidates += outcome.review_candidates;
    report.sensitive_forwarding_queued += outcome.sensitive_forwarding_queued;
    let review_reason = (outcome.final_ai_state == CommunicationAiState::ReviewRequired)
        .then_some(outcome.reason.as_deref())
        .flatten();
    mark_ai_state(
        state_store,
        &message.message_id,
        outcome.final_ai_state,
        review_reason,
        None,
    )
    .await
}

async fn record_mail_ai_failure(
    report: &mut MailAiPipelineReport,
    state_store: &CommunicationAiStatePort,
    pool: &PgPool,
    message: &ProjectedMessage,
    error: &EmailIntelligenceError,
) {
    report.failed += 1;
    let reason = failure_reason(error);
    match state_store
        .record_mail_processing_failure(
            &message.message_id,
            &truncate_error(&error.to_string()),
            ai_failure_is_retryable(error),
            chrono::Utc::now(),
        )
        .await
    {
        Ok(record) => {
            if record.and_then(|record| record.next_attempt_at).is_some() {
                report.retrying += 1;
            }
        }
        Err(state_error) => {
            tracing::warn!(
                message_id = %message.message_id,
                error = %state_error,
                "mail AI pipeline could not record message failure"
            );
        }
    }
    dispatch_mail_ai_signal(pool.clone(), message, None, None, Some(reason), None, 0).await;
}

async fn persist_analysis(
    message_store: &MessageProjectionPort,
    message: &ProjectedMessage,
    analysis: &EmailAnalysis,
    inspection: &LocalAiInspection,
    reputation: &SenderReputationDecision,
) -> Result<(), EmailIntelligenceError> {
    let workflow_hint = if analysis.importance_score >= 80 {
        Some(WorkflowState::NeedsAction)
    } else {
        None
    };

    message_store
        .set_ai_analysis(
            &message.message_id,
            Some(&analysis.category),
            Some(&analysis.summary),
            Some(analysis.importance_score),
        )
        .await?;
    let summary_contract =
        EmailIntelligenceService::summary_contract_for_analysis(analysis, message);
    let mut metadata = message.message_metadata.clone();
    metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract)
        .map_err(|error| EmailIntelligenceError::ParseError(error.to_string()))?;
    metadata["mail_ai_pipeline"] = json!({
        "status": "processed",
        "model": analysis.model,
        "prompt_version": analysis.prompt_version,
        "target_language": analysis.language.as_deref().unwrap_or("unknown"),
        "local_inspection": inspection,
        "reputation": reputation,
        "candidate_policy": if reputation_classification(analysis) == SenderReputationClassification::Spam {
            "spam_blocked_candidates"
        } else {
            "emit_candidates"
        },
        "spam_workflow_policy": "manual_decision_required",
    });
    message_store
        .set_message_metadata(&message.message_id, &metadata)
        .await?;

    if let Some(state) = workflow_hint {
        let _ = message_store
            .transition_workflow_state(&message.message_id, state)
            .await;
    }

    Ok(())
}

async fn emit_review_candidates(
    pool: &PgPool,
    message: &ProjectedMessage,
    analysis: &EmailAnalysis,
) -> Result<usize, EmailIntelligenceError> {
    let review_store = ReviewInboxPort::new(pool.clone());
    let mut count = 0usize;

    for (kind, group, candidates, default_confidence) in review_candidate_groups(analysis) {
        for candidate in candidates {
            let summary = candidate_summary(candidate, group);
            let confidence = normalized_candidate_confidence(
                candidate.confidence.unwrap_or(default_confidence),
                default_confidence,
            );
            let item = NewReviewItem::new(kind, candidate.title.clone(), summary, confidence)
                .metadata(json!({
                    "mirrored_from": "mail_ai_pipeline",
                    "message_id": message.message_id,
                    "observation_id": message.observation_id,
                    "candidate_group": group,
                    "candidate_kind": candidate.kind.as_deref(),
                    "candidate_title": candidate.title,
                    "identifiers": candidate.identifiers.clone(),
                    "model": analysis.model,
                    "prompt_version": analysis.prompt_version,
                }));
            let evidence = NewReviewItemEvidence::new(message.observation_id.clone())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "mail_ai_pipeline",
                    "message_id": message.message_id,
                    "candidate_group": group,
                }));
            let _ = review_store
                .create_with_evidence(&item, &[evidence])
                .await?;
            count += 1;
        }
    }

    Ok(count)
}

fn review_candidate_groups(
    analysis: &EmailAnalysis,
) -> Vec<(
    ReviewItemKind,
    &'static str,
    &[EmailKnowledgeCandidate],
    f64,
)> {
    vec![
        (
            ReviewItemKind::NewPersona,
            "persona",
            &analysis.persona_candidates,
            0.68,
        ),
        (
            ReviewItemKind::NewOrganization,
            "organization",
            &analysis.organization_candidates,
            0.70,
        ),
        (
            ReviewItemKind::PotentialTask,
            "task",
            &analysis.task_candidates,
            0.72,
        ),
        (
            ReviewItemKind::PotentialDecision,
            "decision",
            &analysis.decision_candidates,
            0.74,
        ),
        (
            ReviewItemKind::PotentialObligation,
            "obligation",
            &analysis.obligation_candidates,
            0.74,
        ),
        (
            ReviewItemKind::PotentialRelationship,
            "relationship",
            &analysis.relationship_candidates,
            0.70,
        ),
        (
            ReviewItemKind::KnowledgeCandidate,
            "event",
            &analysis.event_candidates,
            0.68,
        ),
        (
            ReviewItemKind::KnowledgeCandidate,
            "document",
            &analysis.document_candidates,
            0.72,
        ),
        (
            ReviewItemKind::KnowledgeCandidate,
            "agreement",
            &analysis.agreement_candidates,
            0.79,
        ),
        (
            ReviewItemKind::KnowledgeCandidate,
            "fact",
            &analysis.fact_candidates,
            0.70,
        ),
    ]
}

fn candidate_summary(candidate: &EmailKnowledgeCandidate, group: &str) -> String {
    let summary = candidate.summary_text();
    if summary.is_empty() {
        format!("Source-backed {group} candidate from communication evidence")
    } else {
        summary
    }
}

fn normalized_candidate_confidence(value: f64, fallback: f64) -> f64 {
    if !value.is_finite() {
        return fallback;
    }

    // Models commonly return a percentage despite the prompt's decimal contract.
    // Preserve that signal while keeping Review's persisted score within [0.0, 1.0].
    if value > 1.0 {
        return (value / 100.0).clamp(0.0, 1.0);
    }

    value.clamp(0.0, 1.0)
}

async fn mark_ai_state(
    state_store: &CommunicationAiStatePort,
    message_id: &str,
    ai_state: CommunicationAiState,
    review_reason: Option<&str>,
    last_error: Option<&str>,
) -> Result<(), EmailIntelligenceError> {
    let _ = state_store
        .transition(
            message_id,
            CommunicationAiStateTransitionRequest {
                ai_state,
                review_reason: review_reason.map(ToOwned::to_owned),
                last_error: last_error.map(truncate_error),
            },
        )
        .await?;
    Ok(())
}

async fn dispatch_mail_ai_signal(
    pool: PgPool,
    message: &ProjectedMessage,
    analysis: Option<&EmailAnalysis>,
    inspection: Option<&LocalAiInspection>,
    failure_reason: Option<&str>,
    reputation: Option<&SenderReputationDecision>,
    review_candidates: usize,
) {
    let payload = json!({
        "message_id": message.message_id,
        "observation_id": message.observation_id,
        "status": if failure_reason.is_some() { "failed_or_suppressed" } else { "processed" },
        "failure_reason": failure_reason,
        "category": analysis.map(|analysis| analysis.category.as_str()),
        "is_spam": analysis.map(|analysis| analysis.is_spam),
        "is_phishing": analysis.map(|analysis| analysis.is_phishing),
        "model": analysis.map(|analysis| analysis.model.as_str()),
        "prompt_version": analysis.map(|analysis| analysis.prompt_version.as_str()),
        "inspection": inspection,
        "reputation": reputation,
        "review_candidates": review_candidates,
        "body_included": false,
    });
    let subject = json!({
        "kind": "communication_message",
        "message_id": message.message_id,
        "observation_id": message.observation_id,
    });
    let provenance = json!({
        "source": "mail_ai_pipeline",
        "source_message_id": message.message_id,
        "privacy": "body_redacted",
    });
    let _ = dispatch_ai_helper_signal_best_effort(
        pool,
        "mail_intelligence",
        &message.message_id,
        subject,
        payload,
        provenance,
        Some(&message.observation_id),
    )
    .await;
}

fn reputation_classification(analysis: &EmailAnalysis) -> SenderReputationClassification {
    if analysis.is_spam || analysis.is_phishing || is_spam_category(&analysis.category) {
        SenderReputationClassification::Spam
    } else {
        SenderReputationClassification::NonSpam
    }
}

fn sensitivity_label(value: SensitivityLevel) -> &'static str {
    match value {
        SensitivityLevel::Public => "public",
        SensitivityLevel::Low => "low",
        SensitivityLevel::Medium => "medium",
        SensitivityLevel::High => "high",
        SensitivityLevel::Critical => "critical",
    }
}

fn is_spam_category(category: &str) -> bool {
    matches!(
        category.trim().to_ascii_lowercase().as_str(),
        "spam" | "scam" | "phishing"
    )
}

fn failure_reason(error: &EmailIntelligenceError) -> &'static str {
    match error {
        EmailIntelligenceError::RouteNotConfigured(_) => "route_not_configured",
        EmailIntelligenceError::Hub(_) => "model_unavailable",
        EmailIntelligenceError::ParseError(_) => "parse_error",
        _ => "mail_ai_pipeline_failed",
    }
}

fn ai_failure_is_retryable(error: &EmailIntelligenceError) -> bool {
    matches!(
        error,
        EmailIntelligenceError::Hub(_) | EmailIntelligenceError::Sqlx(_)
    )
}

fn normalize_target_language(value: String) -> String {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        "ru".to_owned()
    } else {
        value
    }
}

fn mail_ai_claim_limit(requested: i64) -> i64 {
    requested.clamp(1, MAX_IN_FLIGHT_MAIL_AI_CLAIMS)
}

fn truncate_error(error: &str) -> String {
    const LIMIT: usize = 240;
    let mut value = error.trim().replace(['\n', '\r'], " ");
    if value.chars().count() > LIMIT {
        value = value.chars().take(LIMIT).collect();
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analysis(category: &str, is_spam: bool, is_phishing: bool) -> EmailAnalysis {
        EmailAnalysis {
            category: category.to_owned(),
            summary: "summary".to_owned(),
            key_points: Vec::new(),
            action_items: Vec::new(),
            risks: Vec::new(),
            deadlines: Vec::new(),
            event_candidates: Vec::new(),
            persona_candidates: Vec::new(),
            organization_candidates: Vec::new(),
            document_candidates: Vec::new(),
            agreement_candidates: Vec::new(),
            task_candidates: Vec::new(),
            decision_candidates: Vec::new(),
            obligation_candidates: Vec::new(),
            relationship_candidates: Vec::new(),
            fact_candidates: Vec::new(),
            importance_score: 10,
            is_spam,
            is_phishing,
            suggested_action: None,
            extracted_deadline: None,
            language: Some("en".to_owned()),
            model: "test".to_owned(),
            prompt_version: "test".to_owned(),
        }
    }

    #[test]
    fn spam_category_updates_reputation_as_spam() {
        assert_eq!(
            reputation_classification(&analysis("spam", false, false)),
            SenderReputationClassification::Spam
        );
    }

    #[test]
    fn newsletter_without_spam_updates_reputation_as_non_spam() {
        assert_eq!(
            reputation_classification(&analysis("newsletter", false, false)),
            SenderReputationClassification::NonSpam
        );
    }

    #[test]
    fn intelligence_spam_classification_never_selects_spam_workflow() {
        let analysis = analysis("spam", true, true);
        let workflow_hint = (analysis.importance_score >= 80).then_some(WorkflowState::NeedsAction);

        assert_eq!(workflow_hint, None);
    }

    #[test]
    fn sensitivity_labels_preserve_policy_ordering() {
        assert_eq!(sensitivity_label(SensitivityLevel::Low), "low");
        assert_eq!(sensitivity_label(SensitivityLevel::Medium), "medium");
        assert_eq!(sensitivity_label(SensitivityLevel::High), "high");
        assert_eq!(sensitivity_label(SensitivityLevel::Critical), "critical");
    }

    #[test]
    fn candidate_confidence_accepts_percentages_from_ai_output() {
        assert_eq!(normalized_candidate_confidence(100.0, 0.68), 1.0);
        assert_eq!(normalized_candidate_confidence(75.0, 0.68), 0.75);
    }

    #[test]
    fn candidate_confidence_keeps_decimal_contract_and_safe_fallback() {
        assert_eq!(normalized_candidate_confidence(0.72, 0.68), 0.72);
        assert_eq!(normalized_candidate_confidence(-0.5, 0.68), 0.0);
        assert_eq!(normalized_candidate_confidence(f64::NAN, 0.68), 0.68);
    }

    #[test]
    fn mail_ai_worker_never_claims_more_than_one_slow_model_invocation() {
        assert_eq!(mail_ai_claim_limit(10), 1);
        assert_eq!(mail_ai_claim_limit(1), 1);
        assert_eq!(mail_ai_claim_limit(0), 1);
    }
}
