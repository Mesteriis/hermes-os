//! Live co-admission of Scheduler and the two independent communication workflows.

use super::*;

use std::sync::Mutex;
use std::time::Instant;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hermes_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_CAPABILITY_ID_V1, COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1,
    COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
    COMMUNICATION_DELAYED_DELIVERY_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
    COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1,
    wire::{
        DelayedDeliveryErrorCodeV1, DelayedDeliveryStateV1, DelayedDeliveryStatusChangedV1,
        GetDelayedDeliveryStatusRequestV1, GetDelayedDeliveryStatusResponseV1,
        ScheduleDelayedDeliveryRequestV1, ScheduleDelayedDeliveryResponseV1,
    },
};
use hermes_communication_delayed_delivery_runtime::COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1;
use hermes_communication_delayed_delivery_runtime::delayed_delivery_query_contract_v1;
use hermes_communication_delivery_intent_api::wire::SubmitDeliveryIntentRequestV1;
use hermes_gateway_protocol::v1::{
    ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use hermes_kernel_control_store::PlatformStorageBindingStateV1;
use hermes_runtime_protocol::v1::{
    ManagedRuntimeModuleRequestRequestV1, ManagedRuntimeModuleRequestResponseV1,
    ModuleClientRequestV1,
};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};

use crate::identity::device::signer::DeviceSigner;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeExpectation, ManagedRuntimeModuleRequestHandler,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Scheduler, Communications, delayed-delivery and delivery-intent binaries"]
fn managed_delayed_delivery_starts_with_scheduler_and_delivery_intent() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-delayed-delivery");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_delayed_delivery_conformance_release(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            DELAYED_DELIVERY_LOGICAL_OWNER_ID,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim logical browser owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        DELAYED_DELIVERY_LOGICAL_OWNER_ID,
    );
    record_scheduler_runtime(&store);
    let delivery_intent = admit_delivery_intent_runtime(&store);
    let delayed_delivery = admit_delayed_delivery_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        hermes_gateway_runtime::InMemoryBrowserRealtimeSource::new(32).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    let ambiguous_request_probe = Arc::new(AmbiguousDeliveryIntentRequestProbe::new([0xc2; 16]));
    configure_delivery_intent_runtime_routes_with_request_handler(
        &supervisor,
        &store,
        realtime.clone(),
        Arc::new(AmbiguousDeliveryIntentRequestHandler {
            inner: delivery_intent_request_route_handler(&supervisor, &store),
            probe: Arc::clone(&ambiguous_request_probe),
        }),
    );
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    blob_launch::start_from_kernel(
        &supervisor,
        &store,
        release.kernel(),
        &data,
        &root.join("runtime"),
    )
    .expect("start signed Blob runtime");
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    issue_initial_scheduler_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &scheduler_binding(&store),
    )
    .expect("provision Scheduler Storage binding");
    let delivery_intent = prepare_delivery_intent_runtime(&supervisor, &store, delivery_intent);
    let delayed_delivery = prepare_delayed_delivery_runtime(&supervisor, &store, delayed_delivery);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    let conversation_id = super::delivery_intent_realtime_flow::canonical_conversation_for_message(
        &store,
        &supervisor,
        &message_id,
    );
    let delivery_intent =
        start_delivery_intent_runtime(&supervisor, &store, &root.join("runtime"), delivery_intent);
    let delayed_delivery = start_delayed_delivery_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        delayed_delivery,
    );
    let scheduler_reservation = managed_launch::load(&supervisor, &store, SCHEDULER_REGISTRATION)
        .expect("load Scheduler reservation");
    assert_eq!(
        scheduler_launch::start_from_reservation(
            &supervisor,
            &store,
            release.kernel(),
            &root.join("runtime"),
            scheduler_reservation,
            &scheduler_binding(&store),
        )
        .expect("start Scheduler with delayed-delivery grant"),
        1
    );
    for registration_id in [
        SCHEDULER_REGISTRATION,
        COMMUNICATIONS_REGISTRATION,
        delivery_intent.registration_id.as_str(),
        delayed_delivery.registration_id.as_str(),
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .expect("read managed runtime state"),
            "{registration_id} must stay active in the combined contour"
        );
    }
    assert_eq!(delivery_intent.runtime_generation, 1);
    assert_eq!(delayed_delivery.runtime_generation, 1);
    let managed_contour = DelayedDeliveryManagedContour {
        store: &store,
        supervisor: &supervisor,
        root: &root,
        data: &data,
        kernel: release.kernel(),
    };
    assert_delayed_delivery_fails_closed_during_blob_outage(
        &managed_contour,
        &delayed_delivery.registration_id,
        realtime.clone(),
        conversation_id.clone(),
        0xb1,
        1,
    );
    assert_delayed_delivery_round_trip(
        &managed_contour,
        &delayed_delivery.registration_id,
        realtime.clone(),
        conversation_id.clone(),
        0x71,
        2,
    );
    let predecessor = delayed_delivery;
    let stale_runtime_instance_id = predecessor.runtime_instance_id.clone();
    let stale_runtime_generation = predecessor.runtime_generation;
    let stale_grant_epoch = predecessor.grant_epoch;
    let delayed_delivery =
        restart_delayed_delivery_runtime(&supervisor, &store, &root.join("runtime"), predecessor);
    restart_scheduler_with_current_grants(
        &supervisor,
        &store,
        release.kernel(),
        &root.join("runtime"),
        2,
    );
    assert_stale_delayed_route_is_rejected(
        &store,
        &supervisor,
        &delayed_delivery.registration_id,
        &stale_runtime_instance_id,
        stale_runtime_generation,
        stale_grant_epoch,
    );
    assert_delayed_delivery_round_trip(
        &managed_contour,
        &delayed_delivery.registration_id,
        realtime.clone(),
        conversation_id.clone(),
        0x81,
        3,
    );
    assert_delayed_delivery_survives_ambiguous_delivery_response(
        &managed_contour,
        &delayed_delivery.registration_id,
        realtime.clone(),
        conversation_id.clone(),
        0xc1,
        4,
        &ambiguous_request_probe,
    );
    let delayed_delivery = assert_delayed_delivery_survives_nats_outage(
        &managed_contour,
        delayed_delivery,
        realtime.clone(),
        conversation_id.clone(),
        0x91,
        5,
    );
    assert_delayed_delivery_survives_scheduler_outage(
        &managed_contour,
        &delayed_delivery.registration_id,
        realtime,
        conversation_id,
        0xa1,
        6,
    );
    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    revoke_delayed_delivery_runtime(
        &owner_runtime_dir,
        &owner_signer,
        &store,
        &supervisor,
        &delayed_delivery,
        &delivery_intent.registration_id,
    );
    restart_scheduler_with_current_grants(
        &supervisor,
        &store,
        release.kernel(),
        &root.join("runtime"),
        4,
    );
    assert!(
        supervisor
            .is_active(SCHEDULER_REGISTRATION)
            .expect("observe reconciled Scheduler after delayed-delivery revoke")
    );

    supervisor.shutdown().expect("stop managed processes");
    owner_control
        .join()
        .expect("join owner control")
        .expect("owner control exits");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove kernel fixture");
}

