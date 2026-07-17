//! Immutable identities carried by a fenced storage binding.

use super::StorageBindingErrorV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageBindingIdentityV1 {
    storage_instance_id: String,
    database_id: String,
    owner: String,
    registration_id: String,
    runtime_instance_id: String,
}

impl StorageBindingIdentityV1 {
    pub fn new(
        storage_instance_id: String,
        database_id: String,
        owner: String,
        registration_id: String,
        runtime_instance_id: String,
    ) -> Result<Self, StorageBindingErrorV1> {
        if !valid_identifier(&storage_instance_id)
            || !valid_identifier(&database_id)
            || !valid_identifier(&registration_id)
            || !valid_identifier(&runtime_instance_id)
        {
            return Err(StorageBindingErrorV1::Identifier);
        }
        if !valid_owner(&owner) {
            return Err(StorageBindingErrorV1::Owner);
        }
        Ok(Self {
            storage_instance_id,
            database_id,
            owner,
            registration_id,
            runtime_instance_id,
        })
    }

    pub fn storage_instance_id(&self) -> &str {
        &self.storage_instance_id
    }

    pub fn database_id(&self) -> &str {
        &self.database_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
