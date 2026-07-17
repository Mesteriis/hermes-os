//! PgBouncer access identity and effective connection limits.

use super::StorageBindingErrorV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageEffectiveBudgetsV1 {
    max_connections: u16,
    statement_timeout_millis: u32,
}

impl StorageEffectiveBudgetsV1 {
    pub fn new(
        max_connections: u16,
        statement_timeout_millis: u32,
    ) -> Result<Self, StorageBindingErrorV1> {
        if max_connections == 0 || statement_timeout_millis == 0 {
            return Err(StorageBindingErrorV1::Budget);
        }
        Ok(Self {
            max_connections,
            statement_timeout_millis,
        })
    }

    pub const fn max_connections(&self) -> u16 {
        self.max_connections
    }

    pub const fn statement_timeout_millis(&self) -> u32 {
        self.statement_timeout_millis
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageBindingAccessV1 {
    runtime_principal: String,
    pool_alias: String,
    effective_budgets: StorageEffectiveBudgetsV1,
    storage_bundle_digest: [u8; 32],
}

impl StorageBindingAccessV1 {
    pub fn new(
        runtime_principal: String,
        pool_alias: String,
        effective_budgets: StorageEffectiveBudgetsV1,
        storage_bundle_digest: [u8; 32],
    ) -> Result<Self, StorageBindingErrorV1> {
        if !valid_alias(&runtime_principal) || !valid_alias(&pool_alias) {
            return Err(StorageBindingErrorV1::Identifier);
        }
        if storage_bundle_digest == [0; 32] {
            return Err(StorageBindingErrorV1::Digest);
        }
        Ok(Self {
            runtime_principal,
            pool_alias,
            effective_budgets,
            storage_bundle_digest,
        })
    }

    pub fn runtime_principal(&self) -> &str {
        &self.runtime_principal
    }

    pub fn pool_alias(&self) -> &str {
        &self.pool_alias
    }

    pub const fn effective_budgets(&self) -> StorageEffectiveBudgetsV1 {
        self.effective_budgets
    }

    pub const fn storage_bundle_digest(&self) -> &[u8; 32] {
        &self.storage_bundle_digest
    }
}

fn valid_alias(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
