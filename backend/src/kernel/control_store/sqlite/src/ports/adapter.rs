//! Narrow Control Store port implementations over the SQLite actor facade.

use hermes_kernel_control_store::{
    BundledManagedLaunchBinding, ExternalRuntimeAttestation, ExternalRuntimeIdentity, GrantSet,
    HealthRecoveryStore, InitialOwnerIdentity, ManagedLaunchRecord, ModuleGrantSnapshot,
    ModuleRegistration, ModuleRegistrationState, ModuleRegistryStore, OwnerIdentityStore,
    OwnerPinnedArtifactBinding, PlatformManagedProcessBinding, PlatformManagedProcessLaunch,
    RuntimeTrustStore, ServerBootstrapPairing, SettingsApplyState, SettingsDesiredSnapshot,
    SettingsRegistryStore, SettingsSchemaBinding,
};

use crate::{SqliteControlStore, StoreError};

impl HealthRecoveryStore for SqliteControlStore {
    fn control_store_snapshot(&self) -> &hermes_kernel_control_store::ControlStore {
        self.snapshot()
    }
}

impl OwnerIdentityStore for SqliteControlStore {
    type Error = StoreError;

    fn initial_owner_identity(&self) -> Result<Option<InitialOwnerIdentity>, Self::Error> {
        SqliteControlStore::initial_owner_identity(self)
    }
    fn claim_initial_owner(&self, identity: &InitialOwnerIdentity) -> Result<(), Self::Error> {
        SqliteControlStore::claim_initial_owner(self, identity)
    }
    fn begin_server_bootstrap_pairing(
        &self,
        pairing: &ServerBootstrapPairing,
        now_unix_ms: u64,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::begin_server_bootstrap_pairing(self, pairing, now_unix_ms)
    }
    fn claim_initial_owner_from_server_bootstrap_pairing(
        &self,
        identity: &InitialOwnerIdentity,
        token: &[u8; 32],
        now_unix_ms: u64,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::claim_initial_owner_from_server_bootstrap_pairing(
            self,
            identity,
            token,
            now_unix_ms,
        )
    }
}

impl ModuleRegistryStore for SqliteControlStore {
    type Error = StoreError;

    fn create_pending_registration(
        &self,
        registration: &ModuleRegistration,
        capabilities: &[String],
    ) -> Result<(), Self::Error> {
        SqliteControlStore::create_pending_registration(self, registration, capabilities)
    }
    fn module_registration(&self, id: &str) -> Result<Option<ModuleRegistration>, Self::Error> {
        SqliteControlStore::module_registration(self, id)
    }
    fn transition_module_registration(
        &self,
        id: &str,
        next: ModuleRegistrationState,
    ) -> Result<ModuleRegistration, Self::Error> {
        SqliteControlStore::transition_module_registration(self, id, next)
    }
    fn approve_module_registration(
        &self,
        id: &str,
        capabilities: &[String],
    ) -> Result<GrantSet, Self::Error> {
        SqliteControlStore::approve_module_registration(self, id, capabilities)
    }
    fn module_grant_snapshot(&self, id: &str) -> Result<Option<ModuleGrantSnapshot>, Self::Error> {
        SqliteControlStore::module_grant_snapshot(self, id)
    }
}

impl SettingsRegistryStore for SqliteControlStore {
    type Error = StoreError;

    fn admit_settings_schema(
        &self,
        binding: &SettingsSchemaBinding,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        SqliteControlStore::admit_settings_schema(self, binding, bytes)
    }
    fn settings_schema_artifact(&self, id: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        SqliteControlStore::settings_schema_artifact(self, id)
    }
    fn settings_schema_binding(
        &self,
        id: &str,
    ) -> Result<Option<SettingsSchemaBinding>, Self::Error> {
        SqliteControlStore::settings_schema_binding(self, id)
    }
    fn commit_desired_settings_snapshot(
        &self,
        update: &SettingsDesiredSnapshot,
    ) -> Result<u64, Self::Error> {
        SqliteControlStore::commit_desired_settings_snapshot(self, update)
    }
    fn desired_settings_snapshot(&self, id: &str) -> Result<Option<(u64, Vec<u8>)>, Self::Error> {
        SqliteControlStore::desired_settings_snapshot(self, id)
    }
    fn transition_settings_apply_state(
        &self,
        id: &str,
        revision: u64,
        next: SettingsApplyState,
        reason: Option<&str>,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::transition_settings_apply_state(self, id, revision, next, reason)
    }
    fn confirm_effective_settings_revision(
        &self,
        id: &str,
        revision: u64,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::confirm_effective_settings_revision(self, id, revision)
    }
}

impl RuntimeTrustStore for SqliteControlStore {
    type Error = StoreError;

    fn attest_external_runtime(
        &self,
        value: &ExternalRuntimeAttestation,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::attest_external_runtime(self, value)
    }
    fn effective_external_runtime_attestation(
        &self,
        id: &str,
    ) -> Result<Option<ExternalRuntimeAttestation>, Self::Error> {
        SqliteControlStore::effective_external_runtime_attestation(self, id)
    }
    fn bind_external_runtime_identity(
        &self,
        value: &ExternalRuntimeIdentity,
    ) -> Result<ModuleRegistration, Self::Error> {
        SqliteControlStore::bind_external_runtime_identity(self, value)
    }
    fn external_runtime_identity(
        &self,
        id: &str,
    ) -> Result<Option<ExternalRuntimeIdentity>, Self::Error> {
        SqliteControlStore::external_runtime_identity(self, id)
    }
    fn record_owner_pinned_artifact_binding(
        &self,
        value: &OwnerPinnedArtifactBinding,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_owner_pinned_artifact_binding(self, value)
    }
    fn effective_owner_pinned_artifact_binding(
        &self,
        id: &str,
    ) -> Result<Option<OwnerPinnedArtifactBinding>, Self::Error> {
        SqliteControlStore::effective_owner_pinned_artifact_binding(self, id)
    }
    fn record_bundled_managed_launch_binding(
        &self,
        value: &BundledManagedLaunchBinding,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_bundled_managed_launch_binding(self, value)
    }
    fn effective_bundled_managed_launch_binding(
        &self,
        id: &str,
    ) -> Result<Option<BundledManagedLaunchBinding>, Self::Error> {
        SqliteControlStore::effective_bundled_managed_launch_binding(self, id)
    }
    fn record_managed_launch(&self, value: &ManagedLaunchRecord) -> Result<(), Self::Error> {
        SqliteControlStore::record_managed_launch(self, value)
    }
    fn effective_managed_launch_record(
        &self,
        id: &str,
    ) -> Result<Option<ManagedLaunchRecord>, Self::Error> {
        SqliteControlStore::effective_managed_launch_record(self, id)
    }
    fn record_platform_managed_process_binding(
        &self,
        value: &PlatformManagedProcessBinding,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_platform_managed_process_binding(self, value)
    }
    fn platform_managed_process_binding(
        &self,
        id: &str,
    ) -> Result<Option<PlatformManagedProcessBinding>, Self::Error> {
        SqliteControlStore::platform_managed_process_binding(self, id)
    }
    fn record_platform_managed_process_launch(
        &self,
        value: &PlatformManagedProcessLaunch,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_platform_managed_process_launch(self, value)
    }
    fn platform_managed_process_launch(
        &self,
        id: &str,
    ) -> Result<Option<PlatformManagedProcessLaunch>, Self::Error> {
        SqliteControlStore::platform_managed_process_launch(self, id)
    }
}
