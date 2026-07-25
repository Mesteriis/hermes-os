//! Live managed Telegram process through Kernel leases into managed Communications.

use super::*;

use hermes_events_protocol::validation::envelope::decode_envelope_v1;
use hermes_telegram_api::{
    TelegramClientRequest, TelegramClientResponse, client_contract::TelegramClientContractV1,
};
use hermes_telegram_runtime::client_port::{decode_module_response, encode_module_request};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_runtime_uses_kernel_leases_and_event_only_communications_handoff() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-telegram-runtime");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_telegram_vault(&vault_dir);
    let release = installed_communications_telegram_release(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            [4; 65],
        ))
        .expect("claim initial owner");
    let admitted_telegram = admit_telegram_runtime(&store);
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
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
    let admitted_telegram = prepare_telegram_runtime(&supervisor, &store, admitted_telegram);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));

    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("Event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect event observer");
        let observations = client
            .subscribe("hermes.observation.v1.communications.communication_observed.v1")
            .await
            .expect("subscribe Telegram observations");
        let canonical_events = client
            .subscribe("hermes.event.v1.communications.communication_evidence_recorded.v1")
            .await
            .expect("subscribe canonical Communications events");
        (observations, canonical_events)
    });

    let telegram = start_telegram_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_telegram,
    );
    assert_telegram_lifecycle_query(&store, &supervisor, &telegram);

    let (observation, canonical) = event_runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .expect("managed Telegram observation timeout")
            .expect("managed Telegram observation");
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("canonical Communications event timeout")
            .expect("canonical Communications event");
        (observation, canonical)
    });
    let observation =
        decode_envelope_v1(observation.payload.as_ref()).expect("Telegram observation envelope");
    assert_eq!(
        observation
            .source
            .expect("Telegram observation source")
            .module_id,
        hermes_telegram_runtime::PACKAGE
    );
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications event envelope");
    assert_eq!(
        canonical.causation_message_id, observation.message_id,
        "Communications must derive canonical evidence only from the typed Telegram observation"
    );
    assert_communications_query_delivery(&store, &supervisor);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn assert_telegram_lifecycle_query(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&telegram.registration_id) != Ok(true) {
        assert!(
            std::time::Instant::now() < deadline,
            "managed Telegram runtime did not become ready: {:?}",
            supervisor.last_failure(&telegram.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    let request = encode_module_request(71, &TelegramClientRequest::ListAccounts)
        .expect("encode Telegram lifecycle query");
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &telegram.registration_id,
        &telegram.runtime_instance_id,
        telegram.runtime_generation,
        telegram.grant_epoch,
        TelegramClientContractV1::Lifecycle.capability_id(),
        &request,
    );
    loop {
        let last_error = match crate::modules::capability::router::route_managed_client_request(
            store, &relay, &route,
        ) {
            Ok(bytes) => {
                match decode_module_response(TelegramClientContractV1::Lifecycle, &bytes) {
                    Ok((request_id, TelegramClientResponse::Accounts(accounts))) => {
                        assert_eq!(request_id, 71);
                        assert!(
                            accounts
                                .iter()
                                .any(|account| account.account_id == TELEGRAM_ACCOUNT_ID)
                        );
                        return;
                    }
                    Ok(_) => "Telegram returned the wrong response type".to_owned(),
                    Err(error) => format!("decode Telegram lifecycle response: {error:?}"),
                }
            }
            Err(error) => format!("route Telegram lifecycle query: {error}"),
        };
        assert!(
            std::time::Instant::now() < deadline,
            "managed Telegram lifecycle query is unavailable: {last_error}; child failure: {:?}",
            supervisor.last_failure(&telegram.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
