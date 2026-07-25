//! Live managed Telegram process through Kernel leases into managed Communications.

use super::*;

use hermes_events_protocol::validation::envelope::decode_envelope_v1;
use hermes_telegram_api::{
    TelegramClientRequest, TelegramClientResponse, TelegramOperationState, TelegramProviderCommand,
    TelegramProviderQuery, TelegramProviderQueryResponse, TelegramRuntimeState,
    TelegramSendMessage,
    client_contract::TelegramClientContractV1,
};
use hermes_telegram_runtime::client_port::{
    TelegramClientPortError, decode_module_response, encode_module_request,
};

#[derive(Debug)]
enum TelegramClientRouteError {
    Kernel(String),
    Client(TelegramClientPortError),
}

impl TelegramClientRouteError {
    fn is_retryable(&self) -> bool {
        match self {
            Self::Client(TelegramClientPortError::Protocol(code)) => code == "RUNTIME_BUSY",
            Self::Kernel(error) => matches!(
                error.as_str(),
                "managed runtime V2 relay response is invalid"
                    | "managed runtime relay timed out"
                    | "managed runtime relay is unavailable"
            ),
            Self::Client(_) => false,
        }
    }
}

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
    assert_telegram_account_started(&store, &supervisor, &telegram);

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
    let observation_bytes = observation.payload.to_vec();
    let observation =
        decode_envelope_v1(&observation_bytes).expect("Telegram observation envelope");
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
    event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect duplicate observation publisher");
        client
            .publish(
                "hermes.observation.v1.communications.communication_observed.v1",
                observation_bytes.into(),
            )
            .await
            .expect("republish exact Telegram observation");
        client.flush().await.expect("flush duplicate observation");
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate Telegram observation must not create a second Communications event"
        );
    });
    assert_communications_query_delivery(&store, &supervisor);
    assert_telegram_command_completion(&store, &supervisor, &telegram);

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn route_telegram_client(
    store: &SqliteControlStore,
    relay: &crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort,
    telegram: &StartedTelegramRuntime,
    contract: TelegramClientContractV1,
    request_id: u64,
    request: &TelegramClientRequest,
) -> Result<TelegramClientResponse, TelegramClientRouteError> {
    let request =
        encode_module_request(request_id, request).map_err(TelegramClientRouteError::Client)?;
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &telegram.registration_id,
        &telegram.runtime_instance_id,
        telegram.runtime_generation,
        telegram.grant_epoch,
        contract.capability_id(),
        &request,
    );
    let bytes =
        crate::modules::capability::router::route_managed_client_request(store, relay, &route)
            .map_err(TelegramClientRouteError::Kernel)?;
    let (response_request_id, response) =
        decode_module_response(contract, &bytes).map_err(TelegramClientRouteError::Client)?;
    if response_request_id != request_id {
        return Err(TelegramClientRouteError::Kernel(format!(
            "Telegram response request ID mismatch: expected {request_id}, got {response_request_id}"
        )));
    }
    Ok(response)
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
    loop {
        let last_error = match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Lifecycle,
            71,
            &TelegramClientRequest::ListAccounts,
        ) {
            Ok(TelegramClientResponse::Accounts(accounts)) => {
                assert!(
                    accounts
                        .iter()
                        .any(|account| account.account_id == TELEGRAM_ACCOUNT_ID)
                );
                return;
            }
            Ok(_) => "Telegram returned the wrong lifecycle response type".to_owned(),
            Err(error) => format!("{error:?}"),
        };
        assert!(
            std::time::Instant::now() < deadline,
            "managed Telegram lifecycle query is unavailable: {last_error}; child failure: {:?}",
            supervisor.last_failure(&telegram.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_telegram_account_started(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let now_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Telegram lifecycle clock")
        .as_secs();
    let request = TelegramClientRequest::StartAccount {
        account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
        topology: "managed-local".to_owned(),
        holder: telegram.runtime_instance_id.clone(),
        expires_at_unix_seconds: now_unix_seconds.saturating_add(60),
        now_unix_seconds,
    };
    loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Lifecycle,
            72,
            &request,
        ) {
            Ok(TelegramClientResponse::Account(account)) => {
                assert_eq!(account.runtime_state, TelegramRuntimeState::Running);
                return;
            }
            Ok(_) => panic!("Telegram lifecycle start returned the wrong response type"),
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram lifecycle start remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram lifecycle start failed: {error:?}"),
        }
    }
}

fn assert_telegram_command_completion(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) {
    const OPERATION_ID: &str = "managed-telegram-send-1";

    let relay = supervisor.relay_port();
    let command =
        TelegramClientRequest::Command(TelegramProviderCommand::SendText(TelegramSendMessage {
            operation_id: OPERATION_ID.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            provider_chat_id: "9001".to_owned(),
            text: "managed Telegram command".to_owned(),
        }));
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let response = loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Command,
            73,
            &command,
        ) {
            Ok(response) => break response,
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram command route remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram command route failed: {error:?}"),
        }
    };
    let TelegramClientResponse::Operation(operation) = response else {
        panic!("Telegram command returned the wrong response type");
    };
    assert_eq!(operation.operation_id, OPERATION_ID);
    assert_eq!(
        operation.state,
        TelegramOperationState::Accepted,
        "accepted receipt is distinct from provider completion"
    );

    loop {
        let response = match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Query,
            74,
            &TelegramClientRequest::Query(TelegramProviderQuery::Operations {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                limit: 16,
            }),
        ) {
            Ok(response) => response,
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram operation query remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
                continue;
            }
            Err(error) => panic!("Telegram operation query failed: {error:?}"),
        };
        let TelegramClientResponse::Query(TelegramProviderQueryResponse::Operations(operations)) =
            response
        else {
            panic!("Telegram operation query returned the wrong response type");
        };
        if let Some(operation) = operations
            .iter()
            .find(|operation| operation.operation_id == OPERATION_ID)
        {
            match operation.state {
                TelegramOperationState::Completed => return,
                TelegramOperationState::Failed | TelegramOperationState::DeadLetter => {
                    panic!("Telegram provider command reached a failure terminal state")
                }
                _ => {}
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Telegram provider command did not reach a terminal result"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