struct DelayedDeliveryManagedContour<'a> {
    store: &'a Arc<SqliteControlStore>,
    supervisor: &'a ManagedRuntimeSupervisor,
    root: &'a Path,
    data: &'a Path,
    kernel: &'a Path,
}

struct AmbiguousDeliveryIntentRequestProbe {
    target_operation_id: [u8; 16],
    successful_request_payloads: Mutex<Vec<Vec<u8>>>,
}

impl AmbiguousDeliveryIntentRequestProbe {
    fn new(target_operation_id: [u8; 16]) -> Self {
        Self {
            target_operation_id,
            successful_request_payloads: Mutex::new(Vec::new()),
        }
    }

    fn records_target(&self, request_payload: &[u8]) -> bool {
        SubmitDeliveryIntentRequestV1::decode(request_payload)
            .is_ok_and(|request| request.operation_id == self.target_operation_id)
    }

    fn record_successful_request(&self, request_payload: Vec<u8>) -> bool {
        let mut requests = self
            .successful_request_payloads
            .lock()
            .expect("lock ambiguous delivery-intent request probe");
        requests.push(request_payload);
        requests.len() == 1
    }

    fn successful_request_payloads(&self) -> Vec<Vec<u8>> {
        self.successful_request_payloads
            .lock()
            .expect("read ambiguous delivery-intent request probe")
            .clone()
    }
}

