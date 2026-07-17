//! Authorizes an external runtime capability route against current fenced state.

use hermes_kernel_control_store::{ModuleRegistryStore, RuntimeTrustStore};
use hermes_kernel_control_store_sqlite::StoreError;

use crate::modules::capability::policy::permits_external_route;

pub struct ExternalCapabilityRouteRequest<'a> {
    registration_id: &'a str,
    runtime_id: &'a str,
    runtime_generation: u64,
    capability_id: &'a str,
}

impl<'a> ExternalCapabilityRouteRequest<'a> {
    #[must_use]
    pub fn new(
        registration_id: &'a str,
        runtime_id: &'a str,
        runtime_generation: u64,
        capability_id: &'a str,
    ) -> Self {
        Self {
            registration_id,
            runtime_id,
            runtime_generation,
            capability_id,
        }
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        self.registration_id
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        self.runtime_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
}

pub struct AuthorizedExternalCapabilityRoute {
    grant_epoch: u64,
}

impl AuthorizedExternalCapabilityRoute {
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

pub fn authorize_external_route<S>(
    store: &S,
    request: &ExternalCapabilityRouteRequest<'_>,
) -> Result<AuthorizedExternalCapabilityRoute, String>
where
    S: ModuleRegistryStore<Error = StoreError> + RuntimeTrustStore<Error = StoreError>,
{
    if !permits_external_route(request.capability_id) {
        return Err("capability route is prohibited by Kernel policy".to_owned());
    }
    let snapshot = store
        .module_grant_snapshot(request.registration_id)
        .map_err(|_| "module registration is unavailable".to_owned())?
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    if grants
        .capability_ids()
        .binary_search_by(|candidate| candidate.as_str().cmp(request.capability_id))
        .is_err()
    {
        return Err("capability is not granted to this registration".to_owned());
    }
    let runtime = store
        .effective_external_runtime_attestation(request.registration_id)
        .map_err(|_| "external runtime is unavailable".to_owned())?
        .ok_or_else(|| "external runtime requires a current attestation".to_owned())?;
    if runtime.runtime_id() != request.runtime_id
        || runtime.runtime_generation() != request.runtime_generation
        || runtime.grant_epoch() != grants.grant_epoch()
    {
        return Err("external runtime attestation is stale".to_owned());
    }
    Ok(AuthorizedExternalCapabilityRoute {
        grant_epoch: grants.grant_epoch(),
    })
}
