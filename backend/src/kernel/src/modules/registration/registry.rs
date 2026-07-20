//! Module Registry mutations and read model shared by local control transports.

use hermes_kernel_control_store::{
    ExternalRuntimeAttestation, ExternalRuntimeIdentity, GrantSet, HealthRecoveryStore,
    ModuleRegistration, ModuleRegistrationState, ModuleRegistryStore, OwnerIdentityStore,
    RuntimeTrustStore,
};
use hermes_kernel_control_store_sqlite::StoreError;

use crate::identity::owner::authorization::{authorize as authorize_file_owner, operation_digest};
use crate::infrastructure::filesystem::new_instance_id;
use crate::modules::capability::policy::permits_external_route;
use p256::ecdsa::VerifyingKey;

use super::descriptor::DescriptorRegistrationRequests;

pub struct ModuleRegistryStatus {
    registration: ModuleRegistration,
    effective_capability_count: usize,
    external_runtime_attestation: Option<ExternalRuntimeAttestation>,
}

impl ModuleRegistryStatus {
    #[must_use]
    pub fn registration(&self) -> &ModuleRegistration {
        &self.registration
    }
    #[must_use]
    pub fn effective_capability_count(&self) -> usize {
        self.effective_capability_count
    }
    #[must_use]
    pub fn external_runtime_attestation(&self) -> Option<&ExternalRuntimeAttestation> {
        self.external_runtime_attestation.as_ref()
    }
}

pub fn register<S>(store: &S, descriptor_bytes: &[u8]) -> Result<ModuleRegistration, String>
where
    S: ModuleRegistryStore<Error = StoreError> + OwnerIdentityStore<Error = StoreError>,
{
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err("module registration requires an enrolled initial owner".to_owned());
    }
    let requests = DescriptorRegistrationRequests::decode(descriptor_bytes)?;
    persist_registration(store, requests)
}

fn persist_registration<S>(
    store: &S,
    requests: DescriptorRegistrationRequests,
) -> Result<ModuleRegistration, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    for _ in 0..16 {
        let registration = ModuleRegistration::new(
            new_instance_id()?,
            requests.module_id(),
            requests.owner_id(),
            requests.descriptor_sha256(),
            ModuleRegistrationState::Pending,
            1,
        );
        let bound = requests.bind(&registration);
        match store.create_pending_registration_with_descriptor_requests(
            &registration,
            requests.capability_ids(),
            &bound.storage,
            &bound.events,
            &bound.blobs,
            &bound.scheduler,
        ) {
            Ok(()) => return Ok(registration),
            Err(
                hermes_kernel_control_store_sqlite::StoreError::ModuleRegistrationAlreadyExists,
            ) => {}
            Err(error) => return Err(format!("{error:?}")),
        }
    }
    Err("unable to allocate a unique module registration ID".to_owned())
}

pub fn approve<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<GrantSet, String>
where
    S: HealthRecoveryStore
        + ModuleRegistryStore<Error = StoreError>
        + OwnerIdentityStore<Error = StoreError>,
{
    let mut authorization_fields = Vec::with_capacity(capability_ids.len() + 1);
    authorization_fields.push(registration_id);
    authorization_fields.extend(capability_ids.iter().map(String::as_str));
    authorize_file_owner(
        data_dir,
        store,
        "module.approve.v1",
        operation_digest(&authorization_fields)?,
    )?;
    approve_after_owner_authorization(store, registration_id, capability_ids)
}

pub fn approve_after_owner_authorization<S>(
    store: &S,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<GrantSet, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    if capability_ids
        .iter()
        .any(|capability_id| !permits_external_route(capability_id))
    {
        return Err("capability grant is prohibited by Kernel policy".to_owned());
    }
    store
        .approve_module_registration(registration_id, capability_ids)
        .map_err(|error| format!("{error:?}"))
}

pub fn transition<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    next: ModuleRegistrationState,
) -> Result<ModuleRegistration, String>
where
    S: HealthRecoveryStore
        + ModuleRegistryStore<Error = StoreError>
        + OwnerIdentityStore<Error = StoreError>,
{
    authorize_file_owner(
        data_dir,
        store,
        "module.transition.v1",
        operation_digest(&[registration_id, next.as_str()])?,
    )?;
    transition_after_owner_authorization(store, registration_id, next)
}

pub fn transition_after_owner_authorization<S>(
    store: &S,
    registration_id: &str,
    next: ModuleRegistrationState,
) -> Result<ModuleRegistration, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    store
        .transition_module_registration(registration_id, next)
        .map_err(|error| format!("{error:?}"))
}

pub fn bind_external_runtime_identity<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    public_key_sec1: [u8; 65],
) -> Result<ModuleRegistration, String>
where
    S: HealthRecoveryStore
        + OwnerIdentityStore<Error = StoreError>
        + RuntimeTrustStore<Error = StoreError>,
{
    VerifyingKey::from_sec1_bytes(&public_key_sec1)
        .map_err(|_| "external runtime public key is invalid".to_owned())?;
    let public_key_hex = public_key_sec1
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    authorize_file_owner(
        data_dir,
        store,
        "module.bind_external_runtime_identity.v1",
        operation_digest(&[registration_id, &public_key_hex])?,
    )?;
    bind_external_runtime_identity_after_owner_authorization(
        store,
        registration_id,
        public_key_sec1,
    )
}

pub fn bind_external_runtime_identity_after_owner_authorization<S>(
    store: &S,
    registration_id: &str,
    public_key_sec1: [u8; 65],
) -> Result<ModuleRegistration, String>
where
    S: RuntimeTrustStore<Error = StoreError>,
{
    VerifyingKey::from_sec1_bytes(&public_key_sec1)
        .map_err(|_| "external runtime public key is invalid".to_owned())?;
    store
        .bind_external_runtime_identity(&ExternalRuntimeIdentity::new(
            registration_id,
            public_key_sec1,
        ))
        .map_err(|error| format!("{error:?}"))
}

pub fn status<S>(store: &S, registration_id: &str) -> Result<ModuleRegistryStatus, String>
where
    S: ModuleRegistryStore<Error = StoreError> + RuntimeTrustStore<Error = StoreError>,
{
    let snapshot = store
        .module_grant_snapshot(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "module registration does not exist".to_owned())?;
    let effective_capability_count = snapshot
        .effective_grants()
        .map_or(0, |grants| grants.capability_ids().len());
    let (registration, _) = snapshot.into_parts();
    let external_runtime_attestation = store
        .effective_external_runtime_attestation(registration_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(ModuleRegistryStatus {
        registration,
        effective_capability_count,
        external_runtime_attestation,
    })
}
