//! Live signed admission for the event-only reviewed Task candidate chain.

use super::*;

use crate::identity::device::signer::DeviceSigner;
use hermes_communication_task_candidate_api::{
    COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1, COMMUNICATION_TASK_CANDIDATE_OWNER_V1,
};
use hermes_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_MODULE_ID_V1, REVIEW_TASK_CANDIDATE_OWNER_V1,
    wire::{
        ReviewTaskCandidateDecisionV1, ReviewTaskCandidateErrorCodeV1,
        ReviewTaskCandidatePromotionStatusV1, ReviewTaskCandidateStateV1,
    },
};
use hermes_reviewed_task_candidate_promotion_core::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1, REVIEWED_TASK_CANDIDATE_PROMOTION_OWNER_V1,
};
use hermes_tasks_command_api::{TASKS_MODULE_ID_V1, TASKS_OWNER_ID_V1};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, extraction, Review and Tasks binaries"]
fn managed_task_candidate_approve_reject_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-reviewed-task-candidate");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_task_candidate_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim reviewed Task candidate logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
    );
    let admitted = admit_task_candidate_ensemble_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime = hermes_gateway_runtime::InMemoryBrowserRealtimeSource::new(64)
        .expect("reviewed Task candidate realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_task_candidate_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure reviewed Task candidate Event credential handler");
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
    let admitted = prepare_task_candidate_ensemble_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1
    );
    let mut started =
        start_task_candidate_ensemble_v1(&supervisor, &store, &root.join("runtime"), admitted);
    assert_eq!(started.len(), 4);
    assert_eq!(
        started
            .iter()
            .map(|runtime| (runtime.module_id.as_str(), runtime.owner_id.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1,
                COMMUNICATION_TASK_CANDIDATE_OWNER_V1,
            ),
            (
                REVIEW_TASK_CANDIDATE_MODULE_ID_V1,
                REVIEW_TASK_CANDIDATE_OWNER_V1,
            ),
            (
                REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1,
                REVIEWED_TASK_CANDIDATE_PROMOTION_OWNER_V1,
            ),
            (TASKS_MODULE_ID_V1, TASKS_OWNER_ID_V1),
        ]
    );
    assert!(started.iter().all(|runtime| {
        runtime.runtime_generation == 1
            && runtime.grant_epoch > 0
            && !runtime.registration_id.is_empty()
            && !runtime.runtime_instance_id.is_empty()
    }));

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Task candidate Gateway runtime");
    let router = task_candidate_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let reviews = seed_pending_task_candidate_reviews_v1(&gateway_runtime);

    let approved = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x51,
        &reviews.approved_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionApprove,
    );
    assert_eq!(
        approved.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    assert!(!approved.replayed);
    let approved_state = approved.review.expect("approved Review response");
    assert_eq!(
        approved_state.state,
        ReviewTaskCandidateStateV1::ReviewTaskCandidateStateApproved as i32
    );
    assert_eq!(
        approved_state.promotion_status,
        ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusPending as i32
    );
    assert_eq!(approved_state.review_revision, 2);

    let rejected = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x61,
        &reviews.rejected_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionReject,
    );
    assert_eq!(
        rejected.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    assert!(!rejected.replayed);
    let rejected_state = rejected.review.expect("rejected Review response");
    assert_eq!(
        rejected_state.state,
        ReviewTaskCandidateStateV1::ReviewTaskCandidateStateRejected as i32
    );
    assert_eq!(
        rejected_state.promotion_status,
        ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusNotRequested as i32
    );
    assert_eq!(rejected_state.review_revision, 2);

    let stale = decide_task_candidate_v1(
        &router,
        &gateway_runtime,
        &cookie,
        0x62,
        &reviews.rejected_review_id,
        1,
        ReviewTaskCandidateDecisionV1::ReviewTaskCandidateDecisionApprove,
    );
    assert_eq!(
        stale.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeRevisionConflict as i32
    );
    assert!(stale.review.is_none());

    let (approved_final, rejected_final) =
        wait_for_task_candidate_terminal_states_v1(&router, &gateway_runtime, &cookie, &reviews);
    assert_task_candidate_response_states_v1(&approved_final, &rejected_final);
    let terminal =
        read_task_candidate_terminal_events_v1(&router, &gateway_runtime, &cookie, &reviews);
    assert_exact_task_materialization_v1(&gateway_runtime, &reviews);
    for title in [
        b"Approved candidate title".as_slice(),
        b"Rejected candidate title",
    ] {
        for event in [&terminal.approved, &terminal.rejected] {
            assert!(
                !event
                    .encode_to_vec()
                    .windows(title.len())
                    .any(|window| window == title),
                "client realtime must not expose candidate presentation bytes"
            );
        }
    }
    let approved_cursor = terminal.approved.cursor.clone();
    let rejected_cursor = terminal.rejected.cursor.clone();

    assert!(
        realtime
            .revoke_owner(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
            .expect("clear Task candidate Gateway replay cache")
    );
    let review_position = started
        .iter()
        .position(|runtime| runtime.module_id == REVIEW_TASK_CANDIDATE_MODULE_ID_V1)
        .expect("started Review Task candidate runtime");
    let review = started.remove(review_position);
    let review =
        restart_task_candidate_runtime_v1(&supervisor, &store, &root.join("runtime"), review);
    started.insert(review_position, review);
    let restarted_router =
        task_candidate_gateway_v1(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed = read_task_candidate_terminal_events_v1(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &reviews,
    );
    assert_eq!(replayed.approved.cursor, approved_cursor);
    assert_eq!(replayed.rejected.cursor, rejected_cursor);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove reviewed Task candidate fixture");
}
