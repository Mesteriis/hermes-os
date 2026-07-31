//! Managed Ollama negative conformance without installing or simulating a provider.

use std::io::ErrorKind;
use std::net::TcpListener;

use super::*;

use hermes_ai_contracts::{
    AI_LOCAL_EGRESS_POLICY_REVISION_V1, ai_provider_reply_generation_contract_reference_v1,
    wire::{
        AiEgressPolicyV1, AiInferenceTerminalStatusV1, AiProviderReplyGenerationRequestV1,
        AiProviderReplyGenerationResultV1, AiReplyLanguageV1, AiReplySubjectPolicyV1,
        AiReplyToneV1,
    },
};
use hermes_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage and Ollama AI binaries"]
fn managed_ollama_ai_runtime_replays_provider_unavailable_without_second_http_attempt() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let port_reservation =
        TcpListener::bind(("127.0.0.1", 0)).expect("reserve unavailable Ollama port");
    let ollama_port = port_reservation
        .local_addr()
        .expect("read unavailable Ollama port")
        .port();
    drop(port_reservation);

    let root = unique_target_root("hermes-managed-ollama-ai-negative");
    let data = private_directory(root.join("kernel"));
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_ollama_ai_release_v1(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_store(&root, release.kernel()));
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            OLLAMA_AI_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Ollama AI logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let admitted = admit_ollama_ai_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted);
    let runtime = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted,
        ollama_port,
    );

    let request = provider_request_v1();
    let first = deliver_provider_request_v1(&supervisor, &runtime.registration_id, &request);
    assert!(first.error_code.is_empty());
    let first_result = AiProviderReplyGenerationResultV1::decode(first.response_payload.as_slice())
        .expect("typed Ollama provider-unavailable result");
    assert_eq!(first_result.request_id, request.request_id);
    assert_eq!(
        first_result.terminal_status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable as i32
    );

    let no_second_attempt =
        TcpListener::bind(("127.0.0.1", ollama_port)).expect("guard Ollama replay port");
    no_second_attempt
        .set_nonblocking(true)
        .expect("make Ollama replay guard nonblocking");
    let previous_generation = runtime.runtime_generation;
    let runtime = restart_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        runtime,
        ollama_port,
    );
    assert_eq!(runtime.runtime_generation, previous_generation + 1);

    let replayed = deliver_provider_request_v1(&supervisor, &runtime.registration_id, &request);
    assert_eq!(replayed, first);
    assert_no_ollama_connection_v1(&no_second_attempt);

    let mut conflicting = request.clone();
    conflicting.input_utf8.extend_from_slice(b" changed");
    let rejected = deliver_provider_request_v1(&supervisor, &runtime.registration_id, &conflicting);
    assert_eq!(rejected.request_id, request.request_id);
    assert_eq!(rejected.error_code, "REJECTED");
    assert!(rejected.response_payload.is_empty());
    assert_no_ollama_connection_v1(&no_second_attempt);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Ollama AI fixture");
}

fn provider_request_v1() -> AiProviderReplyGenerationRequestV1 {
    AiProviderReplyGenerationRequestV1 {
        request_id: vec![0x61; 16],
        input_utf8: b"Private source for a bounded local reply".to_vec(),
        tone: AiReplyToneV1::AiReplyToneWarm as i32,
        language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
        subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyOmit as i32,
        maximum_output_bytes: 1_024,
        maximum_output_tokens: 256,
        egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
        egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
    }
}

fn deliver_provider_request_v1(
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    request: &AiProviderReplyGenerationRequestV1,
) -> ManagedRuntimeModuleRequestResponseV1 {
    let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
        request_id: request.request_id.clone(),
        logical_owner_id: OLLAMA_AI_LOGICAL_OWNER_ID_V1.to_owned(),
        contract: Some(ai_provider_reply_generation_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
    };
    let response = supervisor
        .relay(
            registration_id,
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::DeliverModuleRequest(delivery)),
            }
            .encode_to_vec(),
        )
        .expect("deliver managed Ollama provider request");
    let response = ManagedRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode managed Ollama response");
    assert!(response.error_code.is_empty());
    match response.result {
        Some(ControlResult::ModuleRequestDelivery(response)) => response,
        _ => panic!("managed Ollama response is missing"),
    }
}

fn assert_no_ollama_connection_v1(listener: &TcpListener) {
    match listener.accept() {
        Err(error) if error.kind() == ErrorKind::WouldBlock => {}
        Ok(_) => panic!("persisted Ollama terminal replay attempted provider HTTP"),
        Err(error) => panic!("inspect Ollama replay guard: {error}"),
    }
}
