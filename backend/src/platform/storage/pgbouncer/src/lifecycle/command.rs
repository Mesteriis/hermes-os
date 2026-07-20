//! Renders fixed PgBouncer admin commands for one opaque alias.

use crate::PoolAliasV1;

use super::PoolLifecycleErrorV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolLifecycleCommandV1 {
    Pause,
    Resume,
    Disable,
    Kill,
}

impl PoolLifecycleCommandV1 {
    pub fn render(self, alias: &PoolAliasV1) -> Result<String, PoolLifecycleErrorV1> {
        let name = alias.as_str();
        if !valid_alias(name) {
            return Err(PoolLifecycleErrorV1::InvalidAlias);
        }
        Ok(format!("{} {name}", self.verb()))
    }

    const fn verb(self) -> &'static str {
        match self {
            Self::Pause => "PAUSE",
            Self::Resume => "RESUME",
            Self::Disable => "DISABLE",
            Self::Kill => "KILL",
        }
    }
}

fn valid_alias(value: &str) -> bool {
    value.starts_with("runtime_")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
