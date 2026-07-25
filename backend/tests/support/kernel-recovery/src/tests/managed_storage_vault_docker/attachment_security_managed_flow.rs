//! Live signed managed Engine admission before attachment verdict scenarios.

use super::*;

use super::attachment_security_clamav_fixture::{
    AttachmentSecurityClamAvFixture, ClamAvFixtureOutcomeV1,
};
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
    let blob_source = AttachmentSecurityBlobSourceFixture::admit(&store);
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
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted_attachment_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_attachment_security);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mut attachment_security = start_attachment_security_runtime(
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
    let plaintext =
        b"clean attachment payload visible only to Blob and the loopback scanner fixture";
    let blob = blob_source.write(&store, &supervisor, &data, [81; 16], plaintext);
    let attachment = prepare_communications_attachment_for_scan(
        &store,
        "clean",
        blob.declared_size,
        blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(&store, &attachment, &blob, &clamav, plaintext);
    assert_attachment_security_source_blob_read_is_denied(
        &store,
        &supervisor,
        &data,
        &attachment_security,
        &blob,
    );
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    assert_stale_attachment_security_verdict_cas_is_rejected(&store, &attachment);
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );

    let outage_plaintext =
        b"attachment fixture-held-clean payload retained through NATS outage and relay restart";
    let outage_blob = blob_source.write(&store, &supervisor, &data, [86; 16], outage_plaintext);
    let outage_attachment = prepare_communications_attachment_for_scan(
        &store,
        "outage-restart",
        outage_blob.declared_size,
        outage_blob.receipt_sha256,
    );
    let previous_runtime_instance_id = attachment_security.runtime_instance_id.clone();
    attachment_security = assert_attachment_security_outbox_replays_after_nats_outage_and_restart(
        &store,
        &outage_attachment,
        &outage_blob,
        &clamav,
        || {
            supervisor
                .stop(&attachment_security.registration_id)
                .expect("stop Attachment Security runtime with pending verdict");
        },
        || {
            restart_attachment_security_runtime(
                &supervisor,
                &store,
                &root.join("runtime"),
                &attachment_security,
                clamav.port(),
            )
        },
    );
    assert_eq!(attachment_security.runtime_generation, 2);
    assert_ne!(
        attachment_security.runtime_instance_id,
        previous_runtime_instance_id
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            outage_attachment.attachment_anchor_id
        ),
        hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );

    let threat_plaintext =
        b"attachment fixture-threat payload visible only to Blob and scanner fixture";
    let threat_blob = blob_source.write(&store, &supervisor, &data, [82; 16], threat_plaintext);
    let threat_attachment = prepare_communications_attachment_for_scan(
        &store,
        "threat",
        threat_blob.declared_size,
        threat_blob.receipt_sha256,
    );
    assert_threat_attachment_security_verdict_flow(
        &store,
        &threat_attachment,
        &threat_blob,
        &clamav,
        threat_plaintext,
    );
    assert_eq!(
        wait_for_attachment_state(
            &store,
            &supervisor,
            threat_attachment.attachment_anchor_id
        ),
        hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::Quarantined
            as u32
    );

    for (scenario_id, blob_id, plaintext, scanner_outcome) in [
        (
            "malformed",
            [83; 16],
            b"attachment fixture-malformed response scenario".as_slice(),
            ClamAvFixtureOutcomeV1::Malformed,
        ),
        (
            "disconnect",
            [84; 16],
            b"attachment fixture-disconnect I/O scenario".as_slice(),
            ClamAvFixtureOutcomeV1::Disconnect,
        ),
        (
            "timeout",
            [85; 16],
            b"attachment fixture-timeout response scenario".as_slice(),
            ClamAvFixtureOutcomeV1::Timeout,
        ),
    ] {
        let failure_blob = blob_source.write(&store, &supervisor, &data, blob_id, plaintext);
        let failure_attachment = prepare_communications_attachment_for_scan(
            &store,
            scenario_id,
            failure_blob.declared_size,
            failure_blob.receipt_sha256,
        );
        assert_attachment_security_scanner_failure_is_fail_closed(
            &store,
            &failure_attachment,
            &failure_blob,
            &clamav,
            scanner_outcome,
        );
        assert_eq!(
            wait_for_attachment_state(
                &store,
                &supervisor,
                failure_attachment.attachment_anchor_id
            ),
            hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::BlobAdmitted
                as u32
        );
    }
    assert_eq!(clamav.outcome_count(ClamAvFixtureOutcomeV1::Clean), 1);
    assert_eq!(clamav.outcome_count(ClamAvFixtureOutcomeV1::Threat), 1);
    assert_eq!(clamav.outcome_count(ClamAvFixtureOutcomeV1::HeldClean), 1);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}
