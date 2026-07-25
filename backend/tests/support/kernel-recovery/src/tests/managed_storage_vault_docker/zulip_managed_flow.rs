//! Live managed Zulip launch through Kernel-owned admission and platform leases.

use super::*;

use hermes_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use hermes_zulip_api::{
    ZulipClientRequestV1, ZulipClientResponseV1, ZulipCommandV1,
    client_contract::ZulipClientContractV1,
};
use hermes_zulip_runtime::{
    admission::ZULIP_STORAGE_CAPABILITY_ID,
    client_port::{decode_module_response, encode_module_request},
};

use crate::identity::device::signer::DeviceSigner;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Zulip and NATS binaries"]
fn managed_zulip_runtime_uses_kernel_leases_and_route_specific_admission() {
    let contour = ManagedZulipContour::start(ZulipGrantProfileV1::QueryOnly);
    assert_zulip_query_is_admitted(&contour.store, &contour.supervisor, &contour.zulip);
    assert!(
        contour.fixture.accepted_connections() > 0,
        "managed Zulip runtime must reach the live loopback HTTPS fixture"
    );
    assert_ungranted_zulip_command_is_rejected(&contour.store, &contour.supervisor, &contour.zulip);
    assert_stale_zulip_query_generation_is_rejected(
        &contour.store,
        &contour.supervisor,
        &contour.zulip,
    );
    let (owner_runtime_dir, owner_control) = start_owner_control(
        &contour.data,
        &contour.store,
        &contour.shutdown,
        &contour.supervisor,
    );
    revoke_zulip_runtime(
        &owner_runtime_dir,
        &contour.owner_signer,
        &contour.store,
        &contour.supervisor,
        &contour.zulip,
    );

    contour.shutdown_processes();
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    contour.finish();
}

fn assert_zulip_query_is_admitted(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&zulip.registration_id) != Ok(true) {
        assert!(
            std::time::Instant::now() < deadline,
            "managed Zulip runtime did not become ready: {:?}",
            supervisor.last_failure(&zulip.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    let request = ZulipClientRequestV1::OperationStatus {
        operation_id: "unknown-operation".to_owned(),
    };
    let encoded = encode_module_request(11, &request).expect("encode Zulip query");
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &zulip.registration_id,
            &zulip.runtime_instance_id,
            zulip.runtime_generation,
            zulip.grant_epoch,
            ZulipClientContractV1::Query.capability_id(),
            &encoded,
        );
        let last_route = match route_managed_client_request(store, &relay, &route) {
            Ok(bytes) => match decode_module_response(ZulipClientContractV1::Query, &bytes) {
                Ok((11, ZulipClientResponseV1::OperationStatus(None))) => return,
                outcome => format!("unexpected response: {outcome:?}"),
            },
            Err(error) => format!("route error: {error:?}"),
        };
        assert!(
            std::time::Instant::now() < deadline,
            "managed Zulip query remained unavailable: {:?}; {last_route}",
            supervisor.last_failure(&zulip.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_ungranted_zulip_command_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let request = encode_module_request(
        12,
        &ZulipClientRequestV1::Command(ZulipCommandV1::SendStream {
            operation_id: "ungranted-zulip-command".to_owned(),
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
            stream: "operations".to_owned(),
            topic: "admission".to_owned(),
            content: "Kernel must reject this route before Zulip receives it".to_owned(),
        }),
    )
    .expect("encode ungranted Zulip command");
    let route = ManagedCapabilityRouteRequest::new(
        &zulip.registration_id,
        &zulip.runtime_instance_id,
        zulip.runtime_generation,
        zulip.grant_epoch,
        ZulipClientContractV1::Command.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("ungranted Zulip command route"),
        "capability is not granted to this registration"
    );
}

fn assert_stale_zulip_query_generation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let request = encode_module_request(
        13,
        &ZulipClientRequestV1::OperationStatus {
            operation_id: "stale-zulip-query".to_owned(),
        },
    )
    .expect("encode stale Zulip query");
    let route = ManagedCapabilityRouteRequest::new(
        &zulip.registration_id,
        &zulip.runtime_instance_id,
        zulip.runtime_generation + 1,
        zulip.grant_epoch,
        ZulipClientContractV1::Query.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("stale Zulip query generation"),
        "managed runtime fence is stale"
    );
}

fn revoke_zulip_runtime(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let revoked =
        transition_registration(owner_runtime_dir, signer, &zulip.registration_id, "revoked");
    assert_eq!(revoked.state, "revoked");
    assert!(
        revoked.grant_epoch > zulip.grant_epoch,
        "revoke advances the durable grant epoch before process stop"
    );
    let registration = store
        .module_registration(&zulip.registration_id)
        .expect("read revoked Zulip registration")
        .expect("revoked Zulip registration");
    assert_eq!(registration.state(), ModuleRegistrationState::Revoked);
    let binding = store
        .platform_storage_binding(&zulip.registration_id, ZULIP_STORAGE_CAPABILITY_ID)
        .expect("read revoked Zulip Storage binding")
        .expect("revoked Zulip Storage binding");
    assert_eq!(
        binding.state(),
        PlatformStorageBindingStateV1::Revoking,
        "owner transition durably reserves the exact Zulip Storage fence"
    );
    assert!(
        !supervisor
            .stop_if_active(&zulip.registration_id)
            .expect("observe stopped Zulip worker"),
        "owner transition already stopped the exact Zulip worker"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("observe Communications worker"),
        "Zulip revoke must not stop Communications"
    );
    assert_revoked_zulip_query_is_rejected(store, supervisor, zulip);
}

fn assert_revoked_zulip_query_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    zulip: &StartedZulipRuntime,
) {
    let request = encode_module_request(
        14,
        &ZulipClientRequestV1::OperationStatus {
            operation_id: "revoked-zulip-query".to_owned(),
        },
    )
    .expect("encode revoked Zulip query");
    let route = ManagedCapabilityRouteRequest::new(
        &zulip.registration_id,
        &zulip.runtime_instance_id,
        zulip.runtime_generation,
        zulip.grant_epoch,
        ZulipClientContractV1::Query.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("revoked Zulip query route"),
        "module registration is not approved"
    );
}
