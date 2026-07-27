use crate::{
    BundledManagedArtifactProposalInputV1, BundledManagedArtifactProposalReceiptV1,
    ModuleDescriptorRegistrationRequestsV1, ModuleRegistration,
};

/// Atomic persistence boundary for proposing a verified installed module artifact.
pub trait BundledArtifactProposalStore {
    type Error;

    fn propose_bundled_managed_artifact(
        &self,
        proposal: &BundledManagedArtifactProposalInputV1,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
        requests: ModuleDescriptorRegistrationRequestsV1<'_>,
    ) -> Result<BundledManagedArtifactProposalReceiptV1, Self::Error>;
}
