//! Validation for canonical Storage Control protocol messages.

mod binding;
mod bundle;
mod configuration;
mod runtime;
mod topology;

pub use binding::{storage_binding_from_message, validate_storage_binding_message};
pub use bundle::{StorageBundleValidationErrorV1, validate_storage_bundle};
pub use configuration::{
    StorageRuntimeConfigurationErrorV1, validate_storage_runtime_configuration,
};
pub use runtime::{
    StorageRuntimeControlErrorV1, StorageRuntimeStatusErrorV1,
    validate_storage_runtime_control_request, validate_storage_runtime_control_response,
    validate_storage_runtime_status,
};
pub use topology::{StorageRuntimeTopologyErrorV1, validate_storage_runtime_topology};
