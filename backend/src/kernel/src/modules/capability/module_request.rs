//! Authorizes and relays one exact descriptor-declared managed module request.

use std::sync::Arc;
use std::time::Duration;

use hermes_kernel_control_store::ModuleRequestContractV1;
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::{
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestRequestV1,
        ManagedRuntimeModuleRequestResponseV1, managed_runtime_control_request_v1,
        managed_runtime_control_response_v1,
    },
    validation::module_request::{
        validate_module_request_delivery_v1, validate_module_request_request_v1,
        validate_module_request_response_v1,
    },
};
use prost::Message;

use crate::runtime::lifecycle::{
    control::{ManagedRuntimeExpectation, ManagedRuntimeModuleRequestHandler},
    fence::current_managed_runtime_matches,
    supervisor::ManagedRuntimeRelay,
};

pub(crate) struct ModuleRequestRouteHandlerV1<R> {
    store: Arc<SqliteControlStore>,
    relay: R,
}

impl<R> ModuleRequestRouteHandlerV1<R>
where
    R: ManagedRuntimeRelay,
{
    pub(crate) fn new(store: Arc<SqliteControlStore>, relay: R) -> Self {
        Self { store, relay }
    }
}

impl<R> ManagedRuntimeModuleRequestHandler for ModuleRequestRouteHandlerV1<R>
where
    R: ManagedRuntimeRelay,
{
    fn route_module_request(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeModuleRequestRequestV1,
    ) -> Result<ManagedRuntimeModuleRequestResponseV1, String> {
        validate_module_request_request_v1(&request)
            .map_err(|_| "managed module request is denied".to_owned())?;
        ensure_caller_fence(&self.store, expectation)?;

        let caller = self
            .store
            .module_registration(expectation.registration_id())
            .map_err(|_| "managed module request caller is unavailable".to_owned())?
            .ok_or_else(|| "managed module request caller is unavailable".to_owned())?;
        if caller.grant_epoch() != expectation.grant_epoch() {
            return Err("managed module request caller fence is stale".to_owned());
        }
        let logical_owner = self
            .store
            .initial_owner_identity()
            .map_err(|_| "managed module request logical owner is unavailable".to_owned())?
            .ok_or_else(|| "managed module request logical owner is unavailable".to_owned())?;
        let grants = self
            .store
            .module_grant_snapshot(expectation.registration_id())
            .map_err(|_| "managed module request caller grants are unavailable".to_owned())?
            .and_then(|snapshot| snapshot.effective_grants().cloned())
            .ok_or_else(|| "managed module request caller is not approved".to_owned())?;
        let contract = request
            .contract
            .as_ref()
            .ok_or_else(|| "managed module request contract is missing".to_owned())?;
        resolve_caller_capability(
            &self.store,
            expectation.registration_id(),
            grants.capability_ids(),
            contract,
        )?;

        let provider = resolve_provider(&self.store, contract)?;
        let provider_grants = self
            .store
            .module_grant_snapshot(provider.registration_id())
            .map_err(|_| "managed module request provider grants are unavailable".to_owned())?
            .and_then(|snapshot| snapshot.effective_grants().cloned())
            .ok_or_else(|| "managed module request provider is not approved".to_owned())?;
        if provider_grants
            .capability_ids()
            .binary_search_by(|candidate| candidate.as_str().cmp(provider.capability_id()))
            .is_err()
        {
            return Err("managed module request provider capability is not granted".to_owned());
        }
        let provider_launch = current_provider_launch(&self.store, &provider)?;

        let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
            request_id: request.request_id.clone(),
            logical_owner_id: logical_owner.owner_id().to_owned(),
            contract: request.contract.clone(),
            request_payload: request.request_payload,
        };
        validate_module_request_delivery_v1(&delivery)
            .map_err(|_| "managed module request delivery is denied".to_owned())?;
        let response_bytes = ManagedRuntimeRelay::relay_with_timeout(
            &self.relay,
            provider.registration_id(),
            ManagedRuntimeControlRequestV1 {
                operation: Some(
                    managed_runtime_control_request_v1::Operation::DeliverModuleRequest(delivery),
                ),
            }
            .encode_to_vec(),
            Duration::from_millis(u64::from(request.deadline_millis)),
        )?;
        let response = ManagedRuntimeControlResponseV1::decode(response_bytes.as_slice())
            .map_err(|_| "managed module request provider response is invalid".to_owned())?
            .result
            .and_then(|result| match result {
                managed_runtime_control_response_v1::Result::ModuleRequestDelivery(response) => {
                    Some(response)
                }
                _ => None,
            })
            .ok_or_else(|| "managed module request provider response is missing".to_owned())?;
        validate_module_request_response_v1(&response)
            .map_err(|_| "managed module request provider response is rejected".to_owned())?;
        if response.request_id != request.request_id {
            return Err(
                "managed module request provider response does not match request".to_owned(),
            );
        }

        ensure_caller_fence(&self.store, expectation)?;
        ensure_provider_fence(&self.store, &provider, &provider_launch)?;
        Ok(response)
    }
}

