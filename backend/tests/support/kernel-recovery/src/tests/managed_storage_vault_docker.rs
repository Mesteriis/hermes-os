//! Real managed Vault and Storage binaries over disposable PostgreSQL/PgBouncer.

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use futures_util::StreamExt;
use hermes_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingInputV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialRecipientPublicKeyV1, RuntimeNatsJwtCredentialV1, v1::DurableEnvelopeV1,
};
use hermes_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleEventDeliveryPolicyV1,
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
    ModuleStorageRequestV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
    PlatformStorageBundleV1, PlatformStorageEndpointV1, PlatformStorageTopology,
    StorageDeploymentProfileV1,
};
use hermes_runtime_protocol::v1::{
    ManagedDomainRuntimeConfigurationV1, ManagedRuntimeEventCredentialDeliveryV1, ManagedRuntimeEventCredentialRequestV1,
    SchedulerRuntimeControlRequestV1, SchedulerRuntimeControlResponseV1,
    SchedulerScheduleUpsertOutcomeV1, SettingsSchemaRefV1, SettingsSchemaV1,
    UpsertSchedulerScheduleRequestV1,
    scheduler_runtime_control_request_v1::Operation as SchedulerOperation,
    scheduler_runtime_control_response_v1::Result as SchedulerResult,
};
use hermes_scheduler_protocol::v1::ScheduledJobCommandV1;
use hermes_storage_protocol::v1::{
    GetStorageRuntimeStatusRequestV1, StorageRuntimeControlRequestV1,
    StorageRuntimeControlResponseV1, StorageRuntimeStateV1,
    storage_runtime_control_request_v1::Operation,
    storage_runtime_control_response_v1::Result as StorageResult,
};
use nats_jwt::KeyPair;
use prost::Message;

use super::common::*;
use crate::identity::device::signer::FileDeviceSigner;
use crate::platform::managed::signed_bundle::{InstalledSignedBundle, SignedRuntimeArtifact};
use crate::platform::vault::managed_route::KernelManagedVaultRouteHandler;
use crate::platform::vault::owner_derived_key::OwnerDerivedKeyHandlerV1;
use crate::platform::vault::status as vault_status;
use crate::platform::vault::{binding as vault_binding, launch as vault_launch};
use crate::platform::{
    blob::{binding as blob_binding, launch as blob_launch, session::BlobSessionHandlerV1},
    events::{catalog as event_catalog, topology as event_topology},
    macos::managed_launch,
    scheduler::{launch as scheduler_launch, lifecycle as scheduler_lifecycle},
    storage::issuance::{StorageBindingIssueV1, issue_managed},
};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeEventCredentialHandler, ManagedRuntimeExpectation,
};

#[path = "managed_storage_vault_docker/shared_fixture.rs"]
mod shared_fixture;
use shared_fixture::*;
#[path = "managed_storage_vault_docker/scheduler_setup.rs"]
mod scheduler_setup;
use scheduler_setup::*;
#[path = "managed_storage_vault_docker/scheduler_events.rs"]
mod scheduler_events;
use scheduler_events::*;
#[path = "managed_storage_vault_docker/communications_setup.rs"]
mod communications_setup;
use communications_setup::*;

#[test]
#[ignore = "requires disposable Docker plus real managed Vault and Storage binaries"]
fn managed_storage_binary_bootstraps_through_live_vault() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-storage-vault-docker");
    let data = private_directory(root.join("kernel"));
    let vault_dir = private_directory(data.join("vault"));
    initialize_vault(&vault_dir, &credential_directory());
    let release = installed_release(&root);
    let store = Arc::new(configured_store(&root, release.kernel()));
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    assert_eq!(
        start_vault(&supervisor, &store, &data, release.kernel()),
        1,
        "Vault starts from the signed release binding"
    );
    let vault =
        vault_status::read_current(&store, &supervisor.relay_port()).expect("live Vault status");
    assert_eq!(vault.runtime_generation(), 1);
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory()
        ),
        1,
        "Storage starts from the signed release binding"
    );
    assert_reconciling_status(&supervisor, 1);
    supervisor.stop("storage").expect("stop Storage");
    assert_eq!(
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory()
        ),
        2,
        "restarted Storage re-verifies the signed release binding"
    );
    assert_reconciling_status(&supervisor, 2);
    supervisor.shutdown().expect("stop managed processes");
    std::fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Scheduler and NATS binaries"]
