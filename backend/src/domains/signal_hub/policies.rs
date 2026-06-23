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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalPolicyDecision {
    Allow,
    Rejected { reason: String },
    Paused { reason: String },
    Muted { reason: String },
}

pub struct SignalPolicyEvaluator {
    now: DateTime<Utc>,
}

impl SignalPolicyEvaluator {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self { now }
    }

    pub fn decide(
        &self,
        source_code: &str,
        connection_id: Option<&str>,
        event_type: &str,
        policies: &[SignalPolicy],
    ) -> SignalPolicyDecision {
        let matching: Vec<&SignalPolicy> = policies
            .iter()
            .filter(|policy| self.policy_applies(policy, source_code, connection_id, event_type))
            .collect();

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Disabled))
        {
            return SignalPolicyDecision::Rejected {
                reason: policy.reason.clone(),
            };
        }

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Paused))
        {
            return SignalPolicyDecision::Paused {
                reason: policy.reason.clone(),
            };
        }

        if let Some(policy) = matching
            .iter()
            .copied()
            .find(|policy| matches!(policy.mode, SignalPolicyMode::Muted))
        {
            return SignalPolicyDecision::Muted {
                reason: policy.reason.clone(),
            };
        }

        SignalPolicyDecision::Allow
    }

    fn policy_applies(
        &self,
        policy: &SignalPolicy,
        source_code: &str,
        connection_id: Option<&str>,
        event_type: &str,
    ) -> bool {
        if policy
            .expires_at
            .is_some_and(|expires_at| expires_at <= self.now)
        {
            return false;
        }

        if policy
            .source_code
            .as_deref()
            .is_some_and(|policy_source| policy_source != source_code)
        {
            return false;
        }

        if policy
            .connection_id
            .as_deref()
            .is_some_and(|policy_connection| Some(policy_connection) != connection_id)
        {
            return false;
        }

        if policy
            .event_pattern
            .as_deref()
            .is_some_and(|pattern| !event_type_matches(pattern, event_type))
        {
            return false;
        }

        match policy.scope {
            SignalPolicyScope::Global => true,
            SignalPolicyScope::Source => policy.source_code.is_some(),
            SignalPolicyScope::Connection => policy.connection_id.is_some(),
            SignalPolicyScope::EventPattern => policy.event_pattern.is_some(),
            SignalPolicyScope::Profile => true,
        }
    }
}

fn event_type_matches(pattern: &str, event_type: &str) -> bool {
    if pattern == event_type {
        return true;
    }

    let Some(prefix) = pattern.strip_suffix(".*") else {
        return false;
    };

    event_type.starts_with(prefix)
}
