//! Reasons a fenced storage binding lifecycle transition is refused.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageLifecycleErrorV1 {
    RotationRequiresRevocation,
    RevocationInProgress,
    NotRevoking,
}