struct AmbiguousDeliveryIntentRequestHandler {
    inner: Arc<dyn ManagedRuntimeModuleRequestHandler>,
    probe: Arc<AmbiguousDeliveryIntentRequestProbe>,
}

impl ManagedRuntimeModuleRequestHandler for AmbiguousDeliveryIntentRequestHandler {
    fn route_module_request(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeModuleRequestRequestV1,
    ) -> Result<ManagedRuntimeModuleRequestResponseV1, String> {
        let target_request = self.probe.records_target(&request.request_payload);
        let exact_request_payload = request.request_payload.clone();
        let response = self.inner.route_module_request(expectation, request)?;
        if target_request && self.probe.record_successful_request(exact_request_payload) {
            return Err("simulated managed request response loss".to_owned());
        }
        Ok(response)
    }
}

fn assert_delayed_delivery_round_trip(
    contour: &DelayedDeliveryManagedContour<'_>,
    delayed_delivery_registration_id: &str,
    realtime: hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
    conversation_id: Vec<u8>,
    operation_byte: u8,
    authentication_sign_count: u32,
) {
    let DelayedDeliveryManagedContour {
        store,
        supervisor,
        root,
        data,
        ..
    } = contour;
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        store, supervisor, root, data, realtime,
    );
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
        &router,
        &runtime,
        authentication_sign_count,
    );
    let delayed_operation_id = vec![operation_byte; 16];
    let private_body = b"delayed private body must not enter durable events or realtime";
    let deliver_at_unix_millis =
        u64::try_from(current_unix_millis()).expect("positive current time") + 7_000;
    let response = route_proto(
        &router,
        &runtime,
        &cookie,
        COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
        ScheduleDelayedDeliveryRequestV1 {
            protocol_major: 1,
            delayed_operation_id: delayed_operation_id.clone(),
            delivery_operation_id: vec![operation_byte.wrapping_add(1); 16],
            conversation_id,
            reply_to_message_id: None,
            body_utf8: private_body.to_vec(),
            deliver_at_unix_millis,
        }
        .encode_to_vec(),
    );
    let response =
        ScheduleDelayedDeliveryResponseV1::decode(response.as_slice()).expect("schedule response");
    assert_eq!(response.delayed_operation_id, delayed_operation_id);
    assert_eq!(
        response.error,
        DelayedDeliveryErrorCodeV1::DelayedDeliveryErrorCodeUnspecified as i32
    );
    assert_eq!(
        response.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateSchedulePending as i32
    );
    let terminal = wait_for_terminal_status(
        &router,
        &runtime,
        &cookie,
        &delayed_operation_id,
        deliver_at_unix_millis,
        supervisor,
        delayed_delivery_registration_id,
    );
    assert_eq!(
        terminal.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateDeliveryAccepted as i32
    );
    let event =
        read_delayed_delivery_terminal_sse(&router, &runtime, &cookie, &delayed_operation_id);
    assert!(
        !event
            .encode_to_vec()
            .windows(private_body.len())
            .any(|window| window == private_body),
        "delayed-delivery realtime must not contain private content"
    );
}

