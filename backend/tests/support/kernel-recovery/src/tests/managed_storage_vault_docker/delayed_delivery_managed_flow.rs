//! Live co-admission of Scheduler and the two independent communication workflows.

use super::*;

use std::time::Instant;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hermes_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_DELAYED_DELIVERY_SCHEDULE_CONNECT_PATH_V1,
    COMMUNICATION_DELAYED_DELIVERY_STATUS_CONNECT_PATH_V1,
    wire::{
        DelayedDeliveryErrorCodeV1, DelayedDeliveryStateV1, DelayedDeliveryStatusChangedV1,
        GetDelayedDeliveryStatusRequestV1, GetDelayedDeliveryStatusResponseV1,
        ScheduleDelayedDeliveryRequestV1, ScheduleDelayedDeliveryResponseV1,
    },
};
use hermes_gateway_protocol::v1::{
    ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};

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
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            DELAYED_DELIVERY_LOGICAL_OWNER_ID,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        DELAYED_DELIVERY_LOGICAL_OWNER_ID,
    );
    record_scheduler_runtime(&store);
    let delivery_intent = admit_delivery_intent_runtime(&store);
    let delayed_delivery = admit_delayed_delivery_runtime(&store);
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        hermes_gateway_runtime::InMemoryBrowserRealtimeSource::new(32).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_delivery_intent_runtime_routes(&supervisor, &store, realtime.clone());
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
    assert_delayed_delivery_round_trip(
        &store,
        &supervisor,
        &root,
        &data,
        realtime,
        conversation_id,
    );

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove kernel fixture");
}

fn assert_delayed_delivery_round_trip(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
    conversation_id: Vec<u8>,
) {
    let runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = super::delivery_intent_realtime_flow::delivery_intent_gateway(
        store, supervisor, root, data, realtime,
    );
    let cookie =
        super::super::browser_gateway_session::authenticate_gateway_router(&router, &runtime);
    let delayed_operation_id = vec![0x71; 16];
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
            delivery_operation_id: vec![0x72; 16],
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

fn wait_for_terminal_status(
    router: &super::delivery_intent_realtime_flow::DeliveryIntentGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    delayed_operation_id: &[u8],
    deliver_at_unix_millis: u64,
) -> GetDelayedDeliveryStatusResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(16);
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
            "delayed delivery did not reach a terminal state: {}",
            status.state
        );
        std::thread::sleep(Duration::from_millis(100));
    }
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
