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
pub use identity::{
    BrowserDeviceEnrollmentV1, BrowserDeviceIdentityV1, BrowserDeviceStateV1, InitialOwnerIdentity,
    ServerBootstrapPairing,
};
pub use modules::{
    GrantSet, ModuleBlobQuotaRequestV1, ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1,
    ModuleEventRouteDirectionV1, ModuleEventRouteRequestV1, ModuleEventSubscriptionRequirementV1,
    ModuleGrantSnapshot, ModuleRegistration, ModuleRegistrationState, ModuleSchedulerJobRequestV1,
    ModuleStorageRequestV1, SettingsApplyState, SettingsDesiredSnapshot, SettingsSchemaBinding,
};
pub use ports::{
    EventHubTopologyStore, EventsAuthorityStore, HealthRecoveryStore, ModuleRegistryStore,
    OwnerIdentityStore, RuntimeTrustStore, SettingsRegistryStore, StorageBindingStore,
    StorageBundleStore, StorageTopologyStore,
};
pub use recovery::RecoveryFences;
pub use runtime::{
    ExternalRuntimeAttestation, ExternalRuntimeIdentity, PlatformEventHubTopologyV1,
    PlatformEventStreamBudgetV1, PlatformEventsAuthorityConfigurationV1,
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch, PlatformStorageBindingStateV1,
    PlatformStorageBindingV1, PlatformStorageBundleV1, PlatformStorageEndpointV1,
    PlatformStorageTopology, StorageDeploymentProfileV1,
};
pub use state::{ControlStore, StoreHealth};