fn managed_scheduler_crash_uses_storage_control_successor_provisioning() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let fixture = SchedulerRecoveryFixture::start();
    let binding = fixture.start_initial_scheduler();
    let due_at = fixture.persist_recovery_schedule();
    let worker = fixture.restart_after_crash(due_at);
    let successor = fixture.assert_successor(&binding, due_at);
    fixture.assert_revoked_binding_does_not_restart(successor);
    fixture.shutdown(worker);
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS and Communications binaries"]
fn managed_communications_domain_starts_with_owner_local_storage_and_events() {
    assert_eq!(
        std::env::var("HERMES_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("hermes-managed-communications-domain");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_release(&root);
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
        .expect("claim logical browser owner");
    super::browser_gateway_session::admit_browser_test_device(&store, "owner-1");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler))
        .expect("configure Communications Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(&supervisor, &store, release.kernel(), &data, &root.join("runtime"))
            .expect("start signed Blob runtime"),
        1,
        "Blob starts as a separate managed platform process"
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
    configure_communications_jetstream(&store);

    assert_eq!(
        start_communications_domain(&supervisor, &store, &root.join("runtime")),
        1,
        "generic managed-domain launch admits Communications without a Kernel owner facade"
    );
    assert!(
        supervisor
            .is_active(COMMUNICATIONS_REGISTRATION)
            .expect("read Communications process state")
    );
    assert_communications_ingress_delivery(&store, &supervisor);
    assert_communications_relationship_projection(&store, &supervisor);
    assert_communications_attachment_anchor_projection(&store, &supervisor);
    assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
    );
    assert_communications_query_delivery(&store, &supervisor);
    assert_communications_search_query_delivery(&store, &supervisor);
    assert_communications_gateway_query_delivery(&store, &supervisor, &root);
    assert_fenced_communications_target_cannot_issue_blob_custody_grant(
        &store,
        &supervisor,
        &data,
    );

    supervisor.shutdown().expect("stop managed processes");
    unsafe {
        std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove fixture");
    std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
}

fn short_communications_kernel_data_directory() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    PathBuf::from("/tmp").join(format!("hermes-comms-{}-{suffix}", std::process::id()))
}

fn assert_communications_gateway_query_delivery(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &std::path::Path,
) {
    use http_body_util::BodyExt as _;

    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("gateway-cert.der"),
        root.join("gateway-key.der"),
    )
    .expect("Gateway configuration");
    let router = crate::platform::gateway::gateway_service(
        Arc::clone(store),
        supervisor.clone(),
        &configuration,
        None,
    )
    .expect("compose owner Gateway routes");
    let runtime = tokio::runtime::Runtime::new().expect("Gateway test runtime");
    let cookie = super::browser_gateway_session::authenticate_gateway_router(&router, &runtime);
    let payload = prost::Message::encode_to_vec(
        &hermes_communications_api::query_wire::CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(
                hermes_communications_api::query_wire::communications_query_request_v1::Operation::ListAccounts(
                    hermes_communications_api::query_wire::ListAccountsRequestV1 { limit: 16 },
                ),
            ),
        },
    );
    let response = runtime.block_on(router.route(
        hyper::Request::builder()
            .method("POST")
            .uri("/hermes.communications.query.v1.CommunicationsQueryService/Query")
            .header("content-type", "application/connect+proto")
            .header("cookie", cookie)
            .body(http_body_util::Full::new(hyper::body::Bytes::from(payload)))
            .expect("Gateway owner query request"),
    ));
    assert_eq!(response.status(), hyper::StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").and_then(|value| value.to_str().ok()),
        Some("application/proto"),
    );
    assert_eq!(
        response.headers().get("connect-protocol-version").and_then(|value| value.to_str().ok()),
        Some("1"),
    );
    let bytes = runtime
        .block_on(response.into_body().collect())
        .expect("Gateway owner query response")
        .to_bytes();
    let response = hermes_communications_api::query_wire::CommunicationsQueryResponseV1::decode(bytes.as_ref())
        .expect("decode Gateway Communications query response");
    assert!(matches!(
        response.result,
        Some(hermes_communications_api::query_wire::communications_query_response_v1::Result::ListAccounts(accounts))
            if !accounts.accounts.is_empty()
    ));
}

