//! Live signed managed Engine admission before attachment verdict scenarios.

use super::*;

use super::attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture;
use crate::identity::device::signer::DeviceSigner;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and Attachment Security binaries"]
fn managed_attachment_security_engine_starts_with_exact_signed_contracts() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("hermes-managed-attachment-security");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_mail_attachment_security_release(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim initial owner");
    let _admitted_mail = admit_mail_runtime(&store);
    let admitted_attachment_security = admit_attachment_security_runtime(&store);
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
    let admitted_attachment_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_attachment_security);
    configure_communications_jetstream(&store);
    let attachment_security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_attachment_security,
        clamav.port(),
    );
    assert!(
        supervisor
            .is_active(&attachment_security.registration_id)
            .expect("read Attachment Security process state")
    );
    assert_eq!(attachment_security.runtime_generation, 1);
    assert!(attachment_security.grant_epoch > 0);
    assert!(!attachment_security.runtime_instance_id.is_empty());
    assert_eq!(clamav.scan_count(), 0);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}
