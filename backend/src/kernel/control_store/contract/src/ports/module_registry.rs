use crate::{GrantSet, ModuleGrantSnapshot, ModuleRegistration, ModuleRegistrationState};

pub trait ModuleRegistryStore {
    type Error;

    fn create_pending_registration(
        &self,
        registration: &ModuleRegistration,
        requested_capability_ids: &[String],
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
}