struct SchedulerRecoveryFixture {
    root: PathBuf,
    release: InstalledSignedBundle,
    store: Arc<SqliteControlStore>,
    shutdown: Arc<AtomicBool>,
    supervisor: ManagedRuntimeSupervisor,
}

impl SchedulerRecoveryFixture {
    fn start() -> Self {
        let root = unique_target_root("hermes-managed-scheduler-lifecycle");
        let data = private_directory(root.join("kernel"));
        initialize_vault(
            &private_directory(data.join("vault")),
            &credential_directory(),
        );
        let release = installed_scheduler_release(&root);
        let store = Arc::new(configured_scheduler_store(&root, release.kernel()));
        let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
        let shutdown = Arc::new(AtomicBool::new(false));
        let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
        configure_route_handler(&supervisor, &store, &data);
        supervisor
            .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler))
            .expect("configure Scheduler Event credential handler");
        start_vault(&supervisor, &store, &data, release.kernel());
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
        .unwrap_or_else(|error| panic!("provision initial Scheduler Storage binding: {error:?}"));
        configure_scheduler_jetstream(&store);
        configure_scheduler_delivery_observer(&store);
        Self {
            root,
            release,
            store,
            shutdown,
            supervisor,
        }
    }

    fn start_initial_scheduler(&self) -> hermes_kernel_control_store::PlatformStorageBindingV1 {
        let reservation =
            managed_launch::load(&self.supervisor, &self.store, SCHEDULER_REGISTRATION)
                .expect("load initial Scheduler reservation");
        let binding = scheduler_binding(&self.store);
        assert_eq!(
            scheduler_launch::start_from_reservation(
                &self.supervisor,
                &self.store,
                self.release.kernel(),
                &self.root.join("runtime"),
                reservation,
                &binding,
            )
            .expect("start initial Scheduler"),
            1
        );
        binding
    }

    fn persist_recovery_schedule(&self) -> i64 {
        let replaced_due_at = future_due_at_unix_millis();
        let due_at = replaced_due_at + 3_000;
        upsert_recovery_schedule(
            &self.supervisor,
            1,
            replaced_due_at,
            SchedulerScheduleUpsertOutcomeV1::Inserted,
        );
        upsert_recovery_schedule(
            &self.supervisor,
            2,
            due_at,
            SchedulerScheduleUpsertOutcomeV1::Updated,
        );
        due_at
    }

    fn restart_after_crash(&self, due_at: i64) -> std::thread::JoinHandle<Result<(), String>> {
        self.supervisor
            .stop(SCHEDULER_REGISTRATION)
            .expect("simulate Scheduler crash");
        wait_until_due(due_at);
        let store = Arc::clone(&self.store);
        let supervisor = self.supervisor.clone();
        let shutdown = Arc::clone(&self.shutdown);
        let runtime_dir = self.root.join("runtime");
        let kernel = self.release.kernel().to_path_buf();
        std::thread::spawn(move || {
            scheduler_lifecycle::serve(store, &kernel, &runtime_dir, shutdown, supervisor)
        })
    }

    fn assert_successor(
        &self,
        binding: &hermes_kernel_control_store::PlatformStorageBindingV1,
        due_at: i64,
    ) -> hermes_kernel_control_store::PlatformStorageBindingV1 {
        wait_for_scheduler_generation(&self.supervisor, &self.store, 2);
        let successor = scheduler_binding(&self.store);
        assert_eq!(successor.runtime_generation(), 2);
        assert_ne!(
            successor.runtime_instance_id(),
            binding.runtime_instance_id()
        );
        assert_eq!(successor.role_epoch(), 2);
        assert_eq!(successor.credential_lease_revision(), 2);
        assert_recovered_scheduler_delivery(&self.store, due_at);
        successor
    }

    fn assert_revoked_binding_does_not_restart(
        &self,
        successor: hermes_kernel_control_store::PlatformStorageBindingV1,
    ) {
        let revoking = self
            .store
            .begin_platform_storage_binding_revocation(
                SCHEDULER_REGISTRATION,
                STORAGE_CAPABILITY,
                successor.binding_revision(),
            )
            .expect("reserve successor binding revocation");
        self.supervisor
            .stop(SCHEDULER_REGISTRATION)
            .expect("stop revoked Scheduler");
        std::thread::sleep(Duration::from_millis(600));
        assert!(
            !self
                .supervisor
                .is_active(SCHEDULER_REGISTRATION)
                .expect("read Scheduler state")
        );
        assert_eq!(revoking.runtime_generation(), 2);
    }

    fn shutdown(self, worker: std::thread::JoinHandle<Result<(), String>>) {
        self.shutdown.store(true, Ordering::Release);
        worker
            .join()
            .expect("join Scheduler lifecycle")
            .expect("lifecycle exits");
        self.supervisor.shutdown().expect("stop managed processes");
        std::fs::remove_dir_all(self.root).expect("remove fixture");
    }
}