fn resolve_caller_capability(
    store: &SqliteControlStore,
    registration_id: &str,
    granted_capabilities: &[String],
    contract: &hermes_runtime_protocol::v1::ContractReferenceV1,
) -> Result<(), String> {
    for capability_id in granted_capabilities {
        let dependencies = store
            .module_contract_dependencies(registration_id, capability_id)
            .map_err(|_| "managed module request dependencies are unavailable".to_owned())?;
        if dependencies
            .iter()
            .any(|dependency| exact_dependency_matches(dependency, contract))
        {
            return Ok(());
        }
    }
    Err("managed module request dependency is not granted".to_owned())
}

fn resolve_provider(
    store: &SqliteControlStore,
    contract: &hermes_runtime_protocol::v1::ContractReferenceV1,
) -> Result<ModuleRequestContractV1, String> {
    let routes = store
        .approved_module_request_rpc_routes()
        .map_err(|_| "managed module request providers are unavailable".to_owned())?;
    let mut matches = routes
        .into_iter()
        .filter(|route| exact_provider_matches(route, contract));
    let provider = matches
        .next()
        .ok_or_else(|| "managed module request provider is unavailable".to_owned())?;
    if matches.next().is_some() {
        return Err("managed module request provider is ambiguous".to_owned());
    }
    Ok(provider)
}

fn exact_dependency_matches(
    expected: &hermes_kernel_control_store::ModuleQueryContractV1,
    actual: &hermes_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    expected.owner() == actual.owner
        && expected.name() == actual.name
        && expected.major() == actual.major
        && expected.revision() == actual.revision
        && expected.schema_sha256().as_slice() == actual.schema_sha256
}

fn exact_provider_matches(
    expected: &ModuleRequestContractV1,
    actual: &hermes_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    expected.owner() == actual.owner
        && expected.name() == actual.name
        && expected.major() == actual.major
        && expected.revision() == actual.revision
        && expected.schema_sha256().as_slice() == actual.schema_sha256
}

fn ensure_caller_fence(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
) -> Result<(), String> {
    current_managed_runtime_matches(
        store,
        expectation.registration_id(),
        expectation.runtime_instance_id(),
        expectation.runtime_generation(),
        expectation.grant_epoch(),
    )
    .map_err(|_| "managed module request caller is unavailable".to_owned())?
    .then_some(())
    .ok_or_else(|| "managed module request caller fence is stale".to_owned())
}

fn current_provider_launch(
    store: &SqliteControlStore,
    provider: &ModuleRequestContractV1,
) -> Result<hermes_kernel_control_store::ManagedLaunchRecord, String> {
    let launch = store
        .effective_managed_launch_record(provider.registration_id())
        .map_err(|_| "managed module request provider is unavailable".to_owned())?
        .ok_or_else(|| "managed module request provider is unavailable".to_owned())?;
    ensure_provider_fence(store, provider, &launch)?;
    Ok(launch)
}

fn ensure_provider_fence(
    store: &SqliteControlStore,
    provider: &ModuleRequestContractV1,
    launch: &hermes_kernel_control_store::ManagedLaunchRecord,
) -> Result<(), String> {
    current_managed_runtime_matches(
        store,
        provider.registration_id(),
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
    )
    .map_err(|_| "managed module request provider is unavailable".to_owned())?
    .then_some(())
    .ok_or_else(|| "managed module request provider fence is stale".to_owned())
}
