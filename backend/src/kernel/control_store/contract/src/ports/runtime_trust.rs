use crate::{
    BundledManagedLaunchBinding, ExternalRuntimeAttestation, ExternalRuntimeIdentity,
    ManagedLaunchRecord, ModuleRegistration, OwnerPinnedArtifactBinding,
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch,
};

pub trait RuntimeTrustStore {
    type Error;

    fn attest_external_runtime(
        &self,
        attestation: &ExternalRuntimeAttestation,
    ) -> Result<(), Self::Error>;
    fn effective_external_runtime_attestation(
        &self,
        registration_id: &str,
    ) -> Result<Option<ExternalRuntimeAttestation>, Self::Error>;
    fn bind_external_runtime_identity(
        &self,
        identity: &ExternalRuntimeIdentity,
    ) -> Result<ModuleRegistration, Self::Error>;
    fn external_runtime_identity(
        &self,
        registration_id: &str,
    ) -> Result<Option<ExternalRuntimeIdentity>, Self::Error>;
    fn record_owner_pinned_artifact_binding(
        &self,
        binding: &OwnerPinnedArtifactBinding,
    ) -> Result<(), Self::Error>;
    fn effective_owner_pinned_artifact_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<OwnerPinnedArtifactBinding>, Self::Error>;
    fn record_bundled_managed_launch_binding(
        &self,
        binding: &BundledManagedLaunchBinding,
    ) -> Result<(), Self::Error>;
    fn effective_bundled_managed_launch_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<BundledManagedLaunchBinding>, Self::Error>;
    fn record_managed_launch(&self, record: &ManagedLaunchRecord) -> Result<(), Self::Error>;
    fn effective_managed_launch_record(
        &self,
        registration_id: &str,
    ) -> Result<Option<ManagedLaunchRecord>, Self::Error>;
    fn record_platform_managed_process_binding(
        &self,
        binding: &PlatformManagedProcessBinding,
    ) -> Result<(), Self::Error>;
    fn platform_managed_process_binding(
        &self,
        process_id: &str,
    ) -> Result<Option<PlatformManagedProcessBinding>, Self::Error>;
    fn record_platform_managed_process_launch(
        &self,
        launch: &PlatformManagedProcessLaunch,
    ) -> Result<(), Self::Error>;
    fn platform_managed_process_launch(
        &self,
        process_id: &str,
    ) -> Result<Option<PlatformManagedProcessLaunch>, Self::Error>;
}
