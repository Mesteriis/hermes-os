//! Live signed admission for the event-only reviewed Task candidate chain.

use super::*;

use crate::identity::device::signer::DeviceSigner;
use hermes_communication_task_candidate_api::{
    COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1, COMMUNICATION_TASK_CANDIDATE_OWNER_V1,
};
use hermes_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_MODULE_ID_V1, REVIEW_TASK_CANDIDATE_OWNER_V1,
};
use hermes_reviewed_task_candidate_promotion_core::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1, REVIEWED_TASK_CANDIDATE_PROMOTION_OWNER_V1,
};
use hermes_tasks_command_api::{TASKS_MODULE_ID_V1, TASKS_OWNER_ID_V1};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, extraction, Review and Tasks binaries"]
fn managed_task_candidate_chain_starts_from_one_signed_release() {
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
    configure_task_candidate_realtime_v1(&supervisor, &store, realtime);
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
    let started =
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

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove reviewed Task candidate fixture");
}
