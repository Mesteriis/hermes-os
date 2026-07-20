use crate::{
    GrantSet, ModuleBlobQuotaRequestV1, ModuleEventRouteRequestV1, ModuleGrantSnapshot,
    ModuleRegistration, ModuleRegistrationState, ModuleSchedulerJobRequestV1,
    ModuleStorageRequestV1,
};

pub trait ModuleRegistryStore {
    type Error;

    fn create_pending_registration(
        &self,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
    ) -> Result<(), Self::Error>;
    fn create_pending_registration_with_requests(
        &self,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
        storage_requests: &[ModuleStorageRequestV1],
        event_requests: &[ModuleEventRouteRequestV1],
        blob_requests: &[ModuleBlobQuotaRequestV1],
    ) -> Result<(), Self::Error>;
    fn create_pending_registration_with_descriptor_requests(
        &self,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
        storage_requests: &[ModuleStorageRequestV1],
        event_requests: &[ModuleEventRouteRequestV1],
        blob_requests: &[ModuleBlobQuotaRequestV1],
        scheduler_requests: &[ModuleSchedulerJobRequestV1],
    ) -> Result<(), Self::Error>;
    fn module_registration(
        &self,
        registration_id: &str,
    ) -> Result<Option<ModuleRegistration>, Self::Error>;
    fn transition_module_registration(
        &self,
        registration_id: &str,
        next: ModuleRegistrationState,
    ) -> Result<ModuleRegistration, Self::Error>;
    fn approve_module_registration(
        &self,
        registration_id: &str,
        capability_ids: &[String],
    ) -> Result<GrantSet, Self::Error>;
    fn module_grant_snapshot(
        &self,
        registration_id: &str,
    ) -> Result<Option<ModuleGrantSnapshot>, Self::Error>;
    fn approved_module_grant_snapshots(&self) -> Result<Vec<ModuleGrantSnapshot>, Self::Error>;
    fn module_storage_request(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<ModuleStorageRequestV1>, Self::Error>;
    fn module_event_route_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleEventRouteRequestV1>, Self::Error>;
    fn module_blob_quota_request(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<ModuleBlobQuotaRequestV1>, Self::Error>;
    fn module_scheduler_job_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleSchedulerJobRequestV1>, Self::Error>;
}