fn assert_recovered_scheduler_delivery(store: &SqliteControlStore, due_at: i64) {
    let envelope = recovered_scheduler_delivery(store);
    assert!(
        matches!(envelope.contract, Some(contract) if contract.owner == "platform" && contract.name == "maintenance")
    );
    assert!(
        matches!(envelope.source, Some(source) if source.module_id == SCHEDULER_REGISTRATION && source.runtime_generation == 2)
    );
    let command = ScheduledJobCommandV1::decode(envelope.payload.as_slice())
        .expect("decode recovered Scheduler command");
    assert_eq!(command.schedule_revision, 2);
    assert_eq!(command.scheduled_for_unix_millis, due_at);
}

fn configure_route_handler(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
) {
    let vault_route = Arc::new(KernelManagedVaultRouteHandler::new(
        Arc::clone(store),
        data,
        Arc::new(supervisor.relay_port()),
    ));
    let vault_handler: Arc<dyn crate::runtime::lifecycle::control::ManagedRuntimeVaultRouteHandler> =
        vault_route.clone();
    supervisor
        .configure_vault_route_handler(vault_handler)
        .expect("Vault route handler");
    supervisor
        .configure_owner_derived_key_handler(Arc::new(OwnerDerivedKeyHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            vault_route,
        )))
        .expect("owner-derived key handler");
    supervisor
        .configure_blob_session_handler(Arc::new(BlobSessionHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
            data.to_path_buf(),
        )))
        .expect("Blob session handler");
}

fn upsert_recovery_schedule(
    supervisor: &ManagedRuntimeSupervisor,
    schedule_revision: u64,
    due_at: i64,
    expected_outcome: SchedulerScheduleUpsertOutcomeV1,
) {
    let request = SchedulerRuntimeControlRequestV1 {
        operation: Some(SchedulerOperation::UpsertSchedule(
            UpsertSchedulerScheduleRequestV1 {
                schedule_id: vec![9; 16],
                schedule_revision,
                job_owner: "platform".to_owned(),
                job_name: "maintenance".to_owned(),
                job_major: 1,
                contract_name: "platform.maintenance".to_owned(),
                contract_revision: 1,
                contract_schema_sha256: vec![7; 32],
                scope_id: "recovery:opaque".to_owned(),
                concurrency_key: "recovery:opaque".to_owned(),
                enabled: true,
                policy_canonical_bytes: one_shot_recovery_policy(due_at),
                next_due_at_unix_millis: due_at,
                updated_at_unix_millis: due_at - 1_000,
            },
        )),
    };
    let response = supervisor
        .relay(SCHEDULER_REGISTRATION, request.encode_to_vec())
        .expect("persist recovery schedule through Scheduler control");
    let response = SchedulerRuntimeControlResponseV1::decode(response.as_slice())
        .expect("decode Scheduler schedule response");
    assert!(matches!(
        response.result,
        Some(SchedulerResult::UpsertSchedule(result))
            if result.schedule_revision == schedule_revision
                && result.outcome == expected_outcome as i32
    ));
    assert!(response.error_code.is_empty());
}

fn future_due_at_unix_millis() -> i64 {
    current_unix_millis() + 2_000
}

fn current_unix_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("wall clock after epoch")
        .as_millis()
        .try_into()
        .expect("Unix milliseconds fit i64")
}

fn wait_until_due(due_at: i64) {
    let now = current_unix_millis();
    if due_at > now {
        let delay = u64::try_from(due_at - now).expect("future due delay") + 100;
        std::thread::sleep(Duration::from_millis(delay));
    }
}

