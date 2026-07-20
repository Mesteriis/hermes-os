//! Invalid field classes for a StorageBindingV1.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBindingErrorV1 {
    Identifier,
    Owner,
    Fence,
    Budget,
    Digest,
    PoolAlias,
}
