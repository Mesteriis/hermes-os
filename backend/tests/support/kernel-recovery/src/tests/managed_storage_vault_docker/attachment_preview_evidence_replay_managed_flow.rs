//! Live signed admission smoke gate for retained Preview evidence replay.

use super::*;

use super::{
    attachment_preview_evidence_replay_persistence_fixture::{
        wait_for_retained_preview_evidence_message_ids_v1,
        wait_for_retained_preview_replay_terminal_v1,
    },
    attachment_preview_gateway_fixture::{
        attachment_preview_gateway_v1, get_attachment_preview_v1, post_attachment_preview_proto_v1,
        read_attachment_preview_blob_v1, read_terminal_attachment_preview_sse_event_v1,
        wait_for_ready_attachment_preview_v1,
    },
    attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture,
    mail_attachment_flow::assert_mail_attachment_lifecycle,
};

use crate::identity::device::signer::DeviceSigner;
use hermes_attachment_preview_api::{
    ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1, ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
    wire::{
        AttachmentPreviewErrorCodeV1, AttachmentPreviewStateV1,
        IssueAttachmentPreviewReadRequestV1, IssueAttachmentPreviewReadResponseV1,
        StartAttachmentPreviewRequestV1, StartAttachmentPreviewResponseV1,
    },
};
use hermes_attachment_preview_evidence_replay_api::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1,
    wire::{
        AttachmentPreviewEvidenceReplayErrorV1, AttachmentPreviewEvidenceReplayStateV1,
        ReplayProducerSelectionV1, StartAttachmentPreviewEvidenceReplayRequestV1,
        StartAttachmentPreviewEvidenceReplayResponseV1,
    },
};
use hyper::StatusCode;

const SAFETY_STATE_SUBJECT_V1: &str =
    "hermes.event.v1.communications.communication_attachment_safety_state_changed.v1";
const SCAN_CANDIDATE_SUBJECT_V1: &str = "hermes.observation.v1.attachment_security.\
    attachment_security_scan_candidate_observed.v1";

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

