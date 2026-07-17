//! Error taxonomy for the private SQLite Control Store adapter.

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    MissingMetadata,
    UnsupportedSchema(i64),
    MigrationInvariant { expected: i64, actual: i64 },
    MigrationSchemaAssertion { version: i64 },
    InvalidGeneration,
    RecoveryFenceOverflow,
    InstallationIdentityMismatch,
    InvalidExportDestination,
    IntegrityCheckFailed(String),
    InitialOwnerAlreadyClaimed,
    InvalidInitialOwnerIdentity,
    InvalidServerBootstrapPairing,
    ServerBootstrapPairingAlreadyActive,
    ServerBootstrapPairingMissing,
    ServerBootstrapPairingExpired,
    ServerBootstrapPairingTokenRejected,
    InvalidModuleRegistration,
    ModuleRegistrationAlreadyExists,
    ModuleRegistrationMissing,
    InvalidModuleRegistrationTransition,
    InvalidCapabilityGrant,
    InvalidExternalRuntimeAttestation,
    StaleExternalRuntimeAttestation,
    InvalidExternalRuntimeIdentity,
    ExternalRuntimeIdentityAlreadyBound,
    InvalidBundledManagedLaunchBinding,
    BundledManagedLaunchBindingRevisionConflict,
    InvalidManagedLaunchRecord,
    StaleManagedLaunchRecord,
    InvalidPlatformManagedProcessBinding,
    PlatformManagedProcessBindingRevisionConflict,
    InvalidPlatformManagedProcessLaunch,
    StalePlatformManagedProcessLaunch,
    InvalidOwnerPinnedArtifactBinding,
    OwnerPinnedArtifactBindingRevisionConflict,
    InvalidSettingsSchemaBinding,
    SettingsSchemaRevisionCollision,
    SettingsRevisionConflict,
    InvalidSettingsApplyState,
    QueueFull,
    DeadlineExceeded,
    ActorStopped,
}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
