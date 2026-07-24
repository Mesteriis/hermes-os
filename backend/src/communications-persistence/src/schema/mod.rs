//! Immutable Communications-owned schema artifacts.

mod bundle;

pub use bundle::{COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1, communications_storage_bundle_v1};

pub const COMMUNICATIONS_SCHEMA_V1: &str =
    include_str!("../../migrations/0001_communications_state.sql");
