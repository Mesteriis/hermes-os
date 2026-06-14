use serde_json::{Value, json};

use super::CallError;
use super::validation::validate_non_empty;

pub trait SpeechToTextProvider {
    fn provider_name(&self) -> &'static str;
    fn transcribe_fixture(&self, audio_ref: &str) -> Result<FixtureTranscript, CallError>;
}

pub struct FixtureSpeechToTextProvider;

impl SpeechToTextProvider for FixtureSpeechToTextProvider {
    fn provider_name(&self) -> &'static str {
        "fixture-stt"
    }

    fn transcribe_fixture(&self, audio_ref: &str) -> Result<FixtureTranscript, CallError> {
        validate_non_empty("audio_ref", audio_ref)?;
        Ok(FixtureTranscript {
            text: format!("Fixture transcript for {audio_ref}: follow up on the Telegram call."),
            segments: json!([
                {
                    "speaker": "local",
                    "start_ms": 0,
                    "end_ms": 2400,
                    "text": "follow up on the Telegram call"
                }
            ]),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureTranscript {
    pub text: String,
    pub segments: Value,
}
