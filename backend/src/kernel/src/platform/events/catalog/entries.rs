//! Resolves Event Hub route entries from approved Control Store state.

use hermes_kernel_control_store::{
    ModuleEventRouteRequestV1, ModuleGrantSnapshot, ModuleRegistryStore,
};
use hermes_kernel_control_store_sqlite::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCatalogEntryV1 {
    registration_id: String,
    module_id: String,
    grant_epoch: u64,
    capability_id: String,
    route: ModuleEventRouteRequestV1,
}

impl EventCatalogEntryV1 {
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn module_id(&self) -> &str {
        &self.module_id
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub fn route(&self) -> &ModuleEventRouteRequestV1 {
        &self.route
    }
}

pub fn resolve<S>(store: &S) -> Result<Vec<EventCatalogEntryV1>, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    store
        .approved_module_grant_snapshots()
        .map_err(|error| format!("{error:?}"))?
        .iter()
        .map(|snapshot| resolve_snapshot(store, snapshot))
        .collect::<Result<Vec<_>, _>>()
        .map(|entries| entries.into_iter().flatten().collect())
}

fn resolve_snapshot<S>(
    store: &S,
    snapshot: &ModuleGrantSnapshot,
) -> Result<Vec<EventCatalogEntryV1>, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    let registration = snapshot.registration();
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "approved module snapshot lacks effective grants".to_owned())?;
    let mut entries = Vec::new();
    for capability_id in grants.capability_ids() {
        let routes = store
            .module_event_route_requests(registration.registration_id(), capability_id)
            .map_err(|error| format!("{error:?}"))?;
        entries.extend(routes.into_iter().map(|route| EventCatalogEntryV1 {
            registration_id: registration.registration_id().to_owned(),
            module_id: registration.module_id().to_owned(),
            grant_epoch: grants.grant_epoch(),
            capability_id: capability_id.clone(),
            route,
        }));
    }
    Ok(entries)
}
