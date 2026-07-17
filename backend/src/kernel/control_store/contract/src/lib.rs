//! Typed, persistence-agnostic Kernel Control Store boundary.

mod distribution;
mod identity;
mod modules;
mod ports;
mod recovery;
mod runtime;
mod state;

pub use distribution::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, OwnerPinnedArtifactBinding,
};
pub use identity::{InitialOwnerIdentity, ServerBootstrapPairing};
pub use modules::{
    GrantSet, ModuleGrantSnapshot, ModuleRegistration, ModuleRegistrationState, SettingsApplyState,
    SettingsDesiredSnapshot, SettingsSchemaBinding,
};
pub use ports::{
    HealthRecoveryStore, ModuleRegistryStore, OwnerIdentityStore, RuntimeTrustStore,
    SettingsRegistryStore,
};
pub use recovery::RecoveryFences;
pub use runtime::{
    ExternalRuntimeAttestation, ExternalRuntimeIdentity, PlatformManagedProcessBinding,
    PlatformManagedProcessLaunch,
};
pub use state::{ControlStore, StoreHealth};
