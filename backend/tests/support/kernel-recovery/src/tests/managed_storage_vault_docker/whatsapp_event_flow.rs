//! Live WhatsApp host execution and event-only Communications handoff.

use std::time::Instant;

use super::*;

use hermes_whatsapp_api::{
    WhatsAppProviderCommand, WhatsAppProviderCommandStateV1, WhatsAppPublicClientRequestV1,
    WhatsAppPublicClientResponseV1,
    client_contract::WhatsAppClientContractV1,
    host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostBridgeEnvelopeV1,
        WhatsAppHostObservationV1,
    },
};
use hermes_whatsapp_runtime::client_port::{decode_module_response, encode_module_request};

use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

const OBSERVATION_SUBJECT: &str = "hermes.observation.v1.communications.communication_observed.v1";
const CANONICAL_EVENT_SUBJECT: &str =
    "hermes.event.v1.communications.communication_evidence_recorded.v1";
const PRIVATE_COMMAND_TEXT: &str = "private WhatsApp body must stay integration-owned";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, WhatsApp, NATS and Communications binaries"]
fn managed_whatsapp_runtime_delivers_live_command_and_event_only_communications_handoff() {
    let contour = ManagedWhatsAppContour::start(WhatsAppGrantProfileV1::CommandAndQuery);
    let events = contour
        .store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("WhatsApp event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (client, mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect WhatsApp event observer");
        let observations = client
            .subscribe(OBSERVATION_SUBJECT)
            .await
            .expect("subscribe WhatsApp observations");
        let canonical_events = client
            .subscribe(CANONICAL_EVENT_SUBJECT)
            .await
            .expect("subscribe canonical Communications events");
        client
            .flush()
            .await
            .expect("activate WhatsApp event observers");
        (client, observations, canonical_events)
    });

    const OPERATION_ID: &str = "managed-whatsapp-live-command-1";
    const HOST_CLAIM_ID: &str = "managed-whatsapp-host-claim-1";
    assert_whatsapp_command_accepted(&contour, OPERATION_ID);
    execute_whatsapp_command(&contour, OPERATION_ID, HOST_CLAIM_ID);
    assert_whatsapp_operation_succeeded(&contour, OPERATION_ID);
    assert_command_result_stays_owner_local(
        &event_runtime,
        &mut observations,
        &mut canonical_events,
    );

    let message_bytes =
        submit_message_observation(&contour, "whatsapp-provider-event-1", "provider-message-1");
    let (observation_bytes, observation_message_id, canonical_message_id) =
        receive_whatsapp_observation(
            &event_runtime,
            &mut observations,
            &mut canonical_events,
            &contour,
            "initial message",
        );
    assert_private_command_text_absent(&observation_bytes);
    assert_ne!(
        observation_bytes, message_bytes,
        "the private host operation payload is not the durable observation envelope",
    );

    event_runtime.block_on(async {
        client
            .publish(OBSERVATION_SUBJECT, observation_bytes.clone().into())
            .await
            .expect("republish exact WhatsApp observation");
        client
            .flush()
            .await
            .expect("flush duplicate WhatsApp observation");
        let duplicate = tokio::time::timeout(Duration::from_secs(1), observations.next())
            .await
            .expect("duplicate WhatsApp observation timeout")
            .expect("duplicate WhatsApp observation");
        let duplicate = decode_envelope_v1(duplicate.payload.as_ref())
            .expect("duplicate WhatsApp observation envelope");
        assert_eq!(duplicate.message_id, observation_message_id);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate WhatsApp observation must not create a second Communications event",
        );
    });
    let initial_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);

    set_authenticated_nats_container_running(false);
    submit_message_observation(&contour, "whatsapp-provider-event-2", "provider-message-2");
    std::thread::sleep(Duration::from_millis(2_500));
    assert!(
        contour
            .supervisor
            .is_active(&contour.whatsapp.registration_id)
            .expect("read managed WhatsApp state"),
        "managed WhatsApp runtime must remain active while NATS is unavailable",
    );
    assert_eq!(
        contour
            .supervisor
            .last_failure(&contour.whatsapp.registration_id)
            .expect("read managed WhatsApp failure"),
        None,
    );
    set_authenticated_nats_container_running(true);
    wait_for_authenticated_nats_reconnect(&event_runtime, &client, "WhatsApp event observer");

    let (_, replayed_observation_id, replayed_canonical_id) = receive_whatsapp_observation(
        &event_runtime,
        &mut observations,
        &mut canonical_events,
        &contour,
        "outage replay",
    );
    assert_ne!(replayed_observation_id, observation_message_id);
    assert_ne!(
        replayed_canonical_id, canonical_message_id,
        "outage replay must deliver the second WhatsApp provider observation",
    );
    let replayed_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);
    assert_ne!(
        replayed_evidence_id, initial_evidence_id,
        "Communications query must expose the replayed WhatsApp evidence",
    );

    contour.shutdown_processes();
    contour.finish();
}

