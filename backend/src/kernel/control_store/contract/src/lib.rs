//! Typed, persistence-agnostic Kernel Control Store boundary.

mod distribution;
mod identity;
mod modules;
mod operation;
mod ports;
mod recovery;
mod runtime;
mod state;

pub use distribution::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, OwnerPinnedArtifactBinding,
    OwnerPinnedArtifactBindingInputV1,
};
pub use identity::{
    BrowserDeviceEnrollmentInputV1, BrowserDeviceEnrollmentV1, BrowserDeviceIdentityV1,
    BrowserDeviceStateV1, InitialOwnerIdentity, ServerBootstrapPairing,
};
pub use modules::{
    GrantSet, ModuleBlobQuotaRequestV1, ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1,
    ModuleEventRouteDirectionV1, ModuleEventRouteRequestV1, ModuleEventSubscriptionRequirementV1,
    ModuleGrantSnapshot, ModuleRegistration, ModuleRegistrationState, ModuleSchedulerJobRequestV1,
    ModuleStorageRequestV1, SettingsApplyState, SettingsDesiredSnapshot, SettingsSchemaBinding,
};
pub use operation::{
    OperationAdmissionV1, OperationIdV1, OperationStatusV1, OperationTerminalOutcomeV1,
};
pub use ports::{
    EventHubTopologyStore, EventsAuthorityStore, HealthRecoveryStore, ModuleRegistryStore,
    OperationJournalStore, OwnerIdentityStore, RuntimeTrustStore, SettingsRegistryStore,
    StorageBindingStore, StorageBundleStore, StorageTopologyStore,
};
pub use recovery::RecoveryFences;
pub use runtime::{
    ExternalRuntimeAttestation, ExternalRuntimeIdentity, PlatformEventHubTopologyV1,
    PlatformEventStreamBudgetV1, PlatformEventsAuthorityConfigurationV1,
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch, PlatformStorageBindingErrorV1,
    PlatformStorageBindingInputV1, PlatformStorageBindingStateV1, PlatformStorageBindingV1,
    PlatformStorageBundleErrorV1, PlatformStorageBundleV1, PlatformStorageEndpointV1,
    PlatformStorageTopology, PlatformStorageTopologyInputV1, StorageDeploymentProfileV1,
};
pub use state::{ControlStore, StoreHealth};
