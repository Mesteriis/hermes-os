//! Reasons Storage Control refuses a replacement binding.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageReconciliationErrorV1 {
    IdentityMismatch,
    StaleStorageGeneration,
}
