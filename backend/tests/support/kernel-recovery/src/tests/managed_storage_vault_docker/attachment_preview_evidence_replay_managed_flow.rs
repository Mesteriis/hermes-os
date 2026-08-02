//! Live signed admission smoke gate for retained Preview evidence replay.

use super::*;

use crate::identity::device::signer::DeviceSigner;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and replay workflow binaries"]
fn managed_attachment_preview_evidence_replay_runtime_starts_with_exact_signed_contracts() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-attachment-preview-evidence-replay");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_preview_replay_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim retained Preview evidence replay logical owner");

    let _admitted_mail = admit_mail_runtime(&store);
    let admitted_replay = admit_attachment_preview_evidence_replay_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
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
    let admitted_replay =
        prepare_attachment_preview_evidence_replay_runtime_v1(&supervisor, &store, admitted_replay);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let replay = start_attachment_preview_evidence_replay_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_replay,
    );
    assert!(
        supervisor
            .is_active(&replay.registration_id)
            .expect("read retained Preview evidence replay process state")
    );
    assert_eq!(replay.runtime_generation, 1);
    assert!(replay.grant_epoch > 0);
    assert!(!replay.runtime_instance_id.is_empty());

    let previous_runtime_instance_id = replay.runtime_instance_id.clone();
    supervisor
        .stop(&replay.registration_id)
        .expect("stop retained Preview evidence replay predecessor");
    let replay = restart_attachment_preview_evidence_replay_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        replay,
    );
    assert_eq!(replay.runtime_generation, 2);
    assert_ne!(replay.runtime_instance_id, previous_runtime_instance_id);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove retained Preview evidence replay fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}