fn one_shot_recovery_policy(due_at: i64) -> Vec<u8> {
    let mut policy = Vec::with_capacity(32);
    policy.push(1); // encoding version
    policy.push(1); // trigger: at
    policy.extend_from_slice(&due_at.to_be_bytes());
    policy.push(1); // overlap: forbid
    policy.push(2); // misfire: fire once after successor recovery
    policy.extend_from_slice(&1_u16.to_be_bytes()); // retry attempts
    policy.extend_from_slice(&1_000_u64.to_be_bytes()); // retry backoff
    policy.extend_from_slice(&1_000_u64.to_be_bytes()); // command deadline
    policy.extend_from_slice(&0_u64.to_be_bytes()); // jitter
    policy
}

const SCHEDULER_REGISTRATION: &str = "scheduler_registration";
const STORAGE_CAPABILITY: &str = "storage.scheduler";
const DISPATCH_CAPABILITY: &str = "events.scheduler.dispatch";
const ACK_CAPABILITY: &str = "events.scheduler.ack";
const RESULT_CAPABILITY: &str = "events.scheduler.result";

fn scheduler_binding(
    store: &SqliteControlStore,
) -> hermes_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(SCHEDULER_REGISTRATION, STORAGE_CAPABILITY)
        .expect("read Scheduler Storage binding")
        .expect("Scheduler Storage binding")
}

fn wait_for_scheduler_generation(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    expected_generation: u64,
) {
    // A managed child is allowed 15 seconds to announce readiness; include the
    // lifecycle poll and Storage/Vault provisioning time before declaring the
    // recovery contour failed.
    let deadline = std::time::Instant::now() + Duration::from_secs(25);
    while std::time::Instant::now() < deadline {
        let active = supervisor
            .is_active(SCHEDULER_REGISTRATION)
            .expect("read Scheduler runtime state");
        let generation = store
            .effective_managed_launch_record(SCHEDULER_REGISTRATION)
            .expect("read Scheduler launch record")
            .map(|record| record.runtime_generation());
        if active && generation == Some(expected_generation) {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "Scheduler successor did not reach generation {expected_generation}: {:?}",
        supervisor.last_failure(SCHEDULER_REGISTRATION)
    );
}

struct UnauthenticatedNatsCredentialHandler;

impl ManagedRuntimeEventCredentialHandler for UnauthenticatedNatsCredentialHandler {
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String> {
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| "Scheduler Event request is invalid".to_owned())?;
        let recipient = NatsRuntimeCredentialRecipientPublicKeyV1::from_bytes(
            request
                .recipient_public_key_x25519
                .as_slice()
                .try_into()
                .map_err(|_| "Scheduler Event request is invalid".to_owned())?,
        )
        .map_err(|_| "Scheduler Event request is invalid".to_owned())?;
        let binding = NatsRuntimeCredentialDeliveryBindingV1::new(
            NatsRuntimeCredentialDeliveryBindingInputV1 {
                logical_owner_id: "scheduler".to_owned(),
                registration_id: expectation.registration_id().to_owned(),
                runtime_instance_id: expectation.runtime_instance_id().to_owned(),
                runtime_generation: expectation.runtime_generation(),
                grant_epoch: expectation.grant_epoch(),
                credential_revision: request.credential_revision,
                request_id,
                recipient_public_key: recipient,
            },
        )
        .map_err(|_| "Scheduler Event binding is invalid".to_owned())?;
        let key = KeyPair::new_user();
        let credential = RuntimeNatsJwtCredentialV1::new(
            "test-jwt".to_owned(),
            key.seed()
                .map_err(|_| "Scheduler Event key is unavailable".to_owned())?,
            key.public_key(),
            u64::MAX,
        )
        .map_err(|_| "Scheduler Event credential is invalid".to_owned())?;
        let delivery = credential
            .seal_for(&binding)
            .map_err(|_| "Scheduler Event delivery is unavailable".to_owned())?;
        Ok(ManagedRuntimeEventCredentialDeliveryV1 {
            encapped_key: delivery.encapped_key().to_vec(),
            ciphertext: delivery.ciphertext().to_vec(),
            tag: delivery.tag().to_vec(),
            consumer_bindings: Vec::new(),
            publish_subjects: Vec::new(),
        })
    }
}
fn private_directory(path: PathBuf) -> PathBuf {
    std::fs::create_dir_all(&path).expect("private directory");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("private directory mode");
    path
}
