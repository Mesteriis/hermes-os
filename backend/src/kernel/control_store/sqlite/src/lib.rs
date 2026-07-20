//! SQLite-backed private Kernel Control Store adapter.

mod actor;
mod configuration;
mod database;
mod module_state;
mod owner;
mod ports;
mod recovery;
mod schema;

pub use database::error::StoreError;
pub use database::store::SqliteControlStore;
pub use recovery::{ControlStoreExport, StagedControlStoreRestore};

pub(crate) use database::validation::{
    module_registration_state_from_str, settings_apply_state_from_str, valid_capability_ids,
    valid_identity_token, valid_owner_pinned_artifact_binding, valid_sanitized_reason_code,
    valid_settings_binding_state,
};
