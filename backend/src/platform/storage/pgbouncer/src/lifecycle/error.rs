//! Rejects non-generation-scoped pool lifecycle commands.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolLifecycleErrorV1 {
    InvalidAlias,
}
