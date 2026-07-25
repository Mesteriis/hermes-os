//! Live Zulip command and typed event-only Communications handoff conformance.

use std::time::Instant;

use super::*;

use hermes_events_protocol::validation::envelope::decode_envelope_v1;
use hermes_zulip_api::{
    ZulipClientRequestV1, ZulipClientResponseV1, ZulipCommandOperationOutcomeV1, ZulipCommandV1,
    client_contract::{ZULIP_MODULE_ID, ZulipClientContractV1},
};
use hermes_zulip_runtime::client_port::{
    ZulipClientPortErrorV1, decode_module_response, encode_module_request,
};

use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

const OBSERVATION_SUBJECT: &str = "hermes.observation.v1.communications.communication_observed.v1";
const CANONICAL_EVENT_SUBJECT: &str =
    "hermes.event.v1.communications.communication_evidence_recorded.v1";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Zulip binaries"]
fn managed_zulip_runtime_delivers_live_command_and_event_only_communications_handoff() {
    let contour = ManagedZulipContour::start(ZulipGrantProfileV1::CommandAndQuery);
    let events = contour
        .store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("Zulip event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (client, mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect Zulip event observer");
        let observations = client
            .subscribe(OBSERVATION_SUBJECT)
            .await
            .expect("subscribe Zulip observations");
        let canonical_events = client
            .subscribe(CANONICAL_EVENT_SUBJECT)
            .await
            .expect("subscribe canonical Communications events");
        client
            .flush()
            .await
            .expect("activate Zulip event observers");
        (client, observations, canonical_events)
    });

    const OPERATION_ID: &str = "managed-zulip-live-command-1";
    assert_zulip_command_accepted(&contour, OPERATION_ID);
    assert_zulip_operation_completed(&contour, OPERATION_ID);
    assert_eq!(
        contour.fixture.message_commands(),
        1,
        "the accepted command must execute exactly once against the live provider"
    );

    assert_eq!(contour.fixture.release_next_event(), 1);
    let (observation_bytes, observation_message_id, canonical_message_id) =
        receive_zulip_observation(
            &event_runtime,
            &mut observations,
            &mut canonical_events,
            &contour,
            "initial",
        );
    event_runtime.block_on(async {
        client
            .publish(OBSERVATION_SUBJECT, observation_bytes.into())
            .await
            .expect("republish exact Zulip observation");
        client
            .flush()
            .await
            .expect("flush duplicate Zulip observation");
        let duplicate = tokio::time::timeout(Duration::from_secs(1), observations.next())
            .await
            .expect("duplicate Zulip observation timeout")
            .expect("duplicate Zulip observation");
        let duplicate = decode_envelope_v1(duplicate.payload.as_ref())
            .expect("duplicate Zulip observation envelope");
        assert_eq!(duplicate.message_id, observation_message_id);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate Zulip observation must not create a second Communications event"
        );
    });
    let initial_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);

    set_authenticated_nats_container_running(false);
    assert_eq!(contour.fixture.release_next_event(), 2);
    wait_for_served_event(&contour, 2);
    std::thread::sleep(Duration::from_millis(2_500));
    assert!(
        contour
            .supervisor
            .is_active(&contour.zulip.registration_id)
            .expect("read managed Zulip state"),
        "managed Zulip runtime must remain active while NATS is unavailable"
    );
    assert_eq!(
        contour
            .supervisor
            .last_failure(&contour.zulip.registration_id)
            .expect("read managed Zulip failure"),
        None
    );
    set_authenticated_nats_container_running(true);
    wait_for_authenticated_nats_reconnect(&event_runtime, &client, "Zulip event observer");

    let (_, replayed_observation_id, replayed_canonical_id) = receive_zulip_observation(
        &event_runtime,
        &mut observations,
        &mut canonical_events,
        &contour,
        "outage replay",
    );
    assert_ne!(replayed_observation_id, observation_message_id);
    assert_ne!(
        replayed_canonical_id, canonical_message_id,
        "outage replay must deliver the second provider observation"
    );
    let replayed_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);
    assert_ne!(
        replayed_evidence_id, initial_evidence_id,
        "Communications query must expose the replayed Zulip evidence"
    );

    contour.shutdown_processes();
    contour.finish();
}