#[test]
#[ignore = "requires disposable Docker plus the complete retained Preview evidence replay managed ensemble"]
fn managed_attachment_preview_evidence_replay_restores_expired_sources_to_browser_preview() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("hermes-managed-attachment-preview-evidence-recovery");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_mail_vault(&vault_dir);
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
        .expect("claim retained Preview recovery logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
    );

    let admitted_mail = admit_mail_runtime(&store);
    let admitted_security = admit_attachment_security_runtime(&store);
    let admitted_preview = admit_attachment_preview_runtime_v1(&store);
    let admitted_replay = admit_attachment_preview_evidence_replay_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        hermes_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_attachment_preview_realtime_v1(&supervisor, &store, realtime.clone());
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
    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    let admitted_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_security);
    let admitted_preview =
        prepare_attachment_preview_runtime_v1(&supervisor, &store, admitted_preview);
    let admitted_replay =
        prepare_attachment_preview_evidence_replay_runtime_v1(&supervisor, &store, admitted_replay);
    configure_communications_jetstream_for_retained_replay_test(&store);
    let communications_generation =
        start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let mail = start_mail_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        imap.port(),
    );
    let _replay = start_attachment_preview_evidence_replay_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_replay,
    );

    let attachment_anchor_id = assert_mail_attachment_lifecycle(&store, &supervisor, &mail);
    let _security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_security,
        clamav.port(),
    );
    wait_for_retained_preview_attachment_state_v1(
        &store,
        &supervisor,
        attachment_anchor_id,
        hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32,
    );
    let message_ids = wait_for_retained_preview_evidence_message_ids_v1(attachment_anchor_id);
    wait_for_communications_jetstream_subject_expiry(&store, SAFETY_STATE_SUBJECT_V1);
    wait_for_communications_jetstream_subject_expiry(&store, SCAN_CANDIDATE_SUBJECT_V1);

    let _preview = start_attachment_preview_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_preview,
    );
    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = attachment_preview_gateway_v1(&store, &supervisor, &root, &data, realtime);
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let preview = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            protocol_major: 1,
            operation_id: vec![0xD1; 16],
            attachment_anchor_id: attachment_anchor_id.to_vec(),
        },
    );
    assert_eq!(
        preview.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    assert_eq!(preview.state, AttachmentPreviewStateV1::Accepted as i32);
    assert_eq!(
        get_attachment_preview_v1(&router, &gateway_runtime, &cookie, &preview.run_id,).state,
        AttachmentPreviewStateV1::Accepted as i32
    );

    let replay_operation_id = [0xD2; 16];
    let replay = post_attachment_preview_proto_v1::<
        _,
        StartAttachmentPreviewEvidenceReplayResponseV1,
    >(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONNECT_PATH_V1,
        StartAttachmentPreviewEvidenceReplayRequestV1 {
            protocol_major: 1,
            operation_id: replay_operation_id.to_vec(),
            attachment_anchor_id: attachment_anchor_id.to_vec(),
            communications: Some(ReplayProducerSelectionV1 {
                producer_registration_id: COMMUNICATIONS_REGISTRATION.to_owned(),
                producer_runtime_generation: communications_generation,
                producer_grant_epoch: producer_grant_epoch_v1(&store, COMMUNICATIONS_REGISTRATION),
                original_message_ids: vec![message_ids.communications.to_vec()],
            }),
            mail: Some(ReplayProducerSelectionV1 {
                producer_registration_id: mail.registration_id.clone(),
                producer_runtime_generation: mail.runtime_generation,
                producer_grant_epoch: mail.grant_epoch,
                original_message_ids: vec![message_ids.mail.to_vec()],
            }),
        },
    );
    assert_eq!(
        replay.error,
        AttachmentPreviewEvidenceReplayErrorV1::Unspecified as i32
    );
    assert!(
        replay.state == AttachmentPreviewEvidenceReplayStateV1::Accepted as i32
            || replay.state == AttachmentPreviewEvidenceReplayStateV1::AwaitingProducers as i32,
        "replay start must return an accepted or already-dispatched operation"
    );
    let replay_diagnostics = wait_for_retained_preview_replay_terminal_v1(replay_operation_id);
    assert_eq!(
        replay_diagnostics.state,
        AttachmentPreviewEvidenceReplayStateV1::Completed as i16
    );
    assert_eq!(replay_diagnostics.error, 0);
    assert_eq!(replay_diagnostics.producer_results, 2);
    assert_eq!(replay_diagnostics.communications_published_audits, 1);
    assert_eq!(replay_diagnostics.mail_published_audits, 1);

    let ready = wait_for_ready_attachment_preview_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &preview.run_id,
        "retained evidence recovery",
    );
    assert_eq!(ready.state, AttachmentPreviewStateV1::Ready as i32);
    let event = read_terminal_attachment_preview_sse_event_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &preview.run_id,
    );
    assert!(!event.payload.is_empty());
    let ticket = post_attachment_preview_proto_v1::<_, IssueAttachmentPreviewReadResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
        IssueAttachmentPreviewReadRequestV1 {
            protocol_major: 1,
            run_id: preview.run_id,
        },
    );
    assert_eq!(
        ticket.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    let (status, body) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&cookie),
        ticket.opaque_read_ticket,
    );
    assert_eq!(status, StatusCode::OK);
    assert!(!body.is_empty());

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove retained Preview recovery fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn producer_grant_epoch_v1(store: &SqliteControlStore, registration_id: &str) -> u64 {
    store
        .module_grant_snapshot(registration_id)
        .expect("read replay producer grant snapshot")
        .expect("replay producer grant snapshot")
        .effective_grants()
        .expect("approved replay producer grants")
        .grant_epoch()
}

fn wait_for_retained_preview_attachment_state_v1(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    attachment_anchor_id: [u8; 16],
    expected_state: u32,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if wait_for_attachment_state(store, supervisor, attachment_anchor_id) == expected_state {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "retained Preview attachment did not reach the expected safety state"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
