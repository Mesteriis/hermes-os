use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
    Suggest,
    AskBeforeExecute,
    AutoExecute,
    DryRun,
}

impl RuleMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleMode::Suggest => "suggest",
            RuleMode::AskBeforeExecute => "ask_before_execute",
            RuleMode::AutoExecute => "auto_execute",
            RuleMode::DryRun => "dry_run",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "suggest" => Some(RuleMode::Suggest),
            "ask_before_execute" => Some(RuleMode::AskBeforeExecute),
            "auto_execute" => Some(RuleMode::AutoExecute),
            "dry_run" => Some(RuleMode::DryRun),
            _ => None,
        }
    }
}

pub(in crate::domains::communications::rules) fn format_mode(mode: RuleMode) -> String {
    mode.as_str().to_owned()
}
