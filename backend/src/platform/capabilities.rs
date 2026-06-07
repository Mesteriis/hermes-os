use serde::Serialize;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CapabilityActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    Export,
    SecretAccess,
    Automation,
}

impl CapabilityActionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::Export => "export",
            Self::SecretAccess => "secret_access",
            Self::Automation => "automation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CapabilityDecisionStatus {
    Allowed,
    Rejected,
}

impl CapabilityDecisionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityDecision {
    action_class: CapabilityActionClass,
    capability: String,
    decision: CapabilityDecisionStatus,
    reason: String,
    confirmation_required: bool,
    scoped_automation_policy: bool,
    automation_policy_id: Option<String>,
}

impl CapabilityDecision {
    pub fn scoped_automation_allowed(
        capability: impl Into<String>,
        automation_policy_id: impl Into<String>,
    ) -> Self {
        Self {
            action_class: CapabilityActionClass::Automation,
            capability: capability.into(),
            decision: CapabilityDecisionStatus::Allowed,
            reason: "scoped_automation_policy_authorized".to_owned(),
            confirmation_required: false,
            scoped_automation_policy: true,
            automation_policy_id: Some(automation_policy_id.into()),
        }
    }

    pub fn rejected_high_risk(
        action_class: CapabilityActionClass,
        capability: impl Into<String>,
        reason: impl Into<String>,
        automation_policy_id: Option<String>,
    ) -> Self {
        Self {
            action_class,
            capability: capability.into(),
            decision: CapabilityDecisionStatus::Rejected,
            reason: reason.into(),
            confirmation_required: true,
            scoped_automation_policy: false,
            automation_policy_id,
        }
    }

    pub fn audit_metadata(&self) -> Value {
        json!({
            "action_class": self.action_class.as_str(),
            "capability": self.capability,
            "decision": self.decision.as_str(),
            "reason": self.reason,
            "confirmation_required": self.confirmation_required,
            "scoped_automation_policy": self.scoped_automation_policy,
            "automation_policy_id": self.automation_policy_id,
        })
    }
}