#[allow(clippy::too_many_arguments)]
fn assert_delayed_delivery_survives_ambiguous_delivery_response(
    contour: &DelayedDeliveryManagedContour<'_>,
    delayed_delivery_registration_id: &str,
    realtime: hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
    conversation_id: Vec<u8>,
    operation_byte: u8,
    authentication_sign_count: u32,
    probe: &AmbiguousDeliveryIntentRequestProbe,
) {
    let DelayedDeliveryManagedContour {
        store,
        supervisor,
        root,
        data,
        ..
    } = contour;
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        store, supervisor, root, data, realtime,
    );
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
        &router,
        &runtime,
        authentication_sign_count,
    );
    let delayed_operation_id = vec![operation_byte; 16];
    let delivery_operation_id = vec![operation_byte.wrapping_add(1); 16];
    let private_body = b"delayed private body retained through ambiguous request outcome";
    let deliver_at_unix_millis =
        u64::try_from(current_unix_millis()).expect("positive current time") + 7_000;
    let response = route_proto(
        &router,
        &runtime,
        &cookie,
        COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
        ScheduleDelayedDeliveryRequestV1 {
            protocol_major: 1,
            delayed_operation_id: delayed_operation_id.clone(),
            delivery_operation_id: delivery_operation_id.clone(),
            conversation_id,
            reply_to_message_id: None,
            body_utf8: private_body.to_vec(),
            deliver_at_unix_millis,
        }
        .encode_to_vec(),
    );
    let response =
        ScheduleDelayedDeliveryResponseV1::decode(response.as_slice()).expect("schedule response");
    assert_eq!(response.delayed_operation_id, delayed_operation_id);
    assert_eq!(
        response.error,
        DelayedDeliveryErrorCodeV1::DelayedDeliveryErrorCodeUnspecified as i32
    );
    assert_eq!(
        response.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateSchedulePending as i32
    );
    let terminal = wait_for_terminal_status(
        &router,
        &runtime,
        &cookie,
        &delayed_operation_id,
        deliver_at_unix_millis,
        supervisor,
        delayed_delivery_registration_id,
    );
    assert_eq!(
        terminal.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateDeliveryAccepted as i32,
        "the durable due command must retry after losing a successful provider response"
    );
    let requests = probe.successful_request_payloads();
    assert_eq!(
        requests.len(),
        2,
        "ambiguous response loss must produce one idempotent durable replay"
    );
    assert_eq!(
        requests[0], requests[1],
        "durable replay must preserve the exact delivery-intent request bytes"
    );
    let replayed = SubmitDeliveryIntentRequestV1::decode(requests[1].as_slice())
        .expect("decode replayed delivery-intent request");
    assert_eq!(replayed.operation_id, delivery_operation_id);
    let event =
        read_delayed_delivery_terminal_sse(&router, &runtime, &cookie, &delayed_operation_id);
    assert!(
        !event
            .encode_to_vec()
            .windows(private_body.len())
            .any(|window| window == private_body),
        "ambiguous request recovery realtime must not contain private content"
    );
}

fn assert_delayed_delivery_survives_nats_outage(
    contour: &DelayedDeliveryManagedContour<'_>,
    delayed_delivery: StartedDelayedDeliveryRuntime,
    realtime: hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
    conversation_id: Vec<u8>,
    operation_byte: u8,
    authentication_sign_count: u32,
) -> StartedDelayedDeliveryRuntime {
    let DelayedDeliveryManagedContour {
        store,
        supervisor,
        root,
        data,
        ..
    } = contour;
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        store, supervisor, root, data, realtime,
    );
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
        &router,
        &runtime,
        authentication_sign_count,
    );
    let nats_endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let nats_observer = runtime
        .block_on(async_nats::connect(nats_endpoint))
        .expect("connect delayed-delivery NATS outage observer");
    let delayed_operation_id = vec![operation_byte; 16];
    let private_body = b"delayed private body retained through NATS outage";
    let deliver_at_unix_millis =
        u64::try_from(current_unix_millis()).expect("positive current time") + 8_000;
    super::nats_outage_fixture::set_authenticated_nats_container_running(false);
    let response = route_proto(
        &router,
        &runtime,
        &cookie,
        COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
        ScheduleDelayedDeliveryRequestV1 {
            protocol_major: 1,
            delayed_operation_id: delayed_operation_id.clone(),
            delivery_operation_id: vec![operation_byte.wrapping_add(1); 16],
            conversation_id,
            reply_to_message_id: None,
            body_utf8: private_body.to_vec(),
            deliver_at_unix_millis,
        }
        .encode_to_vec(),
    );
    let response =
        ScheduleDelayedDeliveryResponseV1::decode(response.as_slice()).expect("schedule response");
    std::thread::sleep(Duration::from_millis(1_500));
    let pending = delayed_delivery_status(
        &router,
        &runtime,
        &cookie,
        &delayed_operation_id,
        deliver_at_unix_millis,
    );
    let delayed_runtime_active = supervisor
        .is_active(&delayed_delivery.registration_id)
        .expect("observe delayed-delivery runtime during NATS outage");
    super::nats_outage_fixture::set_authenticated_nats_container_running(true);
    super::nats_outage_fixture::wait_for_authenticated_nats_reconnect(
        &runtime,
        &nats_observer,
        "delayed-delivery outage observer",
    );

    assert_eq!(response.delayed_operation_id, delayed_operation_id);
    assert_eq!(
        response.error,
        DelayedDeliveryErrorCodeV1::DelayedDeliveryErrorCodeUnspecified as i32
    );
    assert_eq!(
        response.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateSchedulePending as i32
    );
    assert_eq!(
        pending.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateSchedulePending as i32,
        "NATS outage must retain the exact operation before Scheduler acceptance"
    );
    assert!(
        delayed_runtime_active,
        "NATS outage must not stop the delayed-delivery runtime"
    );
    assert!(
        supervisor
            .is_active(SCHEDULER_REGISTRATION)
            .expect("observe Scheduler after NATS reconnect"),
        "bounded transient backoff must keep Scheduler active through a short NATS outage"
    );
    let terminal = wait_for_terminal_status(
        &router,
        &runtime,
        &cookie,
        &delayed_operation_id,
        deliver_at_unix_millis,
        supervisor,
        &delayed_delivery.registration_id,
    );
    assert_eq!(
        terminal.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateDeliveryAccepted as i32,
        "the retained Scheduler command must complete after NATS reconnect"
    );
    let event =
        read_delayed_delivery_terminal_sse(&router, &runtime, &cookie, &delayed_operation_id);
    assert!(
        !event
            .encode_to_vec()
            .windows(private_body.len())
            .any(|window| window == private_body),
        "NATS outage recovery realtime must not contain private content"
    );
    delayed_delivery
}

