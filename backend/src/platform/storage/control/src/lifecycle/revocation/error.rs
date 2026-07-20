//! Fail-closed outcomes for a Storage revoke sequence.

use super::StorageRevocationReportV1;
use crate::StorageLifecycleErrorV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRevocationErrorV1 {
    Lifecycle(StorageLifecycleErrorV1),
    Incomplete(StorageRevocationReportV1),
}
