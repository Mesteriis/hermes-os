//! Fenced Scheduler runtime identity embedded in an owner-neutral command envelope.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerDispatchIdentityV1 {
    runtime_id: String,
    runtime_instance_id: [u8; 16],
    runtime_generation: u64,
}

impl SchedulerDispatchIdentityV1 {
    pub fn new(
        runtime_id: String,
        runtime_instance_id: [u8; 16],
        runtime_generation: u64,
    ) -> Result<Self, SchedulerDispatchIdentityErrorV1> {
        (valid_runtime_id(&runtime_id)
            && runtime_instance_id.iter().any(|byte| *byte != 0)
            && runtime_generation > 0)
            .then_some(Self {
                runtime_id,
                runtime_instance_id,
                runtime_generation,
            })
            .ok_or(SchedulerDispatchIdentityErrorV1::Invalid)
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    #[must_use]
    pub const fn runtime_instance_id(&self) -> [u8; 16] {
        self.runtime_instance_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerDispatchIdentityErrorV1 {
    Invalid,
}

fn valid_runtime_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
