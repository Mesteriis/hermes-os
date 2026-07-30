//! Live co-admission of Scheduler and the two independent communication workflows.

use super::*;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Scheduler, Communications, delayed-delivery and delivery-intent binaries"]
fn managed_delayed_delivery_starts_with_scheduler_and_delivery_intent() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-delayed-delivery");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_delayed_delivery_conformance_release(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            DELAYED_DELIVERY_LOGICAL_OWNER_ID,
            "desktop-1",
            [4; 65],
        ))
        .expect("claim logical browser owner");
    record_scheduler_runtime(&store);
    let delivery_intent = admit_delivery_intent_runtime(&store);
    let delayed_delivery = admit_delayed_delivery_runtime(&store);
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        hermes_gateway_runtime::InMemoryBrowserRealtimeSource::new(32).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_delivery_intent_runtime_routes(&supervisor, &store, realtime);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    blob_launch::start_from_kernel(
        &supervisor,
        &store,
        release.kernel(),
        &data,
        &root.join("runtime"),
    )
    .expect("start signed Blob runtime");
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
    issue_initial_scheduler_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &scheduler_binding(&store),
    )
    .expect("provision Scheduler Storage binding");
    let delivery_intent = prepare_delivery_intent_runtime(&supervisor, &store, delivery_intent);
    let delayed_delivery = prepare_delayed_delivery_runtime(&supervisor, &store, delayed_delivery);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let delivery_intent =
        start_delivery_intent_runtime(&supervisor, &store, &root.join("runtime"), delivery_intent);
    let delayed_delivery = start_delayed_delivery_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        delayed_delivery,
    );
    let scheduler_reservation = managed_launch::load(&supervisor, &store, SCHEDULER_REGISTRATION)
        .expect("load Scheduler reservation");
    assert_eq!(
        scheduler_launch::start_from_reservation(
            &supervisor,
            &store,
            release.kernel(),
            &root.join("runtime"),
            scheduler_reservation,
            &scheduler_binding(&store),
        )
        .expect("start Scheduler with delayed-delivery grant"),
        1
    );
    for registration_id in [
        SCHEDULER_REGISTRATION,
        COMMUNICATIONS_REGISTRATION,
        delivery_intent.registration_id.as_str(),
        delayed_delivery.registration_id.as_str(),
    ] {
        assert!(
            supervisor
                .is_active(registration_id)
                .expect("read managed runtime state"),
            "{registration_id} must stay active in the combined contour"
        );
    }
    assert_eq!(delivery_intent.runtime_generation, 1);
    assert_eq!(delayed_delivery.runtime_generation, 1);

    supervisor.shutdown().expect("stop managed processes");
    std::fs::remove_dir_all(root).expect("remove fixture");
}