fn assert_command_result_stays_owner_local(
    runtime: &tokio::runtime::Runtime,
    observations: &mut async_nats::Subscriber,
    canonical_events: &mut async_nats::Subscriber,
) {
    runtime.block_on(async {
        assert!(
            tokio::time::timeout(Duration::from_millis(500), observations.next())
                .await
                .is_err(),
            "WhatsApp terminal command result must not become Communications evidence",
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(500), canonical_events.next())
                .await
                .is_err(),
            "Communications must not create canonical truth from a provider command receipt",
        );
    });
}

fn assert_whatsapp_command_accepted(contour: &ManagedWhatsAppContour, operation_id: &str) {
    let response = route_whatsapp_client(
        contour,
        WhatsAppClientContractV1::Command,
        31,
        &WhatsAppPublicClientRequestV1::Command(WhatsAppProviderCommand::SendText {
            operation_id: operation_id.to_owned(),
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            provider_chat_id: "provider-chat-1".to_owned(),
            text: PRIVATE_COMMAND_TEXT.to_owned(),
        }),
    );
    assert!(
        matches!(
            response,
            WhatsAppPublicClientResponseV1::Accepted { operation_id: accepted }
                if accepted == operation_id
        ),
        "WhatsApp command must return only an accepted receipt",
    );
}

fn execute_whatsapp_command(
    contour: &ManagedWhatsAppContour,
    operation_id: &str,
    host_claim_id: &str,
) {
    let mut host = WhatsAppHostBridgeTestClient::connect(&contour.whatsapp);
    let commands = host.claim_commands(WHATSAPP_ACCOUNT_ID, host_claim_id);
    assert!(
        matches!(
            commands.as_slice(),
            [WhatsAppProviderCommand::SendText {
                operation_id: claimed_operation_id,
                account_id,
                provider_chat_id,
                text,
            }] if claimed_operation_id == operation_id
                && account_id == WHATSAPP_ACCOUNT_ID
                && provider_chat_id == "provider-chat-1"
                && text == PRIVATE_COMMAND_TEXT
        ),
        "the native host must lease the exact integration-owned provider command",
    );
    let provider_event_id = "whatsapp-command-result-1";
    assert_eq!(
        host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            provider_event_id: provider_event_id.to_owned(),
            observed_at_unix_seconds: 1_785_000_001,
            observation: WhatsAppHostObservationV1::CommandResult {
                operation_id: operation_id.to_owned(),
                provider_request_id: Some("provider-request-1".to_owned()),
                succeeded: true,
                host_claim_id: host_claim_id.to_owned(),
            },
        }),
        provider_event_id,
    );
}

fn assert_whatsapp_operation_succeeded(contour: &ManagedWhatsAppContour, operation_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_whatsapp_client(
            contour,
            WhatsAppClientContractV1::Query,
            32,
            &WhatsAppPublicClientRequestV1::OperationStatus {
                operation_id: operation_id.to_owned(),
            },
        );
        if matches!(
            response,
            WhatsAppPublicClientResponseV1::OperationStatus(Some(status))
                if status.operation_id == operation_id
                    && status.account_id == WHATSAPP_ACCOUNT_ID
                    && status.state == WhatsAppProviderCommandStateV1::Succeeded
                    && status.completed_at_unix_seconds.is_some()
        ) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "WhatsApp provider command did not reach terminal success",
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn submit_message_observation(
    contour: &ManagedWhatsAppContour,
    provider_event_id: &str,
    provider_message_id: &str,
) -> Vec<u8> {
    let envelope = WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: provider_event_id.to_owned(),
        observed_at_unix_seconds: 1_785_000_002,
        observation: WhatsAppHostObservationV1::MessageIdentity {
            provider_chat_id: "provider-chat-1".to_owned(),
            provider_message_id: provider_message_id.to_owned(),
            sender_id: "provider-sender-1".to_owned(),
        },
    };
    let private_host_payload =
        hermes_whatsapp_api::host_bridge::encode_host_bridge_payload(&envelope)
            .expect("encode private WhatsApp host operation");
    let mut host = WhatsAppHostBridgeTestClient::connect(&contour.whatsapp);
    assert_eq!(host.submit_observation(&envelope), provider_event_id);
    private_host_payload
}

