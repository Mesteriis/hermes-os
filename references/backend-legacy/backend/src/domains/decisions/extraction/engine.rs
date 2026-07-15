use super::detection::{detect_decision, sentences};
use super::errors::DecisionEngineError;
use super::models::{DecisionExtractionInput, DecisionExtractionResult};

pub struct DecisionEngine;

impl DecisionEngine {
    pub fn detect_candidates(
        input: &DecisionExtractionInput,
    ) -> Result<DecisionExtractionResult, DecisionEngineError> {
        input.validate()?;

        let mut result = DecisionExtractionResult::default();
        for sentence in sentences(&input.text) {
            if let Some(candidate) = detect_decision(input, sentence) {
                result.decisions.push(candidate);
            }
        }

        Ok(result)
    }
}
