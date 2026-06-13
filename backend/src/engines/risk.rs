use thiserror::Error;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskAttentionStatus {
    Healthy,
    NeedsAttention,
    AtRisk,
}

impl RiskAttentionStatus {
    pub fn as_persona_health_status(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::NeedsAttention => "needs_attention",
            Self::AtRisk => "at_risk",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskSeverity {
    pub fn parse(value: &str) -> Result<Self, RiskEngineError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            other => Err(RiskEngineError::InvalidSeverity(other.to_owned())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    fn suggested_handling_state(self) -> &'static str {
        match self {
            Self::Critical | Self::High => "review_now",
            Self::Medium | Self::Low => "monitor",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RiskObservationDraft {
    pub affected_entity_kind: String,
    pub affected_entity_id: String,
    pub risk_type: String,
    pub evidence: String,
    pub source: String,
    pub confidence: f64,
    pub severity: RiskSeverity,
    pub suggested_handling_state: String,
    pub review_state: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RiskSignal {
    pub severity: RiskSeverity,
    pub resolved: bool,
}

impl RiskSignal {
    pub fn unresolved(severity: RiskSeverity) -> Self {
        Self {
            severity,
            resolved: false,
        }
    }

    pub fn resolved(severity: RiskSeverity) -> Self {
        Self {
            severity,
            resolved: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum RiskEngineError {
    #[error("invalid risk severity `{0}`")]
    InvalidSeverity(String),
    #[error("risk observation {0} must not be empty")]
    EmptyField(&'static str),
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RiskEngineError> {
    if value.trim().is_empty() {
        return Err(RiskEngineError::EmptyField(field));
    }
    Ok(())
}
