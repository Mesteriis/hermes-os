//! Exact durable Event Hub route request retained from one module descriptor.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleEventEnvelopeKindV1 {
    Command,
    Event,
    Observation,
    Result,
    Ack,
}

impl ModuleEventEnvelopeKindV1 {
    #[must_use]
    pub const fn as_i64(self) -> i64 {
        match self {
            Self::Command => 1,
            Self::Event => 2,
            Self::Observation => 3,
            Self::Result => 4,
            Self::Ack => 5,
        }
    }

    #[must_use]
    pub const fn from_i64(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::Command),
            2 => Some(Self::Event),
            3 => Some(Self::Observation),
            4 => Some(Self::Result),
            5 => Some(Self::Ack),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleEventRouteDirectionV1 {
    Publish,
    Consume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleEventSubscriptionRequirementV1 {
    Required,
    Optional,
}

impl ModuleEventSubscriptionRequirementV1 {
    #[must_use]
    pub const fn as_i64(self) -> i64 {
        match self {
            Self::Required => 1,
            Self::Optional => 2,
        }
    }

    #[must_use]
    pub const fn from_i64(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::Required),
            2 => Some(Self::Optional),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleEventDeliveryPolicyV1 {
    requirement: ModuleEventSubscriptionRequirementV1,
    max_deliver: u8,
    ack_wait_millis: u32,
}

impl ModuleEventDeliveryPolicyV1 {
    #[must_use]
    pub const fn new(
        requirement: ModuleEventSubscriptionRequirementV1,
        max_deliver: u8,
        ack_wait_millis: u32,
    ) -> Self {
        Self {
            requirement,
            max_deliver,
            ack_wait_millis,
        }
    }

    #[must_use]
    pub const fn requirement(self) -> ModuleEventSubscriptionRequirementV1 {
        self.requirement
    }

    #[must_use]
    pub const fn max_deliver(self) -> u8 {
        self.max_deliver
    }

    #[must_use]
    pub const fn ack_wait_millis(self) -> u32 {
        self.ack_wait_millis
    }
}

impl ModuleEventRouteDirectionV1 {
    #[must_use]
    pub const fn as_i64(self) -> i64 {
        match self {
            Self::Publish => 1,
            Self::Consume => 2,
        }
    }

    #[must_use]
    pub const fn from_i64(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::Publish),
            2 => Some(Self::Consume),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEventRouteRequestV1 {
    registration_id: String,
    capability_id: String,
    envelope_kind: ModuleEventEnvelopeKindV1,
    contract_owner: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    direction: ModuleEventRouteDirectionV1,
    max_in_flight: u16,
    delivery_policy: Option<ModuleEventDeliveryPolicyV1>,
}

pub struct ModuleEventRouteRequestInputV1 {
    pub registration_id: String,
    pub capability_id: String,
    pub envelope_kind: ModuleEventEnvelopeKindV1,
    pub contract_owner: String,
    pub contract_name: String,
    pub contract_major: u32,
    pub contract_revision: u32,
    pub contract_schema_sha256: [u8; 32],
    pub direction: ModuleEventRouteDirectionV1,
    pub max_in_flight: u16,
    pub delivery_policy: Option<ModuleEventDeliveryPolicyV1>,
}

impl ModuleEventRouteRequestV1 {
    #[must_use]
    pub fn new(fields: ModuleEventRouteRequestInputV1) -> Self {
        Self {
            registration_id: fields.registration_id,
            capability_id: fields.capability_id,
            envelope_kind: fields.envelope_kind,
            contract_owner: fields.contract_owner,
            contract_name: fields.contract_name,
            contract_major: fields.contract_major,
            contract_revision: fields.contract_revision,
            contract_schema_sha256: fields.contract_schema_sha256,
            direction: fields.direction,
            max_in_flight: fields.max_in_flight,
            delivery_policy: fields.delivery_policy,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
    #[must_use]
    pub const fn envelope_kind(&self) -> ModuleEventEnvelopeKindV1 {
        self.envelope_kind
    }
    #[must_use]
    pub fn contract_owner(&self) -> &str {
        &self.contract_owner
    }
    #[must_use]
    pub fn contract_name(&self) -> &str {
        &self.contract_name
    }
    #[must_use]
    pub const fn contract_major(&self) -> u32 {
        self.contract_major
    }
    #[must_use]
    pub const fn contract_revision(&self) -> u32 {
        self.contract_revision
    }
    #[must_use]
    pub const fn contract_schema_sha256(&self) -> &[u8; 32] {
        &self.contract_schema_sha256
    }
    #[must_use]
    pub const fn direction(&self) -> ModuleEventRouteDirectionV1 {
        self.direction
    }
    #[must_use]
    pub const fn max_in_flight(&self) -> u16 {
        self.max_in_flight
    }
    #[must_use]
    pub const fn delivery_policy(&self) -> Option<ModuleEventDeliveryPolicyV1> {
        self.delivery_policy
    }
}
