//! Live Mail credential binding, provider quiesce and Settings successor evidence.

use std::time::Duration;

use hermes_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSendMailRequestV1, MailSyncInboxRequestV1,
    account::{
        MailAccountReadinessV1, MailAccountStatusRequestV1, MailAccountStatusV1,
        MailBindCredentialRequestV1, MailCredentialBindingStateV1, MailCredentialPurposeV1,
    },
    client_contract::MailClientContractV1,
};
use hermes_mail_runtime::admission::MAIL_STORAGE_CAPABILITY_ID;
use prost::Message;

use crate::identity::device::signer::DeviceSigner;

use super::*;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Mail, IMAP, SMTP and NATS"]
fn managed_mail_credential_rotation_quiesces_until_settings_successor() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let smtp = MailSmtpFixture::start();
    let root = unique_target_root("hermes-managed-mail-account-credential");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let seeded = seed_mail_vault(&vault_dir);
    let release = installed_communications_mail_release(&root);
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
    let admitted_mail = admit_mail_account_credential_runtime(&store);
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
    let admitted_mail = prepare_mail_runtime(&supervisor, &store, admitted_mail);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let predecessor = start_mail_delivery_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        imap.port(),
        smtp_settings(&smtp),
    );
    wait_for_mail_ready(&supervisor, &predecessor);

    let active = query_account_status(&store, &supervisor, &predecessor, 81);
    assert_eq!(active.readiness, MailAccountReadinessV1::Ready);
    assert!(active.bindings.iter().all(|binding| {
        binding.state == MailCredentialBindingStateV1::Active
            && binding.credential_revision == Some(1)
            && binding.applied_runtime_generation == Some(predecessor.runtime_generation)
    }));

    rotate_basic_mail_vault(&vault_dir, &seeded);
    for (request_id, purpose) in [
        (82, MailCredentialPurposeV1::ImapPassword),
        (83, MailCredentialPurposeV1::SmtpPassword),
    ] {
        let response = route_mail_client(
            &store,
            &supervisor,
            &predecessor,
            MailClientContractV1::AccountCredentialBind,
            request_id,
            &MailClientRequestV1::BindCredential(MailBindCredentialRequestV1 {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                purpose,
                expected_binding_revision: 1,
                credential_revision: 2,
            }),
        );
        let MailClientResponseV1::CredentialBinding(receipt) = response else {
            panic!("Mail credential bind returned the wrong response");
        };
        assert_eq!(receipt.binding_revision, 2);
        assert_eq!(receipt.state, MailCredentialBindingStateV1::PendingRestart);
    }
    let pending = query_account_status(&store, &supervisor, &predecessor, 84);
    assert_eq!(pending.readiness, MailAccountReadinessV1::PendingRestart);
    assert!(pending.bindings.iter().all(|binding| {
        binding.state == MailCredentialBindingStateV1::PendingRestart
            && binding.credential_revision == Some(2)
            && binding.applied_runtime_generation.is_none()
    }));

    let sync_error = route_mail_client_once(
        &store,
        &supervisor,
        &predecessor,
        MailClientContractV1::Sync,
        85,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "mail-sync-quiesced".to_owned(),
        }),
    )
    .expect_err("pending IMAP binding must quiesce provider sync");
    assert_eq!(sync_error, "Mail route runtime error");
    let delivery_error = route_mail_client_once(
        &store,
        &supervisor,
        &predecessor,
        MailClientContractV1::Delivery,
        86,
        &delivery_request("mail-delivery-quiesced"),
    )
    .expect_err("pending SMTP binding must quiesce provider delivery");
    assert_eq!(delivery_error, "Mail route runtime error");
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(imap.accepted_connections(), 0);
    assert_eq!(smtp.accepted_messages(), 0);

    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    let (client, owner_session) = open_owner_control_client(&owner_runtime_dir, &owner_signer);
    let revision_two = mail_delivery_settings_snapshot(
        &predecessor.registration_id,
        imap.port(),
        smtp_settings(&smtp),
        2,
    )
    .encode_to_vec();
    client
        .update_operator_settings(
            &owner_session,
            &predecessor.registration_id,
            1,
            revision_two,
        )
        .expect("commit Mail Settings revision two");
    let applied = client
        .apply_managed_integration_settings(
            &owner_session,
            &predecessor.registration_id,
            MAIL_STORAGE_CAPABILITY_ID,
            MAIL_ACCOUNT_ID,
            2,
            false,
        )
        .expect("apply Mail credential-rotation successor");
    assert_eq!(
        applied.runtime_generation,
        predecessor.runtime_generation + 1
    );
    let successor = current_mail_runtime(&store, &predecessor);
    wait_for_mail_ready(&supervisor, &successor);
    let ready = query_account_status(&store, &supervisor, &successor, 87);
    assert_eq!(ready.readiness, MailAccountReadinessV1::Ready);
    assert!(ready.bindings.iter().all(|binding| {
        binding.state == MailCredentialBindingStateV1::Active
            && binding.credential_revision == Some(2)
            && binding.applied_runtime_generation == Some(successor.runtime_generation)
    }));

    let sync = route_mail_client(
        &store,
        &supervisor,
        &successor,
        MailClientContractV1::Sync,
        88,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "mail-sync-successor".to_owned(),
        }),
    );
    assert!(matches!(
        sync,
        MailClientResponseV1::SyncInboxCompleted {
            observed_messages: 1,
            ..
        }
    ));
    let accepted = route_mail_client(
        &store,
        &supervisor,
        &successor,
        MailClientContractV1::Delivery,
        89,
        &delivery_request("mail-delivery-successor"),
    );
    assert_eq!(
        accepted,
        MailClientResponseV1::MailAccepted {
            operation_id: "mail-delivery-successor".to_owned(),
        }
    );
    assert_delivery_completed(
        &store,
        &supervisor,
        &successor,
        "mail-delivery-successor",
        250,
    );
    assert_eq!(smtp.accepted_messages(), 1);
    assert!(
        route_mail_client_once(
            &store,
            &supervisor,
            &predecessor,
            MailClientContractV1::AccountQuery,
            90,
            &MailClientRequestV1::AccountStatus(MailAccountStatusRequestV1 {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
            }),
        )
        .is_err(),
        "stale Mail generation must not retain its query route"
    );

    supervisor.shutdown().expect("stop managed processes");
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Mail credential fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn query_account_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
) -> MailAccountStatusV1 {
    let response = route_mail_client(
        store,
        supervisor,
        mail,
        MailClientContractV1::AccountQuery,
        request_id,
        &MailClientRequestV1::AccountStatus(MailAccountStatusRequestV1 {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    );
    let MailClientResponseV1::AccountStatus(status) = response else {
        panic!("Mail account query returned the wrong response");
    };
    status
}

fn delivery_request(operation_id: &str) -> MailClientRequestV1 {
    MailClientRequestV1::SendMail(MailSendMailRequestV1 {
        operation_id: operation_id.to_owned(),
        provider_conversation_id: "mail-credential-conversation".to_owned(),
        recipients: vec!["recipient@example.test".to_owned()],
        subject: "credential rotation".to_owned(),
        text_body: "credential rotation body".to_owned(),
        attachment_anchor_ids: Vec::new(),
    })
}

fn smtp_settings(fixture: &MailSmtpFixture) -> MailSmtpFixtureSettingsV1 {
    MailSmtpFixtureSettingsV1 {
        port: fixture.port(),
        ca_certificate_pem: fixture.ca_certificate_pem().to_owned(),
    }
}
