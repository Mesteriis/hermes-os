//! Narrow persistence ports consumed by individual Kernel subsystems.

mod health_recovery;
mod module_registry;
mod owner_identity;
mod runtime_trust;
mod settings_registry;

pub use health_recovery::HealthRecoveryStore;
pub use module_registry::ModuleRegistryStore;
pub use owner_identity::OwnerIdentityStore;
pub use runtime_trust::RuntimeTrustStore;
pub use settings_registry::SettingsRegistryStore;
