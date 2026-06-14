use crate::engines::trust::errors::TrustEngineError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustSignalKind {
    PersonaCompatibilityScore,
    SourceReliability,
}

impl TrustSignalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersonaCompatibilityScore => "persona_compatibility_score",
            Self::SourceReliability => "source_reliability",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrustRelationshipSignal {
    pub kind: TrustSignalKind,
    pub relationship_type: &'static str,
    pub trust_score: f64,
    pub strength_score: f64,
    pub confidence: f64,
    pub explanation: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrustSourceReliabilitySignal {
    pub kind: TrustSignalKind,
    pub affected_source: String,
    pub evidence: String,
    pub confidence: f64,
    pub direction: TrustImpactDirection,
    pub explanation: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustImpactDirection {
    Positive,
    Negative,
}

impl TrustImpactDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }

    pub fn from_confidence(confidence: f64) -> Self {
        if confidence >= 0.5 {
            Self::Positive
        } else {
            Self::Negative
        }
    }
}

pub fn normalize_compatibility_score(score: i16) -> f64 {
    (f64::from(score.clamp(0, 100)) / 100.0 * 10000.0).round() / 10000.0
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), TrustEngineError> {
    if value.trim().is_empty() {
        return Err(TrustEngineError::EmptyField(field));
    }
    Ok(())
}

pub fn validate_confidence(confidence: f64) -> Result<(), TrustEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(TrustEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}
