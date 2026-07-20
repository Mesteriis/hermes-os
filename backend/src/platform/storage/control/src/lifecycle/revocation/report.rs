//! Sanitized evidence for a complete or incomplete revoke attempt.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageRevocationReportV1 {
    vault_lease_invalidated: bool,
    pool_paused: bool,
    pool_disabled: bool,
    pool_killed: bool,
    postgres_role_fenced: bool,
}

impl StorageRevocationReportV1 {
    pub(crate) const fn new(
        vault_lease_invalidated: bool,
        pool_paused: bool,
        pool_disabled: bool,
        pool_killed: bool,
        postgres_role_fenced: bool,
    ) -> Self {
        Self {
            vault_lease_invalidated,
            pool_paused,
            pool_disabled,
            pool_killed,
            postgres_role_fenced,
        }
    }

    pub const fn is_complete(self) -> bool {
        self.vault_lease_invalidated
            && self.pool_paused
            && self.pool_disabled
            && self.pool_killed
            && self.postgres_role_fenced
    }

    pub const fn vault_lease_invalidated(self) -> bool {
        self.vault_lease_invalidated
    }

    pub const fn pool_paused(self) -> bool {
        self.pool_paused
    }

    pub const fn pool_disabled(self) -> bool {
        self.pool_disabled
    }

    pub const fn pool_killed(self) -> bool {
        self.pool_killed
    }

    pub const fn postgres_role_fenced(self) -> bool {
        self.postgres_role_fenced
    }
}
