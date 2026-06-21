use crate::engines::risk::errors::RiskEngineError;

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

    pub fn suggested_handling_state(self) -> &'static str {
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

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RiskEngineError> {
    if value.trim().is_empty() {
        return Err(RiskEngineError::EmptyField(field));
    }
    Ok(())
}
