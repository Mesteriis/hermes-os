//! Full managed Preview custody, Gateway, client Blob, SSE and restart conformance.

use super::*;
use super::{
    attachment_preview_gateway_fixture::{
        attachment_preview_gateway_v1, get_attachment_preview_v1,
        post_attachment_preview_proto_status_v1, post_attachment_preview_proto_v1,
        read_attachment_preview_blob_v1, read_terminal_attachment_preview_sse_event_v1,
        wait_for_ready_attachment_preview_v1,
    },
    attachment_preview_managed_formats::assert_managed_attachment_preview_formats_v1,
    attachment_security_blob_fixture::AttachmentSecurityBlobSourceFixture,
    attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture,
    attachment_security_event_flow::{
        assert_clean_attachment_security_verdict_flow, prepare_communications_attachment_for_scan,
    },
    mail_attachment_flow::wait_for_attachment_state,
};

use crate::identity::device::signer::DeviceSigner;
use hermes_attachment_preview_api::{
    ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1, ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
    wire::{
        AttachmentPreviewContentTypeV1, AttachmentPreviewErrorCodeV1, AttachmentPreviewKindV1,
        AttachmentPreviewStateV1, AttachmentPreviewStatusChangedV1,
        IssueAttachmentPreviewReadRequestV1, IssueAttachmentPreviewReadResponseV1,
        StartAttachmentPreviewRequestV1, StartAttachmentPreviewResponseV1,
    },
};
use hyper::StatusCode;

const PRIVATE_SOURCE: &[u8] =
    b"Private clean-room preview payload.\r\nThe bytes must stay outside query and SSE.";
const EXPECTED_PREVIEW: &[u8] =
    b"Private clean-room preview payload.\nThe bytes must stay outside query and SSE.";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, Attachment Security and Preview binaries"]
