//! Signed managed lifecycle smoke gate for Attachment Translation.

use super::*;

use std::net::TcpListener;

use crate::identity::device::signer::DeviceSigner;

#[test]
#[ignore = "requires disposable Docker plus managed Vault, Storage, Blob, NATS, Text Extraction, AI inference, Ollama AI and Attachment Translation binaries"]
fn managed_attachment_translation_runtime_starts_with_exact_signed_contracts() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let ollama_port = unused_loopback_port_v1();
    let root = unique_target_root("hermes-managed-attachment-translation");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_translation_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Attachment Translation logical owner");

    let admitted_extraction = admit_attachment_text_extraction_runtime_v1(&store);
    let admitted_translation = admit_attachment_translation_runtime_v1(&store);
    let admitted_ollama = admit_ollama_ai_runtime_v1(&store);
    let admitted_ai = admit_ai_inference_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        hermes_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_ai_module_request_router_v1(&supervisor, &store);
    configure_attachment_translation_realtime_v1(&supervisor, &store, realtime);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted_extraction =
        prepare_attachment_text_extraction_runtime_v1(&supervisor, &store, admitted_extraction);
    let admitted_translation =
        prepare_attachment_translation_runtime_v1(&supervisor, &store, admitted_translation);
    let admitted_ollama = prepare_ollama_ai_runtime_v1(&supervisor, &store, admitted_ollama);
    let admitted_ai = prepare_ai_inference_runtime_v1(&supervisor, &store, admitted_ai);
    configure_communications_jetstream(&store);

    let ollama = start_ollama_ai_runtime_v1(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_ollama,
        ollama_port,
    );
    let ai = start_ai_inference_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted_ai);
    let extraction = start_attachment_text_extraction_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_extraction,
    );
    let translation = start_attachment_translation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_translation,
    );
    for (registration_id, owner) in [
        (ollama.registration_id.as_str(), "Ollama integration"),
        (ai.registration_id.as_str(), "AI engine"),
        (
            extraction.registration_id.as_str(),
            "Text Extraction workflow",
        ),
        (
            translation.registration_id.as_str(),
            "Attachment Translation workflow",
        ),
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .unwrap_or_else(|error| panic!("observe {owner}: {error}")),
            "{owner} must remain an independently active managed process"
        );
    }
    assert_eq!(translation.runtime_generation, 1);
    assert!(translation.grant_epoch > 0);
    assert!(!translation.runtime_instance_id.is_empty());

    let previous_instance = translation.runtime_instance_id.clone();
    let translation = restart_attachment_translation_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        translation,
    );
    assert_eq!(translation.runtime_generation, 2);
    assert_ne!(translation.runtime_instance_id, previous_instance);
    for (registration_id, owner) in [
        (ollama.registration_id.as_str(), "Ollama integration"),
        (ai.registration_id.as_str(), "AI engine"),
        (
            extraction.registration_id.as_str(),
            "Text Extraction workflow",
        ),
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .unwrap_or_else(|error| panic!("observe {owner} after restart: {error}")),
            "Attachment Translation restart must not restart {owner}"
        );
    }

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Attachment Translation fixture");
    std::fs::remove_dir_all(data).expect("remove short Attachment Translation Kernel fixture");
}

fn unused_loopback_port_v1() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve unused Ollama port");
    listener.local_addr().expect("unused Ollama address").port()
}
