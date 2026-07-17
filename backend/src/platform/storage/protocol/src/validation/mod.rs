//! Validation for canonical Storage Control protocol messages.

mod binding;
mod bundle;

pub use binding::validate_storage_binding_message;
pub use bundle::{StorageBundleValidationErrorV1, validate_storage_bundle};
