use crate::engines::risk::errors::RiskEngineError;
use crate::engines::risk::models::{
    RiskAttentionStatus, RiskObservationDraft, RiskSeverity, RiskSignal, validate_non_empty,
};

pub struct RiskEngine;

impl RiskEngine {
    pub fn derive_attention_status(risks: &[RiskSignal]) -> RiskAttentionStatus {
        let mut has_attention_risk = false;

        for risk in risks.iter().filter(|risk| !risk.resolved) {
            match risk.severity {
                RiskSeverity::Critical | RiskSeverity::High => return RiskAttentionStatus::AtRisk,
                RiskSeverity::Medium | RiskSeverity::Low => has_attention_risk = true,
            }
        }

        if has_attention_risk {
            RiskAttentionStatus::NeedsAttention
        } else {
            RiskAttentionStatus::Healthy
        }
    }

    pub fn persona_observation(
        person_id: &str,
        risk_type: &str,
        evidence: &str,
        severity: &str,
        source: &str,
    ) -> Result<RiskObservationDraft, RiskEngineError> {
        validate_non_empty("affected entity", person_id)?;
        validate_non_empty("risk type", risk_type)?;
        validate_non_empty("evidence", evidence)?;
        validate_non_empty("source", source)?;

        let severity = RiskSeverity::parse(severity)?;

        Ok(RiskObservationDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: person_id.trim().to_owned(),
            risk_type: risk_type.trim().to_owned(),
            evidence: evidence.trim().to_owned(),
            source: source.trim().to_owned(),
            confidence: 0.5,
            severity,
            suggested_handling_state: severity.suggested_handling_state().to_owned(),
            review_state: "suggested".to_owned(),
        })
    }
}
