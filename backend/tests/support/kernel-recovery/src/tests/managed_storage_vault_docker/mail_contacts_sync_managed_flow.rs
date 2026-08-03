//! Live signed Mail -> workflow -> Contacts event-only conformance.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hermes_contacts_mail_sync_source_api::{
    ContactsMailSyncSourceEnvelopeContextV1, build_contact_changed_for_mail_sync_outbox_record_v1,
    wire::ContactChangedForMailSyncV1,
};
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
    SchedulerRuntimeControlRequestV1, SchedulerRuntimeControlResponseV1,
    SchedulerScheduleUpsertOutcomeV1, UpsertSchedulerScheduleRequestV1,
    scheduler_runtime_control_request_v1::Operation as SchedulerOperation,
    scheduler_runtime_control_response_v1::Result as SchedulerResult,
};
use hermes_scheduler_protocol::{
    MisfirePolicyV1, OverlapPolicyV1, RetryPolicyV1, SCHEDULER_JOB_DESCRIPTOR_SET_V1,
    SchedulePolicyV1, ScheduleTriggerV1,
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
    record_scheduler_runtime_for_mail_contacts_sync(&store);
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
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start Blob for Mail Contacts Sync"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_scheduler_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &scheduler_binding(&store),
    )
    .expect("provision Scheduler Storage binding");
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
        .expect("start Scheduler for Mail Contacts Sync"),
        1
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
        direction: MailContactsSyncDirectionV1::MailContactsSyncDirectionBidirectional as i32,
    };
    let accepted = route_start(&store, &supervisor, &sync.registration_id, 1, &request);
    assert_eq!(
        accepted.error,
        MailContactsSyncErrorCodeV1::MailContactsSyncErrorCodeUnspecified as i32
    );
    assert_eq!(accepted.run_id.len(), 16);
    if !wait_for_people_write(&provider, 1) {
        let diagnostic = runtime.block_on(reverse_diagnostic());
        panic!("Mail did not execute the expected Google People write: {diagnostic:?}");
    }
    let write = provider.last_people_write();
    assert_eq!(write.method, "PATCH");
    assert!(
        write
            .path
            .starts_with("/v1/people/managed-contact-1:updateContact?")
    );
    assert_eq!(
        write.authorization,
        "Bearer managed-mail-gmail-access-token"
    );
    assert_eq!(
        write.body["metadata"]["sources"][0]["etag"],
        "managed-etag-1"
    );
    assert_eq!(
        write.body["names"][0]["displayName"],
        "Private Managed Contact"
    );
    runtime.block_on(wait_for_reverse_terminal(
        3,
        &supervisor,
        &sync.registration_id,
    ));
    let completed =
        wait_for_completed(&store, &supervisor, &sync.registration_id, &accepted.run_id);
    assert_eq!(completed.account_id, MAIL_ACCOUNT_ID);
    assert_eq!(completed.provider_entries_seen, 1);
    assert_eq!(completed.contacts_created, 1);
    assert_eq!(completed.contacts_updated, 0);
    assert_eq!(completed.contacts_unchanged, 0);
    assert_eq!(completed.provider_entries_written, 1);
    assert_eq!(completed.rejected_entries, 0);
    assert_eq!(provider.accepted_people_reads(), 1);

    let local_contact_id = [0xa1; 16];
    runtime.block_on(queue_local_contact_change(
        local_contact_id,
        1,
        "Local Create Contact",
        contacts.runtime_generation,
    ));
    assert!(wait_for_people_write(&provider, 2));
    let create = provider.last_people_write();
    assert_eq!(create.method, "POST");
    assert!(create.path.starts_with("/v1/people:createContact?"));
    assert_eq!(
        create.body["names"][0]["displayName"],
        "Local Create Contact"
    );
    runtime.block_on(wait_for_reverse_contact_terminal(local_contact_id, 1, 3));
    runtime.block_on(assert_provider_link(
        local_contact_id,
        "people/created-contact-1",
        "created-etag-1",
    ));

    runtime.block_on(queue_local_contact_change(
        local_contact_id,
        2,
        "Local Updated Contact",
        contacts.runtime_generation,
    ));
    assert!(wait_for_people_write(&provider, 3));
    let update = provider.last_people_write();
    assert_eq!(update.method, "PATCH");
    assert!(
        update
            .path
            .starts_with("/v1/people/created-contact-1:updateContact?")
    );
    assert_eq!(
        update.body["metadata"]["sources"][0]["etag"],
        "created-etag-1"
    );
    runtime.block_on(wait_for_reverse_contact_terminal(local_contact_id, 2, 3));
    runtime.block_on(assert_provider_link(
        local_contact_id,
        "people/created-contact-1",
        "created-etag-2",
    ));

    upsert_mail_contacts_sync_schedule(&supervisor);
    let scheduled_run_id = runtime.block_on(wait_for_scheduled_run_id());
    let scheduled = wait_for_completed(
        &store,
        &supervisor,
        &sync.registration_id,
        &scheduled_run_id,
    );
    assert_eq!(scheduled.account_id, MAIL_ACCOUNT_ID);
    assert_eq!(scheduled.provider_entries_seen, 1);
    assert_eq!(scheduled.contacts_updated, 1);
    assert_eq!(scheduled.contacts_unchanged, 0);
    assert_eq!(provider.accepted_people_reads(), 2);
    assert_eq!(provider.accepted_people_writes(), 4);
    runtime.block_on(wait_for_scheduler_terminal(&scheduled_run_id));

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
        assert_eq!(contacts_count, 2);
        assert_eq!(completed_inbox, 2);
        pool.close().await;
    });

    let duplicate = route_start(&store, &supervisor, &sync.registration_id, 3, &request);
    assert_eq!(duplicate.run_id, accepted.run_id);
    let duplicate_completed =
        wait_for_completed(&store, &supervisor, &sync.registration_id, &accepted.run_id);
    assert_eq!(duplicate_completed.state_revision, completed.state_revision);
    assert_eq!(provider.accepted_people_reads(), 2);
    assert_eq!(provider.accepted_people_writes(), 4);

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail Contacts Sync fixture");
    std::fs::remove_dir_all(data).expect("remove Mail Contacts Sync Kernel fixture");
}