fn assert_delayed_delivery_survives_scheduler_outage(
    contour: &DelayedDeliveryManagedContour<'_>,
    delayed_delivery_registration_id: &str,
    realtime: hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
    conversation_id: Vec<u8>,
    operation_byte: u8,
    authentication_sign_count: u32,
) {
    let DelayedDeliveryManagedContour {
        store,
        supervisor,
        root,
        data,
        kernel,
    } = contour;
    supervisor
        .stop(SCHEDULER_REGISTRATION)
        .expect("stop Scheduler for delayed-delivery outage");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        store, supervisor, root, data, realtime,
    );
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
        &router,
        &runtime,
        authentication_sign_count,
    );
    let delayed_operation_id = vec![operation_byte; 16];
    let private_body = b"delayed private body retained through Scheduler outage";
    let deliver_at_unix_millis =
        u64::try_from(current_unix_millis()).expect("positive current time") + 8_000;
    let response = route_proto(
        &router,
        &runtime,
        &cookie,
        COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
        ScheduleDelayedDeliveryRequestV1 {
            protocol_major: 1,
            delayed_operation_id: delayed_operation_id.clone(),
            delivery_operation_id: vec![operation_byte.wrapping_add(1); 16],
            conversation_id,
            reply_to_message_id: None,
            body_utf8: private_body.to_vec(),
            deliver_at_unix_millis,
        }
        .encode_to_vec(),
    );
    let response =
        ScheduleDelayedDeliveryResponseV1::decode(response.as_slice()).expect("schedule response");
    std::thread::sleep(Duration::from_millis(1_500));
    let pending = delayed_delivery_status(
        &router,
        &runtime,
        &cookie,
        &delayed_operation_id,
        deliver_at_unix_millis,
    );

    assert_eq!(response.delayed_operation_id, delayed_operation_id);
    assert_eq!(
        response.error,
        DelayedDeliveryErrorCodeV1::DelayedDeliveryErrorCodeUnspecified as i32
    );
    assert_eq!(
        response.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateSchedulePending as i32
    );
    assert_eq!(
        pending.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateSchedulePending as i32,
        "Scheduler outage must retain the exact operation before acceptance"
    );
    assert!(
        supervisor
            .is_active(delayed_delivery_registration_id)
            .expect("observe delayed-delivery runtime during Scheduler outage"),
        "Scheduler outage must not stop the delayed-delivery runtime"
    );
    restart_scheduler_with_current_grants(supervisor, store, kernel, &root.join("runtime"), 3);
    let terminal = wait_for_terminal_status(
        &router,
        &runtime,
        &cookie,
        &delayed_operation_id,
        deliver_at_unix_millis,
        supervisor,
        delayed_delivery_registration_id,
    );
    assert_eq!(
        terminal.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateDeliveryAccepted as i32,
        "the retained command must complete after Scheduler successor recovery"
    );
    let event =
        read_delayed_delivery_terminal_sse(&router, &runtime, &cookie, &delayed_operation_id);
    assert!(
        !event
            .encode_to_vec()
            .windows(private_body.len())
            .any(|window| window == private_body),
        "Scheduler outage recovery realtime must not contain private content"
    );
}

