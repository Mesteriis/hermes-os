//! NATS identities and local publish permits.

use std::collections::BTreeSet;
use std::fmt;

use crate::{subjects::DurableSubjectV1, topology::ConsumerSpecV1};

/// Runtime identity fenced to one generation and grant epoch.
pub struct RuntimeNatsIdentity {
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

/// Password authentication material for one broker connection.
///
/// This is a transitional adapter for the existing authenticated contour. It is
/// intentionally separate from runtime and Event Hub identity so JWT credentials
/// can replace it without changing fencing semantics.
pub struct NatsPasswordCredentialV1 {
    username: String,
    password: zeroize::Zeroizing<String>,
}

/// Kernel-resolved publish fence for one runtime generation and grant epoch.
pub struct RuntimePublishPermitV1 {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    subjects: BTreeSet<String>,
}

/// Kernel-resolved pull-consumer fence for one runtime generation and grant epoch.
#[derive(Clone)]
pub struct RuntimeSubscribePermitV1 {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    consumer: ConsumerSpecV1,
}

impl NatsPasswordCredentialV1 {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Result<Self, String> {
        let username = username.into();
        let password = password.into();
        (valid_credential_id(&username) && !password.is_empty() && password.len() <= 512)
            .then_some(Self {
                username,
                password: zeroize::Zeroizing::new(password),
            })
            .ok_or_else(|| "NATS password credential is invalid".to_owned())
    }

    pub(super) fn credentials(&self) -> (&str, &str) {
        (&self.username, self.password.as_str())
    }
}

impl RuntimeNatsIdentity {
    pub fn new(
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
    ) -> Result<Self, String> {
        let runtime_id = runtime_id.into();
        (valid_runtime_id(&runtime_id) && runtime_generation > 0 && grant_epoch > 0)
            .then_some(Self {
                runtime_id,
                runtime_generation,
                grant_epoch,
            })
            .ok_or_else(|| "NATS runtime identity is invalid".to_owned())
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

impl RuntimePublishPermitV1 {
    pub fn new(
        registration_id: impl Into<String>,
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        subjects: Vec<DurableSubjectV1>,
    ) -> Result<Self, String> {
        let registration_id = registration_id.into();
        let runtime_id = runtime_id.into();
        let subjects: BTreeSet<String> = subjects
            .into_iter()
            .map(|subject| subject.as_str())
            .collect();
        (valid_runtime_id(&registration_id)
            && valid_runtime_id(&runtime_id)
            && runtime_generation > 0
            && grant_epoch > 0
            && !subjects.is_empty())
        .then_some(Self {
            registration_id,
            runtime_id,
            runtime_generation,
            grant_epoch,
            subjects,
        })
        .ok_or_else(|| "NATS runtime publish permit is invalid".to_owned())
    }

    pub(super) fn permits(&self, identity: &RuntimeNatsIdentity, subject: &str) -> bool {
        self.runtime_id == identity.runtime_id
            && self.runtime_generation == identity.runtime_generation
            && self.grant_epoch == identity.grant_epoch
            && self.subjects.contains(subject)
    }
}

impl RuntimeSubscribePermitV1 {
    pub fn new(
        registration_id: impl Into<String>,
        runtime_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        consumer: ConsumerSpecV1,
    ) -> Result<Self, String> {
        let registration_id = registration_id.into();
        let runtime_id = runtime_id.into();
        (valid_runtime_id(&registration_id)
            && valid_runtime_id(&runtime_id)
            && runtime_generation > 0
            && grant_epoch > 0)
            .then_some(Self {
                registration_id,
                runtime_id,
                runtime_generation,
                grant_epoch,
                consumer,
            })
            .ok_or_else(|| "NATS runtime subscribe permit is invalid".to_owned())
    }

    #[must_use]
    pub fn consumer(&self) -> &ConsumerSpecV1 {
        &self.consumer
    }

    pub(super) fn permits(&self, identity: &RuntimeNatsIdentity) -> bool {
        self.runtime_id == identity.runtime_id
            && self.runtime_generation == identity.runtime_generation
            && self.grant_epoch == identity.grant_epoch
    }
}

impl fmt::Debug for NatsPasswordCredentialV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NatsPasswordCredentialV1")
            .field("username", &"[redacted]")
            .field("password", &"[redacted]")
            .finish()
    }
}

impl fmt::Debug for RuntimeNatsIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeNatsIdentity")
            .field("runtime_id", &self.runtime_id)
            .field("runtime_generation", &self.runtime_generation)
            .field("grant_epoch", &self.grant_epoch)
            .finish()
    }
}

impl fmt::Debug for RuntimePublishPermitV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimePublishPermitV1")
            .field("registration_id", &self.registration_id)
            .field("runtime_id", &self.runtime_id)
            .field("runtime_generation", &self.runtime_generation)
            .field("grant_epoch", &self.grant_epoch)
            .field("subject_count", &self.subjects.len())
            .finish()
    }
}

impl fmt::Debug for RuntimeSubscribePermitV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeSubscribePermitV1")
            .field("registration_id", &self.registration_id)
            .field("runtime_id", &self.runtime_id)
            .field("runtime_generation", &self.runtime_generation)
            .field("grant_epoch", &self.grant_epoch)
            .field("stream_kind", &self.consumer.stream_kind())
            .field("durable_name", &self.consumer.durable_name())
            .field("filter_subject", &self.consumer.filter_subject())
            .finish()
    }
}

fn valid_runtime_id(value: &str) -> bool {
    valid_credential_id(value)
}

fn valid_credential_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
