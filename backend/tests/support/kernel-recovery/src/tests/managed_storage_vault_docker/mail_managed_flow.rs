//! Live managed Mail launch through Kernel-owned admission and platform leases.

use super::*;

use hermes_gateway_protocol::owner_control_client::{
    OwnerControlClientV1, OwnerControlProofSignerV1,
};
use hermes_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use hermes_mail_api::{
    MailClientRequestV1, MailSendMailRequestV1, MailSyncInboxRequestV1,
    client_contract::MailClientContractV1,
};
use hermes_mail_runtime::admission::MAIL_STORAGE_CAPABILITY_ID;
use hermes_mail_runtime::client_port::encode_module_request;

use crate::identity::device::signer::DeviceSigner;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Mail and NATS binaries"]
fn managed_mail_runtime_uses_kernel_leases_and_route_specific_admission() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let imap = MailImapFixture::start();
    let root = unique_target_root("hermes-managed-mail-runtime");
    let data = private_directory(short_communications_kernel_data_directory());
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    seed_mail_vault(&vault_dir);
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
    let admitted_mail = admit_mail_runtime(&store);
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

    let mail = start_mail_runtime(
        &supervisor,
        &store,
        &data,
        &root.join("runtime"),
        admitted_mail,
        imap.port(),
    );
    assert_mail_event_only_communications_handoff(&store, &supervisor, &mail);
    assert_mail_attachment_lifecycle(&store, &supervisor, &mail);
    assert_ungranted_delivery_is_rejected(&store, &supervisor, &mail);
    assert_stale_sync_generation_is_rejected(&store, &supervisor, &mail);
    assert!(
        imap.accepted_connections() > 0,
        "managed Mail runtime must reach the live loopback IMAP fixture"
    );
    let (owner_runtime_dir, owner_control) =
        start_owner_control(&data, &store, &shutdown, &supervisor);
    revoke_mail_runtime(
        &owner_runtime_dir,
        &owner_signer,
        &store,
        &supervisor,
        &mail,
    );

    supervisor.shutdown().expect("stop managed processes");
    owner_control
        .join()
        .expect("join owner control server")
        .expect("owner control server");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

struct LiveOwnerSigner<'a>(&'a FileDeviceSigner);

impl OwnerControlProofSignerV1 for LiveOwnerSigner<'_> {
    fn sign_owner_control_proof(&self, message: &[u8]) -> Result<[u8; 64], String> {
        Ok(self.0.sign(message))
    }
}

fn start_owner_control(
    data: &Path,
    store: &Arc<SqliteControlStore>,
    shutdown: &Arc<AtomicBool>,
    supervisor: &ManagedRuntimeSupervisor,
) -> (PathBuf, std::thread::JoinHandle<Result<(), String>>) {
    let runtime_dir = private_directory(data.join("owner-control-runtime"));
    let server_runtime_dir = runtime_dir.clone();
    let server_data = data.to_path_buf();
    let server_store = Arc::clone(store);
    let server_shutdown = Arc::clone(shutdown);
    let server_supervisor = supervisor.clone();
    let server = std::thread::spawn(move || {
        crate::identity::owner_control::serve(
            server_store,
            &server_data,
            &server_runtime_dir,
            server_shutdown,
            server_supervisor,
            None,
        )
    });
    for _ in 0..250 {
        if runtime_dir.join("owner.sock").exists() {
            return (runtime_dir, server);
        }
        if server.is_finished() {
            let outcome = server.join().expect("join failed owner control server");
            panic!("owner control server exited before socket readiness: {outcome:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("owner control socket did not become ready");
}

fn revoke_mail_runtime(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let client = OwnerControlClientV1::new(owner_runtime_dir);
    let owner_session = client
        .open_owner_session(&LiveOwnerSigner(signer))
        .expect("open owner-authorized control session");
    let revoked = client
        .transition_module_registration(&owner_session, &mail.registration_id, "revoked")
        .expect("revoke managed Mail registration");
    assert_eq!(revoked.registration_state, "revoked");
    assert!(
        revoked.grant_epoch > mail.grant_epoch,
        "revoke advances the durable grant epoch before process stop"
    );
    let registration = store
        .module_registration(&mail.registration_id)
        .expect("read revoked Mail registration")
        .expect("revoked Mail registration");
    assert_eq!(registration.state(), ModuleRegistrationState::Revoked);
    let binding = store
        .platform_storage_binding(&mail.registration_id, MAIL_STORAGE_CAPABILITY_ID)
        .expect("read revoked Mail Storage binding")
        .expect("revoked Mail Storage binding");
    assert_eq!(
        binding.state(),
        PlatformStorageBindingStateV1::Revoking,
        "owner transition durably reserves the exact Mail Storage fence"
    );
    assert!(
        !supervisor
            .stop_if_active(&mail.registration_id)
            .expect("observe stopped Mail worker"),
        "owner transition already stopped the exact Mail worker"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("observe Communications worker"),
        "Mail revoke must not stop Communications"
    );
    assert_revoked_sync_route_is_rejected(store, supervisor, mail);
}

fn assert_revoked_sync_route_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        3,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "revoked-mail-sync".to_owned(),
        }),
    )
    .expect("encode revoked Mail sync module request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        MailClientContractV1::Sync.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("revoked Mail sync route"),
        "module registration is not approved"
    );
}

fn assert_ungranted_delivery_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        1,
        &MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: "ungranted-mail-delivery".to_owned(),
            provider_conversation_id: "conversation-1".to_owned(),
            recipients: vec!["recipient@example.test".to_owned()],
            subject: "must not be delivered".to_owned(),
            text_body: "Kernel rejects this route before Mail receives it".to_owned(),
        }),
    )
    .expect("encode exact Mail delivery module request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        MailClientContractV1::Delivery.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("ungranted Mail delivery route"),
        "capability is not granted to this registration"
    );
}

fn assert_stale_sync_generation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        2,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "stale-mail-sync".to_owned(),
        }),
    )
    .expect("encode exact Mail sync module request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation + 1,
        mail.grant_epoch,
        MailClientContractV1::Sync.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("stale Mail sync generation"),
        "managed runtime fence is stale"
    );
}