fn assert_delayed_delivery_fails_closed_during_blob_outage(
    contour: &DelayedDeliveryManagedContour<'_>,
    delayed_delivery_registration_id: &str,
    realtime: hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
    conversation_id: Vec<u8>,
    operation_byte: u8,
    authentication_sign_count: u32,
) {
    let DelayedDeliveryManagedContour {
        store,
        supervisor,
        root,
        data,
        kernel,
    } = contour;
    supervisor
        .stop(blob_binding::BLOB_PROCESS_ID)
        .expect("stop Blob for delayed-delivery outage");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        store, supervisor, root, data, realtime,
    );
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
        &router,
        &runtime,
        authentication_sign_count,
    );
    let delayed_operation_id = vec![operation_byte; 16];
    let private_body = b"delayed private body must fail closed while Blob is unavailable";
    let deliver_at_unix_millis =
        u64::try_from(current_unix_millis()).expect("positive current time") + 8_000;
    let request = ScheduleDelayedDeliveryRequestV1 {
        protocol_major: 1,
        delayed_operation_id: delayed_operation_id.clone(),
        delivery_operation_id: vec![operation_byte.wrapping_add(1); 16],
        conversation_id,
        reply_to_message_id: None,
        body_utf8: private_body.to_vec(),
        deliver_at_unix_millis,
    }
    .encode_to_vec();
    let unavailable = route_proto(
        &router,
        &runtime,
        &cookie,
        COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
        request.clone(),
    );
    let unavailable = ScheduleDelayedDeliveryResponseV1::decode(unavailable.as_slice())
        .expect("Blob outage schedule response");
    assert_eq!(unavailable.delayed_operation_id, delayed_operation_id);
    assert_eq!(
        unavailable.error,
        DelayedDeliveryErrorCodeV1::DelayedDeliveryErrorCodeUnavailable as i32
    );
    assert_eq!(
        unavailable.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateUnspecified as i32
    );
    assert_eq!(unavailable.state_revision, 0);
    let absent = route_proto(
        &router,
        &runtime,
        &cookie,
        COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1,
        GetDelayedDeliveryStatusRequestV1 {
            protocol_major: 1,
            delayed_operation_id: delayed_operation_id.clone(),
        }
        .encode_to_vec(),
    );
    let absent = GetDelayedDeliveryStatusResponseV1::decode(absent.as_slice())
        .expect("Blob outage absent status response");
    assert_eq!(absent.delayed_operation_id, delayed_operation_id);
    assert_eq!(
        absent.error,
        DelayedDeliveryErrorCodeV1::DelayedDeliveryErrorCodeNotFound as i32,
        "failed Blob custody must not create a durable delayed-delivery operation"
    );
    assert!(
        supervisor
            .is_active(delayed_delivery_registration_id)
            .expect("observe delayed-delivery runtime during Blob outage"),
        "Blob outage must not stop the delayed-delivery runtime"
    );
    assert_eq!(
        blob_launch::start_from_kernel(supervisor, store, kernel, data, &root.join("runtime"))
            .expect("start Blob successor after delayed-delivery outage"),
        2
    );
    let recovered = route_proto(
        &router,
        &runtime,
        &cookie,
        COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
        request,
    );
    let recovered = ScheduleDelayedDeliveryResponseV1::decode(recovered.as_slice())
        .expect("Blob recovery schedule response");
    assert_eq!(recovered.delayed_operation_id, delayed_operation_id);
    assert_eq!(
        recovered.error,
        DelayedDeliveryErrorCodeV1::DelayedDeliveryErrorCodeUnspecified as i32
    );
    assert_eq!(
        recovered.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateSchedulePending as i32
    );
    let terminal = wait_for_terminal_status(
        &router,
        &runtime,
        &cookie,
        &delayed_operation_id,
        deliver_at_unix_millis,
        supervisor,
        delayed_delivery_registration_id,
    );
    assert_eq!(
        terminal.state,
        DelayedDeliveryStateV1::DelayedDeliveryStateDeliveryAccepted as i32,
        "the exact request must complete after Blob successor recovery"
    );
    let event =
        read_delayed_delivery_terminal_sse(&router, &runtime, &cookie, &delayed_operation_id);
    assert!(
        !event
            .encode_to_vec()
            .windows(private_body.len())
            .any(|window| window == private_body),
        "Blob outage recovery realtime must not contain private content"
    );
}

