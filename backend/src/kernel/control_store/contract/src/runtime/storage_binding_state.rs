//! Durable lifecycle state for one non-secret Storage binding.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformStorageBindingStateV1 {
    Active,
    Revoking,
}

impl PlatformStorageBindingStateV1 {
    pub const fn as_sql(self) -> i64 {
        match self {
            Self::Active => 1,
            Self::Revoking => 2,
        }
    }

    pub const fn from_sql(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::Active),
            2 => Some(Self::Revoking),
            _ => None,
        }
    }
}
