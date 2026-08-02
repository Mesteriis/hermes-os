//! Live signed Mail -> workflow -> Contacts event-only conformance.

use std::time::{Duration, Instant};

use hermes_mail_contacts_sync_api::{
    MAIL_CONTACTS_SYNC_CAPABILITY_ID_V1, MAIL_CONTACTS_SYNC_MODULE_ID_V1,
    MAIL_CONTACTS_SYNC_OWNER_ID_V1, mail_contacts_sync_query_contract_v1,
    mail_contacts_sync_start_contract_v1,
    wire::{
        GetMailContactsSyncRequestV1, GetMailContactsSyncResponseV1, MailContactsSyncDirectionV1,
        MailContactsSyncErrorCodeV1, MailContactsSyncStateV1, StartMailContactsSyncRequestV1,
        StartMailContactsSyncResponseV1,
    },
};
use hermes_mail_persistence::GmailOAuthCredentialBindingV1;
use hermes_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

use super::*;
use crate::{
    identity::device::signer::DeviceSigner,
    modules::capability::router::{ManagedCapabilityRouteRequest, route_managed_client_request},
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Mail, workflow and Contacts binaries"]
fn managed_mail_contacts_sync_reaches_contacts_through_events() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let provider = MailGmailFixture::start();
    let root = unique_target_root("hermes-managed-mail-contacts-sync");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let seeded = seed_mail_vault(&vault_dir);
    let release = installed_mail_contacts_sync_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("HERMES_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&hermes_kernel_control_store::InitialOwnerIdentity::new(
            MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Mail Contacts Sync logical owner");

    let admitted_mail = admit_mail_google_people_runtime(&store);
    let admitted_contacts = admit_contacts_runtime_v1(&store);
    let admitted_sync = admit_mail_contacts_sync_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        hermes_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_mail_contacts_sync_realtime_v1(&supervisor, &store, realtime);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Mail Contacts Sync Event credentials");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    let admitted_contacts = prepare_contacts_runtime_v1(&supervisor, &store, admitted_contacts);
    let admitted_sync = prepare_mail_contacts_sync_runtime_v1(&supervisor, &store, admitted_sync);
    configure_communications_jetstream(&store);

    let mail = start_mail_google_people_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        MailGmailFixtureSettingsV1 {
            port: provider.port(),
            ca_certificate_pem: provider.ca_certificate_pem().to_owned(),
            oauth: None,
        },
    );
    wait_for_mail_ready(&supervisor, &mail);
    let contacts = start_contacts_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_contacts,
    );
    let sync = start_mail_contacts_sync_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_sync,
    );
    assert_eq!(contacts.runtime_generation, 1);
    assert_eq!(sync.runtime_generation, 1);
    assert!(sync.grant_epoch > 0);
    assert!(!sync.runtime_instance_id.is_empty());

    let runtime = tokio::runtime::Runtime::new().expect("Mail Contacts Sync conformance runtime");
    let _runtime_context = runtime.enter();
    let durable = runtime.block_on(connect_postgres());
    runtime
        .block_on(durable.initialize())
        .expect("initialize Mail persistence");
    let binding: GmailOAuthCredentialBindingV1 = seeded.contacts_binding();
    runtime
        .block_on(durable.store_gmail_oauth_credential_binding(MAIL_ACCOUNT_ID, &binding, 1))
        .expect("store contacts-authorized Gmail binding");

    let request = StartMailContactsSyncRequestV1 {
        protocol_major: 1,
        operation_id: vec![0x81; 16],
        account_id: MAIL_ACCOUNT_ID.to_owned(),
        direction: MailContactsSyncDirectionV1::MailContactsSyncDirectionProviderToContacts as i32,
    };
    let accepted = route_start(&store, &supervisor, &sync.registration_id, 1, &request);
    assert_eq!(
        accepted.error,
        MailContactsSyncErrorCodeV1::MailContactsSyncErrorCodeUnspecified as i32
    );
    assert_eq!(accepted.run_id.len(), 16);
    let completed =
        wait_for_completed(&store, &supervisor, &sync.registration_id, &accepted.run_id);
    assert_eq!(completed.account_id, MAIL_ACCOUNT_ID);
    assert_eq!(completed.provider_entries_seen, 1);
    assert_eq!(completed.contacts_created, 1);
    assert_eq!(completed.contacts_updated, 0);
    assert_eq!(completed.contacts_unchanged, 0);
    assert_eq!(completed.rejected_entries, 0);
    assert_eq!(provider.accepted_people_reads(), 1);

    runtime.block_on(async {
        let pool = contacts_admin_pool_v1().await;
        let contacts_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM hermes_data.contacts_state WHERE logical_owner_id=$1",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .fetch_one(&pool)
        .await
        .expect("count synced Contacts state");
        let completed_inbox: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM hermes_data.contacts_mail_entry_inbox
             WHERE logical_owner_id=$1 AND completed=TRUE",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .fetch_one(&pool)
        .await
        .expect("count synced Contacts inbox");
        assert_eq!(contacts_count, 1);
        assert_eq!(completed_inbox, 1);
        pool.close().await;
    });

    let duplicate = route_start(&store, &supervisor, &sync.registration_id, 3, &request);
    assert_eq!(duplicate.run_id, accepted.run_id);
    let duplicate_completed =
        wait_for_completed(&store, &supervisor, &sync.registration_id, &accepted.run_id);
    assert_eq!(duplicate_completed.state_revision, completed.state_revision);
    assert_eq!(provider.accepted_people_reads(), 1);

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail Contacts Sync fixture");
    std::fs::remove_dir_all(data).expect("remove Mail Contacts Sync Kernel fixture");
}

