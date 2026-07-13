use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalPolicy {
    pub scope: SignalPolicyScope,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub mode: SignalPolicyMode,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalPolicyScope {
    Global,
    Source,
    Connection,
    EventPattern,
    Profile,
}

impl SignalPolicyScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Source => "source",
            Self::Connection => "connection",
            Self::EventPattern => "event_pattern",
            Self::Profile => "profile",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "global" => Some(Self::Global),
            "source" => Some(Self::Source),
            "connection" => Some(Self::Connection),
            "event_pattern" => Some(Self::EventPattern),
            "profile" => Some(Self::Profile),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalPolicyMode {
    Enabled,
    Disabled,
    Muted,
    Paused,
    ReplayOnly,
    FixtureOnly,
}

impl SignalPolicyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Muted => "muted",
            Self::Paused => "paused",
            Self::ReplayOnly => "replay_only",
            Self::FixtureOnly => "fixture_only",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            "muted" => Some(Self::Muted),
            "paused" => Some(Self::Paused),
            "replay_only" => Some(Self::ReplayOnly),
            "fixture_only" => Some(Self::FixtureOnly),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SignalPolicyMode, SignalPolicyScope};

    #[test]
    fn policy_scope_strings_round_trip() {
        for (scope, value) in [
            (SignalPolicyScope::Global, "global"),
            (SignalPolicyScope::Source, "source"),
            (SignalPolicyScope::Connection, "connection"),
            (SignalPolicyScope::EventPattern, "event_pattern"),
            (SignalPolicyScope::Profile, "profile"),
        ] {
            assert_eq!(scope.as_str(), value);
            assert_eq!(SignalPolicyScope::parse(value), Some(scope));
        }
    }

    #[test]
    fn policy_mode_strings_round_trip() {
        for (mode, value) in [
            (SignalPolicyMode::Enabled, "enabled"),
            (SignalPolicyMode::Disabled, "disabled"),
            (SignalPolicyMode::Muted, "muted"),
            (SignalPolicyMode::Paused, "paused"),
            (SignalPolicyMode::ReplayOnly, "replay_only"),
            (SignalPolicyMode::FixtureOnly, "fixture_only"),
        ] {
            assert_eq!(mode.as_str(), value);
            assert_eq!(SignalPolicyMode::parse(value), Some(mode));
        }
    }
}
