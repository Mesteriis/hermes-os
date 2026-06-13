use thiserror::Error;

pub struct TrustEngine;

impl TrustEngine {
    pub fn persona_compatibility_score_signal(score: i16) -> TrustRelationshipSignal {
        TrustRelationshipSignal {
            kind: TrustSignalKind::PersonaCompatibilityScore,
            relationship_type: "trusts",
            trust_score: normalize_compatibility_score(score),
            strength_score: 0.5,
            confidence: 1.0,
            explanation: "compatibility persons.trust_score signal",
        }
    }

    pub fn source_reliability_signal(
        affected_source: &str,
        evidence: &str,
        confidence: f64,
    ) -> Result<TrustSourceReliabilitySignal, TrustEngineError> {
        validate_non_empty("affected source", affected_source)?;
        validate_non_empty("evidence", evidence)?;
        validate_confidence(confidence)?;

        Ok(TrustSourceReliabilitySignal {
            kind: TrustSignalKind::SourceReliability,
            affected_source: affected_source.trim().to_owned(),
            evidence: evidence.trim().to_owned(),
            confidence,
            direction: TrustImpactDirection::from_confidence(confidence),
            explanation: "source reliability signal for review",
        })
    }
}

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

    fn from_confidence(confidence: f64) -> Self {
        if confidence >= 0.5 {
            Self::Positive
        } else {
            Self::Negative
        }
    }
}

fn normalize_compatibility_score(score: i16) -> f64 {
    (f64::from(score.clamp(0, 100)) / 100.0 * 10000.0).round() / 10000.0
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), TrustEngineError> {
    if value.trim().is_empty() {
        return Err(TrustEngineError::EmptyField(field));
    }
    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), TrustEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(TrustEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq)]
pub enum TrustEngineError {
    #[error("trust signal {0} must not be empty")]
    EmptyField(&'static str),
    #[error("trust signal confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),
}