fn route_start(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    request_id: u64,
    request: &StartMailContactsSyncRequestV1,
) -> StartMailContactsSyncResponseV1 {
    let response = route_sync_request(
        store,
        supervisor,
        registration_id,
        request_id,
        mail_contacts_sync_start_contract_v1(),
        request.encode_to_vec(),
    );
    StartMailContactsSyncResponseV1::decode(response.response_payload.as_slice())
        .expect("decode Mail Contacts Sync Start response")
}

fn wait_for_completed(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    run_id: &[u8],
) -> GetMailContactsSyncResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut request_id = 10;
    loop {
        let request = GetMailContactsSyncRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        };
        let response = route_sync_request(
            store,
            supervisor,
            registration_id,
            request_id,
            mail_contacts_sync_query_contract_v1(),
            request.encode_to_vec(),
        );
        let response = GetMailContactsSyncResponseV1::decode(response.response_payload.as_slice())
            .expect("decode Mail Contacts Sync Get response");
        match MailContactsSyncStateV1::try_from(response.state) {
            Ok(MailContactsSyncStateV1::MailContactsSyncStateCompleted) => return response,
            Ok(MailContactsSyncStateV1::MailContactsSyncStateRejected) => {
                panic!("Mail Contacts Sync rejected: {response:?}")
            }
            _ => {}
        }
        assert!(
            Instant::now() < deadline,
            "Mail Contacts Sync did not complete: {response:?}"
        );
        request_id += 1;
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn route_sync_request(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
    request_id: u64,
    contract: ContractReferenceV1,
    request_payload: Vec<u8>,
) -> ModuleClientResponseV1 {
    let launch = store
        .effective_managed_launch_record(registration_id)
        .expect("read Mail Contacts Sync launch")
        .expect("Mail Contacts Sync launch is active");
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: MAIL_CONTACTS_SYNC_MODULE_ID_V1.to_owned(),
        owner_id: MAIL_CONTACTS_SYNC_OWNER_ID_V1.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload,
        logical_owner_id: MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
    }
    .encode_to_vec();
    let route = ManagedCapabilityRouteRequest::new(
        registration_id,
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
        MAIL_CONTACTS_SYNC_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = route_managed_client_request(store, &supervisor.relay_port(), &route)
        .expect("route Mail Contacts Sync client request");
    ModuleClientResponseV1::decode(bytes.as_slice())
        .expect("decode Mail Contacts Sync module response")
}

async fn contacts_admin_pool_v1() -> sqlx::PgPool {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "HERMES_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let options = PgConnectOptions::new()
        .host(&required("HERMES_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
        .port(
            required("HERMES_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                .parse()
                .expect("valid PostgreSQL port"),
        )
        .username("hermes_postgres_admin")
        .password(password.as_str())
        .database("hermes_storage_authenticated")
        .ssl_mode(PgSslMode::Disable);
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect Mail Contacts Sync conformance database")
}