fn route_whatsapp_client(
    contour: &ManagedWhatsAppContour,
    contract: WhatsAppClientContractV1,
    request_id: u64,
    request: &WhatsAppPublicClientRequestV1,
) -> WhatsAppPublicClientResponseV1 {
    let encoded =
        encode_module_request(request_id, request).expect("encode WhatsApp client request");
    let relay = contour.supervisor.relay_port();
    let deadline = Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&contour.whatsapp.registration_id) != Ok(true) {
        assert!(
            Instant::now() < deadline,
            "managed WhatsApp runtime did not become ready: {:?}",
            contour
                .supervisor
                .last_failure(&contour.whatsapp.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &contour.whatsapp.registration_id,
            &contour.whatsapp.runtime_instance_id,
            contour.whatsapp.runtime_generation,
            contour.whatsapp.grant_epoch,
            contract.capability_id(),
            &encoded,
        );
        let last_error = match route_managed_client_request(contour.store.as_ref(), &relay, &route)
        {
            Ok(bytes) => match decode_module_response(contract, &bytes) {
                Ok((response_id, response)) if response_id == request_id => return response,
                outcome => format!("unexpected WhatsApp response: {outcome:?}"),
            },
            Err(error) => error,
        };
        assert!(
            Instant::now() < deadline,
            "WhatsApp client route remained unavailable: {last_error}; managed failure: {:?}",
            contour
                .supervisor
                .last_failure(&contour.whatsapp.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn receive_whatsapp_observation(
    runtime: &tokio::runtime::Runtime,
    observations: &mut async_nats::Subscriber,
    canonical_events: &mut async_nats::Subscriber,
    contour: &ManagedWhatsAppContour,
    phase: &str,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (observation, canonical) = runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} WhatsApp observation timeout"))
            .unwrap_or_else(|| panic!("{phase} WhatsApp observation"));
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} Communications event timeout"))
            .unwrap_or_else(|| panic!("{phase} Communications event"));
        (observation, canonical)
    });
    let observation_bytes = observation.payload.to_vec();
    assert_private_host_route_absent(&observation_bytes, &contour.whatsapp);
    let observation =
        decode_envelope_v1(&observation_bytes).expect("WhatsApp observation durable envelope");
    let source = observation
        .source
        .as_ref()
        .expect("WhatsApp observation source");
    assert_eq!(source.module_id, hermes_whatsapp_runtime::PACKAGE);
    assert_eq!(
        source.runtime_generation,
        contour.whatsapp.runtime_generation,
    );
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications event envelope");
    assert_eq!(
        canonical.causation_message_id, observation.message_id,
        "Communications must derive canonical evidence only from the typed WhatsApp observation",
    );
    (
        observation_bytes,
        observation.message_id,
        canonical.message_id,
    )
}

fn assert_private_host_route_absent(bytes: &[u8], runtime: &StartedWhatsAppRuntime) {
    let socket_path = runtime.host_bridge_socket_path.to_string_lossy();
    assert!(
        !bytes
            .windows(socket_path.len())
            .any(|window| window == socket_path.as_bytes()),
        "private WhatsApp host socket path must not enter the durable event",
    );
    assert!(
        !bytes
            .windows(runtime.route_binding_sha256.len())
            .any(|window| window == runtime.route_binding_sha256.as_slice()),
        "private WhatsApp host route binding must not enter the durable event",
    );
}

fn assert_private_command_text_absent(bytes: &[u8]) {
    assert!(
        !bytes
            .windows(PRIVATE_COMMAND_TEXT.len())
            .any(|window| window == PRIVATE_COMMAND_TEXT.as_bytes()),
        "provider command body must not enter the durable Communications envelope",
    );
}
