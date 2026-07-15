use super::detection::{detect_commitment, sentences};
use super::errors::ObligationEngineError;
use super::models::{
    FollowUpCandidate, ObligationExtractionInput, ObligationExtractionResult,
    ObligationTaskCandidate,
};

pub struct ObligationEngine;

impl ObligationEngine {
    pub fn detect_candidates(
        input: &ObligationExtractionInput,
    ) -> Result<ObligationExtractionResult, ObligationEngineError> {
        input.validate()?;

        let mut result = ObligationExtractionResult::default();
        for sentence in sentences(&input.text) {
            if let Some(candidate) = detect_commitment(input, sentence) {
                result
                    .task_candidates
                    .push(ObligationTaskCandidate::from_obligation(&candidate));
                result
                    .follow_ups
                    .push(FollowUpCandidate::from_obligation(&candidate));
                result.obligations.push(candidate);
            }
        }

        Ok(result)
    }
}
