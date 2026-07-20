//! Narrow persistence ports consumed by individual Kernel subsystems.

mod event_hub_topology;
mod events_authority;
mod health_recovery;
mod module_registry;
mod operation_journal;
mod owner_identity;
mod runtime_trust;
mod settings_registry;
mod storage_binding;
mod storage_bundle;
mod storage_topology;

pub use event_hub_topology::EventHubTopologyStore;
pub use events_authority::EventsAuthorityStore;
pub use health_recovery::HealthRecoveryStore;
pub use module_registry::ModuleRegistryStore;
pub use operation_journal::OperationJournalStore;
pub use owner_identity::OwnerIdentityStore;
pub use runtime_trust::RuntimeTrustStore;
pub use settings_registry::SettingsRegistryStore;
pub use storage_binding::StorageBindingStore;
pub use storage_bundle::StorageBundleStore;
pub use storage_topology::StorageTopologyStore;