fn managed_attachment_preview_reaches_gateway_blob_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("hermes-managed-attachment-preview");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_attachment_preview_ensemble_release_v1(&root);
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
        .expect("claim Attachment Preview logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
    );
    super::super::browser_gateway_session::admit_secondary_browser_test_device(
        &store,
        ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1,
    );

    let admitted_preview = admit_attachment_preview_runtime_v1(&store);
    let admitted_security = admit_attachment_security_runtime(&store);
    let blob_source = AttachmentSecurityBlobSourceFixture::admit(&store);
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
    let admitted_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_security);
    let admitted_preview =
        prepare_attachment_preview_runtime_v1(&supervisor, &store, admitted_preview);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_security,
        clamav.port(),
    );
    let preview = start_attachment_preview_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_preview,
    );
    assert_eq!(security.runtime_generation, 1);
    assert_eq!(preview.runtime_generation, 1);

    let blob = blob_source.write(&store, &supervisor, &data, [0xA1; 16], PRIVATE_SOURCE);
    let attachment = prepare_communications_attachment_for_scan(
        &store,
        "attachment-preview-text",
        blob.declared_size,
        blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(
        &store,
        &attachment,
        &blob,
        &clamav,
        PRIVATE_SOURCE,
    );
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        hermes_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    wait_for_attachment_preview_evidence_v1();

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = attachment_preview_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let request = StartAttachmentPreviewRequestV1 {
        protocol_major: 1,
        operation_id: vec![0xA2; 16],
        attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
    };
    set_authenticated_nats_container_running(false);
    let accepted = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(
        accepted.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    assert_eq!(accepted.run_id.len(), 16);
    wait_for_pending_attachment_preview_custody_outbox_v1();
    set_authenticated_nats_container_running(true);

    let ready = wait_for_ready_attachment_preview_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
        "text",
    );
    assert_eq!(ready.attachment_anchor_id, attachment.attachment_anchor_id);
    assert_eq!(ready.state, AttachmentPreviewStateV1::Ready as i32);
    assert_eq!(ready.preview_kind, AttachmentPreviewKindV1::Text as i32);
    assert_eq!(
        ready.content_type,
        AttachmentPreviewContentTypeV1::TextUtf8 as i32
    );
    assert_eq!(ready.preview_size_bytes, EXPECTED_PREVIEW.len() as u64);
    assert!(!ready.truncated);
    assert!(
        !ready
            .encode_to_vec()
            .windows(PRIVATE_SOURCE.len())
            .any(|window| window == PRIVATE_SOURCE)
    );

    let first_event = read_terminal_attachment_preview_sse_event_v1(
        &router,
        &gateway_runtime,
        &cookie,
        &accepted.run_id,
    );
    let first_payload = AttachmentPreviewStatusChangedV1::decode(first_event.payload.as_slice())
        .expect("Attachment Preview realtime payload");
    assert_eq!(first_payload.state, AttachmentPreviewStateV1::Ready as i32);
    assert!(
        !first_event
            .encode_to_vec()
            .windows(PRIVATE_SOURCE.len())
            .any(|window| window == PRIVATE_SOURCE)
    );
    let first_cursor = first_event.cursor.clone();
    let completed = attachment_preview_diagnostics_v1();
    assert_eq!(
        completed,
        AttachmentPreviewDiagnosticsV1 {
            candidates: 1,
            safety_facts: 1,
            custody_requests: 1,
            pending_custody_outbox: 0,
            custody_results: 1,
            jobs: 1,
            attempts: 1,
            artifacts: 1,
            security_delegation_commands: 1,
            security_delegation_attempts: 1,
            security_delegation_results: 1,
        }
    );
    let duplicate = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(duplicate.state, AttachmentPreviewStateV1::Ready as i32);
    assert_eq!(attachment_preview_diagnostics_v1(), completed);
    let conflicting = post_attachment_preview_proto_v1::<_, StartAttachmentPreviewResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
        StartAttachmentPreviewRequestV1 {
            attachment_anchor_id: vec![0xA3; 16],
            ..request
        },
    );
    assert_eq!(
        conflicting.error,
        AttachmentPreviewErrorCodeV1::InvalidRequest as i32
    );
    assert_eq!(attachment_preview_diagnostics_v1(), completed);

    let ticket = post_attachment_preview_proto_v1::<_, IssueAttachmentPreviewReadResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
        IssueAttachmentPreviewReadRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id.clone(),
        },
    );
    assert_eq!(
        ticket.error,
        AttachmentPreviewErrorCodeV1::Unspecified as i32
    );
    assert_eq!(ticket.opaque_read_ticket.len(), 32);
    let secondary_cookie =
        super::super::browser_gateway_session::authenticate_secondary_gateway_router(
            &router,
            &gateway_runtime,
        );
    let (wrong_actor_status, wrong_actor_body) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&secondary_cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(wrong_actor_status, StatusCode::NOT_FOUND);
    assert!(wrong_actor_body.is_empty());
    let (status, body) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&cookie),
        ticket.opaque_read_ticket.clone(),
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, EXPECTED_PREVIEW);
    let (replay_status, _) = read_attachment_preview_blob_v1(
        &router,
        &gateway_runtime,
        Some(&cookie),
        ticket.opaque_read_ticket,
    );
    assert_eq!(replay_status, StatusCode::NOT_FOUND);

    assert_managed_attachment_preview_formats_v1(
        &store,
        &supervisor,
        &data,
        &blob_source,
        &clamav,
        &router,
        &gateway_runtime,
        &cookie,
    );

    assert!(
        realtime
            .revoke_owner(ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1)
            .expect("clear Attachment Preview Gateway replay cache")
    );
    let previous_generation = preview.runtime_generation;
    let preview =
        restart_attachment_preview_runtime_v1(&supervisor, &store, &root.join("runtime"), preview);
    assert_eq!(preview.runtime_generation, previous_generation + 1);
    let restarted_router =
        attachment_preview_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    assert_eq!(
        get_attachment_preview_v1(
            &restarted_router,
            &gateway_runtime,
            &restarted_cookie,
            &accepted.run_id,
        ),
        ready
    );
    let replayed_event = read_terminal_attachment_preview_sse_event_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_cursor);
    assert_eq!(replayed_event.payload, first_event.payload);

    let unauthenticated = post_attachment_preview_proto_status_v1(
        &restarted_router,
        &gateway_runtime,
        None,
        ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
        IssueAttachmentPreviewReadRequestV1 {
            protocol_major: 1,
            run_id: accepted.run_id,
        },
    );
    assert_eq!(unauthenticated, StatusCode::UNAUTHORIZED);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn wait_for_attachment_preview_evidence_v1() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.candidates == 1 && diagnostics.safety_facts == 1 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Preview did not consume source evidence: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn wait_for_pending_attachment_preview_custody_outbox_v1() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let diagnostics = attachment_preview_diagnostics_v1();
        if diagnostics.custody_requests == 1 && diagnostics.pending_custody_outbox == 1 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Attachment Preview did not retain its custody command during NATS outage: {diagnostics:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}
