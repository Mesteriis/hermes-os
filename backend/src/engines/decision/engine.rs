use crate::engines::decision::detection::{detect_decision, sentences};
use crate::engines::decision::errors::DecisionEngineError;
use crate::engines::decision::models::{DecisionExtractionInput, DecisionExtractionResult};

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
