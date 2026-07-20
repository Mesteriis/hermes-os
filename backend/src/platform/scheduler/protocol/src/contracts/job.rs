//! Stable owner-neutral job identities and payload contract bindings.

const MAX_JOB_TOKEN_BYTES: usize = 64;
const MAX_CONTRACT_NAME_BYTES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct JobKindV1 {
    owner: String,
    name: String,
    major: u16,
}

impl JobKindV1 {
    pub fn new(owner: String, name: String, major: u16) -> Result<Self, JobKindErrorV1> {
        if !valid_token(&owner) || !valid_token(&name) || major == 0 {
            return Err(JobKindErrorV1::InvalidJobKind);
        }
        Ok(Self { owner, name, major })
    }

    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn major(&self) -> u16 {
        self.major
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobContractBindingV1 {
    job_kind: JobKindV1,
    contract_name: String,
    contract_revision: u32,
    schema_sha256: [u8; 32],
}

impl JobContractBindingV1 {
    pub fn new(
        job_kind: JobKindV1,
        contract_name: String,
        contract_revision: u32,
        schema_sha256: [u8; 32],
    ) -> Result<Self, JobKindErrorV1> {
        if !valid_contract_name(&contract_name)
            || contract_revision == 0
            || schema_sha256.iter().all(|byte| *byte == 0)
        {
            return Err(JobKindErrorV1::InvalidContractBinding);
        }
        Ok(Self {
            job_kind,
            contract_name,
            contract_revision,
            schema_sha256,
        })
    }

    #[must_use]
    pub fn job_kind(&self) -> &JobKindV1 {
        &self.job_kind
    }

    #[must_use]
    pub fn contract_name(&self) -> &str {
        &self.contract_name
    }

    #[must_use]
    pub const fn contract_revision(&self) -> u32 {
        self.contract_revision
    }

    #[must_use]
    pub const fn schema_sha256(&self) -> &[u8; 32] {
        &self.schema_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobKindErrorV1 {
    InvalidJobKind,
    InvalidContractBinding,
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_JOB_TOKEN_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_contract_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CONTRACT_NAME_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}