fn restart_scheduler_with_current_grants(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel: &Path,
    runtime_dir: &Path,
    expected_generation: u64,
) {
    let predecessor_binding = scheduler_binding(store);
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive Scheduler successor Storage fences");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        SCHEDULER_REGISTRATION,
        STORAGE_CAPABILITY,
        issue,
    )
    .expect("reserve Scheduler successor for current grants");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Scheduler successor Storage binding");
    assert_eq!(
        scheduler_launch::start_from_reservation(
            supervisor,
            store,
            kernel,
            runtime_dir,
            reservation,
            &binding,
        )
        .expect("start Scheduler with reconciled grants"),
        expected_generation
    );
}

fn assert_stale_delayed_route_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
) {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_DELAYED_DELIVERY_OWNER_V1.to_owned(),
        contract: Some(delayed_delivery_query_contract_v1()),
        request_id: 9_001,
        request_payload: GetDelayedDeliveryStatusRequestV1 {
            protocol_major: 1,
            delayed_operation_id: vec![0x71; 16],
        }
        .encode_to_vec(),
        logical_owner_id: DELAYED_DELIVERY_LOGICAL_OWNER_ID.to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        COMMUNICATION_DELAYED_DELIVERY_CAPABILITY_ID_V1,
        &request,
    );
    assert_eq!(
        crate::modules::capability::router::route_managed_client_request(
            store,
            &supervisor.relay_port(),
            &route,
        )
        .expect_err("stale delayed-delivery route"),
        "managed runtime fence is stale"
    );
}

fn revoke_delayed_delivery_runtime(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    delayed_delivery: &StartedDelayedDeliveryRuntime,
    delivery_intent_registration_id: &str,
) {
    let revoked = transition_registration(
        owner_runtime_dir,
        signer,
        &delayed_delivery.registration_id,
        "revoked",
    );
    assert_eq!(revoked.state, "revoked");
    assert!(revoked.grant_epoch > delayed_delivery.grant_epoch);
    let registration = store
        .module_registration(&delayed_delivery.registration_id)
        .expect("read revoked delayed-delivery registration")
        .expect("revoked delayed-delivery registration");
    assert_eq!(registration.state(), ModuleRegistrationState::Revoked);
    let binding = store
        .platform_storage_binding(
            &delayed_delivery.registration_id,
            COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read revoked delayed-delivery Storage binding")
        .expect("revoked delayed-delivery Storage binding");
    assert_eq!(
        binding.state(),
        PlatformStorageBindingStateV1::Revoking,
        "owner revoke must durably reserve the exact delayed-delivery Storage fence"
    );
    assert!(
        !supervisor
            .stop_if_active(&delayed_delivery.registration_id)
            .expect("observe stopped delayed-delivery runtime"),
        "owner transition already stopped the delayed-delivery runtime"
    );
    assert!(
        supervisor
            .is_active(delivery_intent_registration_id)
            .expect("observe delivery-intent after delayed-delivery revoke")
    );
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_DELAYED_DELIVERY_OWNER_V1.to_owned(),
        contract: Some(delayed_delivery_query_contract_v1()),
        request_id: 9_002,
        request_payload: GetDelayedDeliveryStatusRequestV1 {
            protocol_major: 1,
            delayed_operation_id: vec![0x81; 16],
        }
        .encode_to_vec(),
        logical_owner_id: DELAYED_DELIVERY_LOGICAL_OWNER_ID.to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &delayed_delivery.registration_id,
        &delayed_delivery.runtime_instance_id,
        delayed_delivery.runtime_generation,
        delayed_delivery.grant_epoch,
        COMMUNICATION_DELAYED_DELIVERY_CAPABILITY_ID_V1,
        &request,
    );
    assert_eq!(
        crate::modules::capability::router::route_managed_client_request(
            store,
            &supervisor.relay_port(),
            &route,
        )
        .expect_err("revoked delayed-delivery route"),
        "module registration is not approved"
    );
}

