//! Live signed Attachment Text Extraction launch with exact staged OCR resources.

use super::*;

use crate::identity::device::signer::DeviceSigner;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and Attachment Text Extraction binaries"]
fn managed_attachment_text_extraction_starts_with_exact_staged_ocr_resources() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-attachment-text-extraction");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_text_extraction_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Attachment Text Extraction logical owner");

    let admitted_text = admit_attachment_text_extraction_runtime_v1(&store);
    let _admitted_security = admit_attachment_security_runtime(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
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
    let admitted_text =
        prepare_attachment_text_extraction_runtime_v1(&supervisor, &store, admitted_text);
    configure_communications_jetstream(&store);
    let started = start_attachment_text_extraction_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_text,
    );

    assert!(
        supervisor
            .is_active(&started.registration_id)
            .expect("read Attachment Text Extraction process state")
    );
    assert_eq!(started.runtime_generation, 1);
    assert!(started.grant_epoch > 0);
    assert!(!started.runtime_instance_id.is_empty());
    assert!(
        started
            .capability_ids
            .iter()
            .any(|capability| capability == "attachment_text_extraction.ocr_runtime.v1")
    );
}
