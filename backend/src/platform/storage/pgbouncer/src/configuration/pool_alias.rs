//! Opaque alias scoped to a runtime generation.

use super::PoolConfigErrorV1;
use hermes_storage_protocol::StorageBindingV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolAliasV1(String);

impl PoolAliasV1 {
    pub fn new(registration_id: &str, runtime_generation: u64) -> Result<Self, PoolConfigErrorV1> {
        if !valid_identifier(registration_id) || runtime_generation == 0 {
            return Err(PoolConfigErrorV1::Identifier);
        }
        Ok(Self(format!(
            "runtime_{registration_id}_{runtime_generation}"
        )))
    }

    pub fn from_binding(binding: &StorageBindingV1) -> Result<Self, PoolConfigErrorV1> {
        let alias = Self::new(
            binding.identity().registration_id(),
            binding.fences().runtime_generation(),
        )?;
        if alias.as_str() != binding.access().pool_alias() {
            return Err(PoolConfigErrorV1::Identifier);
        }
        Ok(alias)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