fn wait_for_terminal_status(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    delayed_operation_id: &[u8],
    deliver_at_unix_millis: u64,
    supervisor: &ManagedRuntimeSupervisor,
    delayed_delivery_registration_id: &str,
) -> GetDelayedDeliveryStatusResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        let response = route_proto(
            router,
            runtime,
            cookie,
            COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1,
            GetDelayedDeliveryStatusRequestV1 {
                protocol_major: 1,
                delayed_operation_id: delayed_operation_id.to_vec(),
            }
            .encode_to_vec(),
        );
        let status = GetDelayedDeliveryStatusResponseV1::decode(response.as_slice())
            .expect("delayed-delivery status");
        assert_eq!(status.requested_due_at_unix_millis, deliver_at_unix_millis);
        if matches!(
            DelayedDeliveryStateV1::try_from(status.state),
            Ok(DelayedDeliveryStateV1::DelayedDeliveryStateDeliveryAccepted
                | DelayedDeliveryStateV1::DelayedDeliveryStateFailed)
        ) {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "delayed delivery did not reach a terminal state: {}; delayed_active={:?}; \
             delayed_failure={:?}; scheduler_active={:?}; scheduler_failure={:?}",
            status.state,
            supervisor.is_active(delayed_delivery_registration_id),
            supervisor.last_failure(delayed_delivery_registration_id),
            supervisor.is_active(SCHEDULER_REGISTRATION),
            supervisor.last_failure(SCHEDULER_REGISTRATION),
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn delayed_delivery_status(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    delayed_operation_id: &[u8],
    deliver_at_unix_millis: u64,
) -> GetDelayedDeliveryStatusResponseV1 {
    let response = route_proto(
        router,
        runtime,
        cookie,
        COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1,
        GetDelayedDeliveryStatusRequestV1 {
            protocol_major: 1,
            delayed_operation_id: delayed_operation_id.to_vec(),
        }
        .encode_to_vec(),
    );
    let status =
        GetDelayedDeliveryStatusResponseV1::decode(response.as_slice()).expect("status response");
    assert_eq!(
        status.requested_due_at_unix_millis, deliver_at_unix_millis,
        "status must retain the requested due time"
    );
    status
}

fn route_proto(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    payload: Vec<u8>,
) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let response = runtime.block_on(
            router.route(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/connect+proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(Bytes::from(payload.clone())))
                    .expect("Gateway protobuf request"),
            ),
        );
        let status = response.status();
        let body = runtime
            .block_on(response.into_body().collect())
            .expect("Gateway protobuf response")
            .to_bytes()
            .to_vec();
        if status == StatusCode::OK {
            return body;
        }
        assert!(
            status == StatusCode::INTERNAL_SERVER_ERROR && Instant::now() < deadline,
            "Gateway protobuf route failed with {status}: {}",
            String::from_utf8_lossy(&body)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn read_delayed_delivery_terminal_sse(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    delayed_operation_id: &[u8],
) -> hermes_gateway_protocol::v1::ClientRealtimeEventV1 {
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(8),
            find_delayed_delivery_terminal_event(response.into_body(), delayed_operation_id),
        )
        .await
        .expect("delayed-delivery SSE event timeout")
    })
}

async fn find_delayed_delivery_terminal_event<B>(
    mut body: B,
    delayed_operation_id: &[u8],
) -> hermes_gateway_protocol::v1::ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Gateway SSE body frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Gateway SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Gateway realtime frame");
            let frame =
                ClientRealtimeFrameV1::decode(bytes.as_slice()).expect("decode realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != COMMUNICATION_DELAYED_DELIVERY_REALTIME_CONTRACT_NAME_V1 {
                continue;
            }
            let payload = DelayedDeliveryStatusChangedV1::decode(event.payload.as_slice())
                .expect("decode delayed-delivery realtime event");
            if payload.delayed_operation_id == delayed_operation_id
                && payload.state
                    == DelayedDeliveryStateV1::DelayedDeliveryStateDeliveryAccepted as i32
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before delayed-delivery terminal event");
}