fn assert_zulip_command_accepted(contour: &ManagedZulipContour, operation_id: &str) {
    let request = ZulipClientRequestV1::Command(ZulipCommandV1::SendStream {
        operation_id: operation_id.to_owned(),
        account_id: ZULIP_ACCOUNT_ID.to_owned(),
        stream: "operations".to_owned(),
        topic: "managed".to_owned(),
        content: "managed Zulip provider command".to_owned(),
    });
    let response = route_zulip_client(contour, ZulipClientContractV1::Command, 31, &request);
    let ZulipClientResponseV1::CommandReceipt(receipt) = response else {
        panic!("Zulip command returned the wrong response type");
    };
    assert_eq!(receipt.operation_id, operation_id);
    assert_eq!(receipt.account_id, ZULIP_ACCOUNT_ID);
}

fn assert_zulip_operation_completed(contour: &ManagedZulipContour, operation_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_zulip_client(
            contour,
            ZulipClientContractV1::Query,
            32,
            &ZulipClientRequestV1::OperationStatus {
                operation_id: operation_id.to_owned(),
            },
        );
        let ZulipClientResponseV1::OperationStatus(status) = response else {
            panic!("Zulip operation query returned the wrong response type");
        };
        if let Some(status) = status {
            assert_eq!(status.operation_id, operation_id);
            assert_eq!(status.account_id, ZULIP_ACCOUNT_ID);
            match status.outcome {
                ZulipCommandOperationOutcomeV1::Accepted {
                    provider_message_id: Some(4242),
                    blob_ref: None,
                } => return,
                ZulipCommandOperationOutcomeV1::Rejected => {
                    panic!("Zulip provider command was rejected")
                }
                _ => {}
            }
        }
        assert!(
            Instant::now() < deadline,
            "Zulip provider command did not reach a terminal result"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn route_zulip_client(
    contour: &ManagedZulipContour,
    contract: ZulipClientContractV1,
    request_id: u64,
    request: &ZulipClientRequestV1,
) -> ZulipClientResponseV1 {
    let encoded = encode_module_request(request_id, request).expect("encode Zulip client request");
    let relay = contour.supervisor.relay_port();
    let deadline = Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&contour.zulip.registration_id) != Ok(true) {
        assert!(
            Instant::now() < deadline,
            "managed Zulip runtime did not become ready: {:?}",
            contour
                .supervisor
                .last_failure(&contour.zulip.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &contour.zulip.registration_id,
            &contour.zulip.runtime_instance_id,
            contour.zulip.runtime_generation,
            contour.zulip.grant_epoch,
            contract.capability_id(),
            &encoded,
        );
        let last_error = match route_managed_client_request(contour.store.as_ref(), &relay, &route)
        {
            Ok(bytes) => match decode_module_response(contract, &bytes) {
                Ok((response_id, response)) if response_id == request_id => return response,
                Ok((response_id, _)) => format!("unexpected response id {response_id}"),
                Err(ZulipClientPortErrorV1::Protocol) => "invalid Zulip route response".to_owned(),
                Err(ZulipClientPortErrorV1::Runtime) => "Zulip route runtime error".to_owned(),
            },
            Err(error) => error,
        };
        assert!(
            Instant::now() < deadline,
            "Zulip client route remained unavailable: {last_error}; managed failure: {:?}",
            contour
                .supervisor
                .last_failure(&contour.zulip.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn receive_zulip_observation(
    runtime: &tokio::runtime::Runtime,
    observations: &mut async_nats::Subscriber,
    canonical_events: &mut async_nats::Subscriber,
    contour: &ManagedZulipContour,
    phase: &str,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (observation, canonical) = runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} Zulip observation timeout"))
            .unwrap_or_else(|| panic!("{phase} Zulip observation"));
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} Communications event timeout"))
            .unwrap_or_else(|| panic!("{phase} Communications event"));
        (observation, canonical)
    });
    let observation_bytes = observation.payload.to_vec();
    let observation =
        decode_envelope_v1(&observation_bytes).expect("Zulip observation durable envelope");
    let source = observation
        .source
        .as_ref()
        .expect("Zulip observation source");
    assert_eq!(source.module_id, ZULIP_MODULE_ID);
    assert_eq!(source.runtime_generation, contour.zulip.runtime_generation);
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications event envelope");
    assert_eq!(
        canonical.causation_message_id, observation.message_id,
        "Communications must derive canonical evidence only from the typed Zulip observation"
    );
    (
        observation_bytes,
        observation.message_id,
        canonical.message_id,
    )
}

fn wait_for_served_event(contour: &ManagedZulipContour, expected: u64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while contour.fixture.served_events() < expected {
        assert!(
            Instant::now() < deadline,
            "managed Zulip runtime did not poll released provider event {expected}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
