//! Narrow Control Store port implementations over the SQLite actor facade.

use hermes_kernel_control_store::{
    BrowserDeviceEnrollmentV1, BrowserDeviceIdentityV1, BundledManagedLaunchBinding, ControlStore,
    EventHubTopologyStore, EventsAuthorityStore, ExternalRuntimeAttestation,
    ExternalRuntimeIdentity, GrantSet, HealthRecoveryStore, InitialOwnerIdentity,
    ManagedLaunchRecord, ModuleBlobQuotaRequestV1, ModuleEventRouteRequestV1, ModuleGrantSnapshot,
    ModuleRegistration, ModuleRegistrationState, ModuleRegistryStore, ModuleSchedulerJobRequestV1, ModuleClientRpcRouteV1,
    ModuleStorageRequestV1, ModuleVaultPurposeRequestV1, OwnerIdentityStore, OwnerPinnedArtifactBinding,
    PlatformEventHubTopologyV1, PlatformEventsAuthorityConfigurationV1,
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch, PlatformStorageTopology,
    RuntimeTrustStore, ServerBootstrapPairing, SettingsApplyState, SettingsDesiredSnapshot,
    SettingsRegistryStore, SettingsSchemaBinding, StorageBindingStore, StorageBundleStore,
    StorageTopologyStore,
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
    fn current_identity_epoch(&self) -> Result<u64, Self::Error> {
        SqliteControlStore::current_identity_epoch(self)
    }
    fn browser_device_identity(
        &self,
        device_id: &str,
    ) -> Result<Option<BrowserDeviceIdentityV1>, Self::Error> {
        SqliteControlStore::browser_device_identity(self, device_id)
    }
    fn browser_device_identity_by_credential_id(
        &self,
        credential_id: &[u8],
    ) -> Result<Option<BrowserDeviceIdentityV1>, Self::Error> {
        SqliteControlStore::browser_device_identity_by_credential_id(self, credential_id)
    }
    fn admit_browser_device(
        &self,
        enrollment: &BrowserDeviceEnrollmentV1,
        expected_identity_epoch: u64,
    ) -> Result<BrowserDeviceIdentityV1, Self::Error> {
        SqliteControlStore::admit_browser_device(self, enrollment, expected_identity_epoch)
    }
    fn record_verified_browser_assertion(
        &self,
        credential_id: &[u8],
        observed_sign_count: u32,
        observed_backup_eligible: bool,
        observed_backup_state: bool,
        expected_identity_epoch: u64,
    ) -> Result<BrowserDeviceIdentityV1, Self::Error> {
        SqliteControlStore::record_verified_browser_assertion(
            self,
            credential_id,
            observed_sign_count,
            observed_backup_eligible,
            observed_backup_state,
            expected_identity_epoch,
        )
    }
    fn revoke_browser_device(
        &self,
        device_id: &str,
        expected_identity_epoch: u64,
    ) -> Result<ControlStore, Self::Error> {
        SqliteControlStore::revoke_browser_device(self, device_id, expected_identity_epoch)
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
    fn create_pending_registration_with_requests(
        &self,
        registration: &ModuleRegistration,
        capabilities: &[String],
        storage_requests: &[ModuleStorageRequestV1],
        event_requests: &[ModuleEventRouteRequestV1],
        blob_requests: &[ModuleBlobQuotaRequestV1],
    ) -> Result<(), Self::Error> {
        SqliteControlStore::create_pending_registration_with_requests(
            self,
            registration,
            capabilities,
            storage_requests,
            event_requests,
            blob_requests,
        )
    }
    fn create_pending_registration_with_descriptor_requests(
        &self,
        registration: &ModuleRegistration,
        capabilities: &[String],
        storage_requests: &[ModuleStorageRequestV1],
        event_requests: &[ModuleEventRouteRequestV1],
        blob_requests: &[ModuleBlobQuotaRequestV1],
        scheduler_requests: &[ModuleSchedulerJobRequestV1],
        vault_purpose_requests: &[ModuleVaultPurposeRequestV1],
    ) -> Result<(), Self::Error> {
        SqliteControlStore::create_pending_registration_with_descriptor_requests(
            self,
            registration,
            capabilities,
            storage_requests,
            event_requests,
            blob_requests,
            scheduler_requests,
            vault_purpose_requests,
        )
    }
    fn create_pending_registration_with_all_descriptor_requests(
        &self,
        registration: &ModuleRegistration,
        capabilities: &[String],
        storage_requests: &[ModuleStorageRequestV1],
        event_requests: &[ModuleEventRouteRequestV1],
        blob_requests: &[ModuleBlobQuotaRequestV1],
        scheduler_requests: &[ModuleSchedulerJobRequestV1],
        vault_purpose_requests: &[ModuleVaultPurposeRequestV1],
        client_rpc_routes: &[ModuleClientRpcRouteV1],
    ) -> Result<(), Self::Error> {
        SqliteControlStore::create_pending_registration_with_all_descriptor_requests(
            self, registration, capabilities, storage_requests, event_requests, blob_requests,
            scheduler_requests, vault_purpose_requests, client_rpc_routes,
        )
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
    fn approved_module_grant_snapshots(&self) -> Result<Vec<ModuleGrantSnapshot>, Self::Error> {
        SqliteControlStore::approved_module_grant_snapshots(self)
    }
    fn module_storage_request(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<ModuleStorageRequestV1>, Self::Error> {
        SqliteControlStore::module_storage_request(self, registration_id, capability_id)
    }
    fn module_event_route_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleEventRouteRequestV1>, Self::Error> {
        SqliteControlStore::module_event_route_requests(self, registration_id, capability_id)
    }
    fn approved_module_client_rpc_routes(&self) -> Result<Vec<ModuleClientRpcRouteV1>, Self::Error> {
        SqliteControlStore::approved_module_client_rpc_routes(self)
    }
    fn module_blob_quota_request(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<ModuleBlobQuotaRequestV1>, Self::Error> {
        SqliteControlStore::module_blob_quota_request(self, registration_id, capability_id)
    }
    fn module_scheduler_job_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleSchedulerJobRequestV1>, Self::Error> {
        SqliteControlStore::module_scheduler_job_requests(self, registration_id, capability_id)
    }
    fn module_vault_purpose_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleVaultPurposeRequestV1>, Self::Error> {
        SqliteControlStore::module_vault_purpose_requests(self, registration_id, capability_id)
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

impl EventsAuthorityStore for SqliteControlStore {
    type Error = StoreError;

    fn record_platform_events_authority_configuration(
        &self,
        configuration: &PlatformEventsAuthorityConfigurationV1,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_platform_events_authority_configuration(self, configuration)
    }

    fn platform_events_authority_configuration(
        &self,
    ) -> Result<Option<PlatformEventsAuthorityConfigurationV1>, Self::Error> {
        SqliteControlStore::platform_events_authority_configuration(self)
    }
}

impl EventHubTopologyStore for SqliteControlStore {
    type Error = StoreError;

    fn record_platform_event_hub_topology(
        &self,
        topology: &PlatformEventHubTopologyV1,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_platform_event_hub_topology(self, topology)
    }

    fn platform_event_hub_topology(
        &self,
    ) -> Result<Option<PlatformEventHubTopologyV1>, Self::Error> {
        SqliteControlStore::platform_event_hub_topology(self)
    }
}

impl StorageTopologyStore for SqliteControlStore {
    type Error = StoreError;

    fn record_platform_storage_topology(
        &self,
        topology: &PlatformStorageTopology,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_platform_storage_topology(self, topology)
    }

    fn platform_storage_topology(&self) -> Result<Option<PlatformStorageTopology>, Self::Error> {
        SqliteControlStore::platform_storage_topology(self)
    }
}

impl StorageBindingStore for SqliteControlStore {
    type Error = StoreError;

    fn record_platform_storage_binding(
        &self,
        binding: &hermes_kernel_control_store::PlatformStorageBindingV1,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_platform_storage_binding(self, binding)
    }

    fn platform_storage_binding(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<hermes_kernel_control_store::PlatformStorageBindingV1>, Self::Error> {
        SqliteControlStore::platform_storage_binding(self, registration_id, capability_id)
    }

    fn begin_platform_storage_binding_revocation(
        &self,
        registration_id: &str,
        capability_id: &str,
        binding_revision: u64,
    ) -> Result<hermes_kernel_control_store::PlatformStorageBindingV1, Self::Error> {
        SqliteControlStore::begin_platform_storage_binding_revocation(
            self,
            registration_id,
            capability_id,
            binding_revision,
        )
    }

    fn platform_storage_bindings(
        &self,
    ) -> Result<Vec<hermes_kernel_control_store::PlatformStorageBindingV1>, Self::Error> {
        SqliteControlStore::platform_storage_bindings(self)
    }
}

impl StorageBundleStore for SqliteControlStore {
    type Error = StoreError;

    fn record_platform_storage_bundle(
        &self,
        bundle: &hermes_kernel_control_store::PlatformStorageBundleV1,
    ) -> Result<(), Self::Error> {
        SqliteControlStore::record_platform_storage_bundle(self, bundle)
    }

    fn platform_storage_bundle(
        &self,
        owner_id: &str,
        revision: u64,
    ) -> Result<Option<hermes_kernel_control_store::PlatformStorageBundleV1>, Self::Error> {
        SqliteControlStore::platform_storage_bundle(self, owner_id, revision)
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