fn wait_for_people_write(provider: &MailGmailFixture, expected: usize) -> bool {
    let deadline = Instant::now() + Duration::from_secs(30);
    while provider.accepted_people_writes() != expected {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    true
}

fn wall_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock")
        .as_secs()
        .try_into()
        .expect("wall seconds")
}

async fn reverse_diagnostic() -> Vec<(i16, bool, bool, Option<i16>, Option<i16>)> {
    let pool = contacts_admin_pool_v1().await;
    let rows = sqlx::query_as::<_, (i16, bool, bool, Option<i16>, Option<i16>)>(
        "SELECT operation.state, operation.origin_run_id IS NOT NULL, \
                operation.mail_command_message_id IS NOT NULL, run.rejection_code, \
                mail_command.state \
         FROM hermes_data.mail_contacts_sync_reverse_operations AS operation \
         LEFT JOIN hermes_data.mail_contacts_sync_runs AS run \
           ON run.logical_owner_id=operation.logical_owner_id \
          AND run.run_id=operation.origin_run_id \
         LEFT JOIN hermes_data.mail_address_book_upsert_inbox AS mail_command \
           ON mail_command.command_message_id=operation.mail_command_message_id \
         WHERE operation.logical_owner_id=$1 \
         ORDER BY operation.created_at_unix_millis, operation.operation_id",
    )
    .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
    .fetch_all(&pool)
    .await
    .expect("read reverse diagnostic");
    pool.close().await;
    rows
}

