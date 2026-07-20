//! Atomic execution of previously admitted immutable migration bundles.

mod execution;

pub use execution::apply_storage_bundle;
