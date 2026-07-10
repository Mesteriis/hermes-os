use crate::engines::trust::errors::TrustEngineError;
use crate::engines::trust::models::{
    TrustImpactDirection, TrustRelationshipSignal, TrustSignalKind, TrustSourceReliabilitySignal,
    normalize_compatibility_score, validate_confidence, validate_non_empty,
};

pub struct TrustEngine;

impl TrustEngine {
    pub fn persona_compatibility_score_signal(score: i16) -> TrustRelationshipSignal {
        TrustRelationshipSignal {
            kind: TrustSignalKind::PersonaCompatibilityScore,
            relationship_type: "trusts",
            trust_score: normalize_compatibility_score(score),
            strength_score: 0.5,
            confidence: 1.0,
            explanation: "compatibility personas.trust_score signal",
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