async fn wait_for_reverse_terminal(
    expected_state: i16,
    supervisor: &ManagedRuntimeSupervisor,
    registration_id: &str,
) {
    let pool = contacts_admin_pool_v1().await;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let state = sqlx::query_scalar::<_, i16>(
            "SELECT state FROM hermes_data.mail_contacts_sync_reverse_operations \
             WHERE logical_owner_id=$1 ORDER BY created_at_unix_millis DESC LIMIT 1",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .fetch_optional(&pool)
        .await
        .expect("read reverse Mail Contacts Sync operation");
        if state == Some(expected_state) {
            pool.close().await;
            return;
        }
        assert!(
            Instant::now() < deadline,
            "reverse Mail Contacts Sync did not reach state {expected_state}: {state:?}; \
             active={:?}; last_failure={:?}",
            supervisor.is_active(registration_id),
            supervisor.last_failure(registration_id)
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn queue_local_contact_change(
    contact_id: [u8; 16],
    revision: u64,
    display_name: &str,
    runtime_generation: u64,
) {
    let pool = contacts_admin_pool_v1().await;
    let now_seconds = wall_seconds();
    let now_millis = now_seconds * 1_000 + i64::try_from(revision).expect("bounded revision");
    if revision == 1 {
        sqlx::query(
            "INSERT INTO hermes_data.contacts_state (logical_owner_id, contact_id, display_name, \
             contact_revision, created_at_unix_seconds, created_at_nanos, updated_at_unix_seconds, \
             updated_at_nanos) VALUES ($1,$2,$3,$4,$5,0,$5,0)",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .bind(contact_id.as_slice())
        .bind(display_name)
        .bind(i64::try_from(revision).expect("bounded revision"))
        .bind(now_seconds)
        .execute(&pool)
        .await
        .expect("seed local Contact state");
        sqlx::query(
            "INSERT INTO hermes_data.contacts_email_identities \
             (logical_owner_id, normalized_email, contact_id) VALUES ($1,$2,$3)",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .bind("local-create@example.test")
        .bind(contact_id.as_slice())
        .execute(&pool)
        .await
        .expect("seed local Contact email");
        sqlx::query(
            "INSERT INTO hermes_data.contacts_phone_identities \
             (logical_owner_id, normalized_phone, contact_id) VALUES ($1,$2,$3)",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .bind("+12025550199")
        .bind(contact_id.as_slice())
        .execute(&pool)
        .await
        .expect("seed local Contact phone");
    } else {
        let updated = sqlx::query(
            "UPDATE hermes_data.contacts_state SET display_name=$3, contact_revision=$4, \
             updated_at_unix_seconds=$5 WHERE logical_owner_id=$1 AND contact_id=$2 AND \
             contact_revision=$4-1",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .bind(contact_id.as_slice())
        .bind(display_name)
        .bind(i64::try_from(revision).expect("bounded revision"))
        .bind(now_seconds)
        .execute(&pool)
        .await
        .expect("update local Contact state");
        assert_eq!(updated.rows_affected(), 1);
    }
    let event = build_contact_changed_for_mail_sync_outbox_record_v1(
        ContactChangedForMailSyncV1 {
            contact_id: contact_id.to_vec(),
            contact_revision: revision,
            logical_owner_id: MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
        },
        &ContactsMailSyncSourceEnvelopeContextV1 {
            module_id: "hermes-contacts-runtime".to_owned(),
            runtime_instance_id: "managed-contacts-create-source".to_owned(),
            runtime_generation,
            recorded_at_unix_seconds: now_seconds,
            recorded_at_nanos: i32::try_from(revision).expect("bounded revision"),
        },
    )
    .expect("build local Contact changed event");
    sqlx::query(
        "INSERT INTO hermes_data.contacts_outbox (logical_owner_id, message_id, envelope_sha256, \
         envelope_bytes, created_at_unix_millis) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
    .bind(event.message_id().as_slice())
    .bind(event.envelope_sha256().as_slice())
    .bind(event.exact_bytes())
    .bind(now_millis)
    .execute(&pool)
    .await
    .expect("queue local Contact changed event");
    pool.close().await;
}

async fn wait_for_reverse_contact_terminal(
    contact_id: [u8; 16],
    revision: u64,
    expected_state: i16,
) {
    let pool = contacts_admin_pool_v1().await;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let state = sqlx::query_scalar::<_, i16>(
            "SELECT state FROM hermes_data.mail_contacts_sync_reverse_operations WHERE \
             logical_owner_id=$1 AND contact_id=$2 AND contact_revision=$3",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .bind(contact_id.as_slice())
        .bind(i64::try_from(revision).expect("bounded revision"))
        .fetch_optional(&pool)
        .await
        .expect("read local reverse operation");
        if state == Some(expected_state) {
            pool.close().await;
            return;
        }
        assert!(
            Instant::now() < deadline,
            "local reverse operation state: {state:?}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn assert_provider_link(contact_id: [u8; 16], entry_id: &str, etag: &str) {
    let pool = contacts_admin_pool_v1().await;
    let actual = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT provider_entry_id, provider_etag FROM hermes_data.contacts_provider_links WHERE \
         logical_owner_id=$1 AND contact_id=$2 AND source_account_id=$3",
    )
    .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
    .bind(contact_id.as_slice())
    .bind(MAIL_ACCOUNT_ID)
    .fetch_one(&pool)
    .await
    .expect("read reconciled Contacts provider link");
    assert_eq!(actual.0, entry_id);
    assert_eq!(actual.1.as_deref(), Some(etag));
    pool.close().await;
}

fn upsert_mail_contacts_sync_schedule(supervisor: &ManagedRuntimeSupervisor) {
    let now = current_unix_millis();
    let due_at = now + 1_500;
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 900_000,
        },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::FireOnce,
        RetryPolicyV1::new(3, 1_000).expect("Mail Contacts Sync retry policy"),
        120_000,
        0,
    )
    .expect("Mail Contacts Sync schedule policy");
    let request = SchedulerRuntimeControlRequestV1 {
        operation: Some(SchedulerOperation::UpsertSchedule(
            UpsertSchedulerScheduleRequestV1 {
                schedule_id: vec![0x91; 16],
                schedule_revision: 1,
                job_owner: MAIL_CONTACTS_SYNC_OWNER_ID_V1.to_owned(),
                job_name: "scheduled_sync".to_owned(),
                job_major: 1,
                contract_name: "mail_contacts_sync.scheduled_sync".to_owned(),
                contract_revision: 1,
                contract_schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
                scope_id: MAIL_CONTACTS_SYNC_CONFIGURATION_ID_V1.to_owned(),
                concurrency_key: MAIL_CONTACTS_SYNC_CONFIGURATION_ID_V1.to_owned(),
                enabled: true,
                policy_canonical_bytes: policy.canonical_bytes(),
                next_due_at_unix_millis: due_at,
                updated_at_unix_millis: now,
            },
        )),
    };
    let response = supervisor
        .relay(SCHEDULER_REGISTRATION, request.encode_to_vec())
        .expect("upsert Mail Contacts Sync schedule");
    let response = SchedulerRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode Mail Contacts Sync schedule response");
    assert!(matches!(
        response.result,
        Some(SchedulerResult::UpsertSchedule(result))
            if result.schedule_revision == 1
                && result.outcome == SchedulerScheduleUpsertOutcomeV1::Inserted as i32
    ));
    assert!(response.error_code.is_empty());
}

async fn wait_for_scheduled_run_id() -> [u8; 16] {
    let pool = contacts_admin_pool_v1().await;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let row = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT run_id FROM hermes_data.mail_contacts_sync_runs
             WHERE logical_owner_id=$1 AND trigger_kind=2
             ORDER BY created_at_unix_millis DESC LIMIT 1",
        )
        .bind(MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1)
        .fetch_optional(&pool)
        .await
        .expect("read scheduled Mail Contacts Sync run");
        if let Some(run_id) = row {
            pool.close().await;
            return run_id.try_into().expect("scheduled run id");
        }
        assert!(
            Instant::now() < deadline,
            "Scheduler did not launch Mail Contacts Sync"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_scheduler_terminal(run_id: &[u8; 16]) {
    let pool = contacts_admin_pool_v1().await;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let outcome = sqlx::query_scalar::<_, String>(
            "SELECT outcome FROM hermes_platform.scheduler_run_results WHERE run_id=$1",
        )
        .bind(run_id.as_slice())
        .fetch_optional(&pool)
        .await
        .expect("read Scheduler terminal receipt");
        if outcome.as_deref() == Some("finished") {
            pool.close().await;
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Scheduler did not receive terminal Mail Contacts Sync result: {outcome:?}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
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
        .unwrap_or_else(|error| {
            panic!(
                "route Mail Contacts Sync client request: {error}; active={:?}; last_failure={:?}",
                supervisor.is_active(registration_id),
                supervisor.last_failure(registration_id)
            )
        });
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
