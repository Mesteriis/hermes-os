use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use crate::ai::hub::{AiHub, LocalAiInspection, SharedAiHub};
use crate::domains::communications::ai_state::{
    CommunicationAiState, CommunicationAiStatePort, CommunicationAiStateTransitionRequest,
};
use crate::domains::communications::messages::{
    CommunicationMessageProjectionPort, ProjectedMessage, WorkflowState,
};
use crate::domains::communications::spam_reputation::{
    SenderReputationClassification, SenderReputationDecision, SenderReputationPort,
};
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxPort, ReviewItemKind,
};
use crate::domains::signal_hub::dispatch_ai_helper_signal_best_effort;
use crate::workflows::email_intelligence::errors::EmailIntelligenceError;
use crate::workflows::email_intelligence::models::{EmailAnalysis, EmailKnowledgeCandidate};
use crate::workflows::email_intelligence::service::EmailIntelligenceService;

const DEFAULT_BATCH_LIMIT: i64 = 10;

#[derive(Clone)]
pub struct MailAiPipelineService {
    pool: PgPool,
    hub: Option<SharedAiHub>,
    target_language: String,
}

impl MailAiPipelineService {
    pub fn new(pool: PgPool, hub: Option<SharedAiHub>, target_language: impl Into<String>) -> Self {
        Self {
            pool,
            hub,
            target_language: normalize_target_language(target_language.into()),
        }
    }

    pub async fn process_next_batch(
        &self,
        limit: i64,
    ) -> Result<MailAiPipelineReport, EmailIntelligenceError> {
        let state_store = CommunicationAiStatePort::new(self.pool.clone());
        let message_ids = state_store
            .pending_mail_message_ids(limit.clamp(1, DEFAULT_BATCH_LIMIT))
            .await?;
        let mut report = MailAiPipelineReport {
            claimed: message_ids.len(),
            ..MailAiPipelineReport::default()
        };
        let message_store = CommunicationMessageProjectionPort::new(self.pool.clone());

        for message_id in message_ids {
            let Some(message) = message_store.message(&message_id).await? else {
                continue;
            };
            mark_ai_state(
                &state_store,
                &message.message_id,
                CommunicationAiState::Processing,
                None,
                None,
            )
            .await?;

            match self.process_message(&message).await {
                Ok(outcome) => {
                    report.processed += 1;
                    if outcome.suppressed {
                        report.suppressed += 1;
                    }
                    report.review_candidates += outcome.review_candidates;
                    mark_ai_state(
                        &state_store,
                        &message.message_id,
                        CommunicationAiState::Processed,
                        outcome.reason.as_deref(),
                        None,
                    )
                    .await?;
                }
                Err(error) => {
                    report.failed += 1;
                    let reason = failure_reason(&error);
                    mark_ai_state(
                        &state_store,
                        &message.message_id,
                        CommunicationAiState::Failed,
                        Some(reason),
                        Some(&error.to_string()),
                    )
                    .await?;
                    dispatch_mail_ai_signal(
                        self.pool.clone(),
                        &message,
                        None,
                        None,
                        Some(reason),
                        None,
                        0,
                    )
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
        let message_store = CommunicationMessageProjectionPort::new(self.pool.clone());
        let reputation_store = SenderReputationPort::new(self.pool.clone());
        let reputation = reputation_store.evaluate_message(message).await?;
        if reputation.suppressed {
            return self
                .process_reputation_suppressed_message(message, &message_store, &reputation)
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
            reason: Some("mail_ai_processed".to_owned()),
        })
    }

    async fn process_reputation_suppressed_message(
        &self,
        message: &ProjectedMessage,
        message_store: &CommunicationMessageProjectionPort,
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
        let _ = message_store
            .transition_workflow_state(&message.message_id, WorkflowState::Spam)
            .await;
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
            reason: Some("sender_reputation_zero".to_owned()),
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MailAiPipelineReport {
    pub claimed: usize,
    pub processed: usize,
    pub suppressed: usize,
    pub failed: usize,
    pub review_candidates: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MailAiPipelineMessageOutcome {
    suppressed: bool,
    review_candidates: usize,
    reason: Option<String>,
}

async fn persist_analysis(
    message_store: &CommunicationMessageProjectionPort,
    message: &ProjectedMessage,
    analysis: &EmailAnalysis,
    inspection: &LocalAiInspection,
    reputation: &SenderReputationDecision,
) -> Result<(), EmailIntelligenceError> {
    let workflow_hint = if analysis.is_spam
        || analysis.is_phishing
        || reputation_classification(analysis) == SenderReputationClassification::Spam
    {
        Some(WorkflowState::Spam)
    } else if analysis.importance_score >= 80 {
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
            let item = NewReviewItem::new(
                kind,
                candidate.title.clone(),
                summary,
                candidate.confidence.unwrap_or(default_confidence),
            )
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
            ReviewItemKind::NewPerson,
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

fn is_spam_category(category: &str) -> bool {
    matches!(
        category.trim().to_ascii_lowercase().as_str(),
        "spam" | "scam" | "phishing"
    )
}

fn failure_reason(error: &EmailIntelligenceError) -> &'static str {
    match error {
        EmailIntelligenceError::RouteNotConfigured(_) => "route_not_configured",
        EmailIntelligenceError::LocalModelRequired { .. } => "local_model_required",
        EmailIntelligenceError::Hub(_) => "model_unavailable",
        EmailIntelligenceError::ParseError(_) => "parse_error",
        _ => "mail_ai_pipeline_failed",
    }
}

fn normalize_target_language(value: String) -> String {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        "ru".to_owned()
    } else {
        value
    }
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
}
