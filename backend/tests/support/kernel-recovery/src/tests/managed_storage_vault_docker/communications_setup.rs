use super::*;

use hermes_communications_api::query_wire::{
    CommunicationsQueryRequestV1, CommunicationsQueryResponseV1, ListAccountsRequestV1,
    communications_query_request_v1::Operation,
    communications_query_response_v1::Result as QueryResult,
};
use hermes_communications_persistence::communications_storage_bundle_v1;
use hermes_communications_runtime::admission::{
    COMMUNICATIONS_EVENTS_CAPABILITY_ID, COMMUNICATIONS_MODULE_ID,
    COMMUNICATIONS_OBSERVE_CAPABILITY_ID, COMMUNICATIONS_OWNER_ID,
    COMMUNICATIONS_STORAGE_CAPABILITY_ID, COMMUNICATIONS_QUERY_CAPABILITY_ID,
    communications_module_descriptor_v1,
    communications_settings_schema_bytes_v1,
    communication_evidence_recorded_contract_reference_v1,
};
use hermes_communications_runtime::query_client_port::encode_module_query_request_v1;
use hermes_kernel_control_store::PlatformStorageBindingStateV1;
use hermes_runtime_protocol::v1::ModuleClientResponseV1;

pub(super) const COMMUNICATIONS_REGISTRATION: &str = "communications-runtime";
const COMMUNICATIONS_RUNTIME_INSTANCE_ID: &str = "02020202020202020202020202020202";

pub(super) fn configured_communications_store(root: &Path, kernel: &Path) -> SqliteControlStore {
    let store = configured_store(root, kernel);
    let schema = communications_settings_schema_bytes_v1();
    let descriptor = communications_module_descriptor_v1("managed-communications-live").encode_to_vec();
    let grant_epoch = record_communications_registration(&store, &descriptor);
    record_communications_runtime_fixture(&store, &schema, &descriptor, grant_epoch);
    store
}

pub(super) fn issue_initial_communications_storage_binding(store: &SqliteControlStore) {
    let bundle = store
        .platform_storage_bundle("communications", 1)
        .expect("read Communications Storage bundle")
        .expect("Communications Storage bundle is present");
    let binding = issue_managed(
        store,
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_RUNTIME_INSTANCE_ID,
        1,
        COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(1, 1, 1, *bundle.digest())
            .expect("initial Communications Storage issue"),
    )
    .expect("issue Communications Storage binding");
    assert_eq!(binding.runtime_generation(), 1);
}

pub(super) fn communications_storage_binding(
    store: &SqliteControlStore,
) -> hermes_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(COMMUNICATIONS_REGISTRATION, COMMUNICATIONS_STORAGE_CAPABILITY_ID)
        .expect("read Communications Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Communications Storage binding")
}

pub(super) fn configure_communications_jetstream(store: &SqliteControlStore) {
    let configuration = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let contracts = event_catalog::resolve_contracts(store).expect("resolve Event Hub contracts");
    let plan = event_topology::plan(&contracts, &configuration).expect("plan Event Hub topology");
    let endpoint = configuration.nats_endpoint().to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(&endpoint).await.expect("connect JetStream"),
            );
            for stream in plan.streams() {
                let (name, subject) = communications_stream_details(stream.kind());
                context
                    .create_stream(async_nats::jetstream::stream::Config {
                        name: name.to_owned(),
                        subjects: vec![subject.to_owned()],
                        ..Default::default()
                    })
                    .await
                    .expect("create Communications Event stream");
            }
            for consumer in plan.consumers() {
                let subject = consumer.subject().as_str();
                let stream_name = communications_stream_for_subject(&subject);
                context
                    .create_consumer_on_stream(
                        async_nats::jetstream::consumer::pull::Config {
                            durable_name: Some(consumer.durable_name().to_owned()),
                            filter_subject: subject,
                            ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                            ack_wait: Duration::from_millis(
                                consumer.delivery_policy().ack_wait_millis().into(),
                            ),
                            max_deliver: i64::from(consumer.delivery_policy().max_deliver()),
                            max_ack_pending: i64::from(consumer.max_in_flight()),
                            ..Default::default()
                        },
                        stream_name,
                    )
                    .await
                    .expect("create Communications Event consumer");
            }
        });
}

pub(super) fn start_communications_domain(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> u64 {
    let reservation = managed_launch::load(supervisor, store, COMMUNICATIONS_REGISTRATION)
        .expect("load Communications reservation");
    let binding = communications_storage_binding(store);
    let topology = crate::platform::storage::topology::current(store)
        .expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Communications Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: COMMUNICATIONS_OWNER_ID.to_owned(),
            registration_id: COMMUNICATIONS_REGISTRATION.to_owned(),
            runtime_instance_id: COMMUNICATIONS_RUNTIME_INSTANCE_ID.to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
        },
    )
    .expect("start Communications domain")
}

pub(super) fn assert_communications_query_delivery(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let payload = CommunicationsQueryRequestV1 {
        protocol_major: 1,
        operation: Some(Operation::ListAccounts(ListAccountsRequestV1 { limit: 16 })),
    }
    .encode_to_vec();
    let request = encode_module_query_request_v1(1, &payload)
        .expect("encode Communications query module request");
    let launch = store
        .effective_managed_launch_record(COMMUNICATIONS_REGISTRATION)
        .expect("read Communications launch")
        .expect("Communications launch is active");
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        COMMUNICATIONS_REGISTRATION,
        launch.runtime_instance_id(),
        launch.runtime_generation(),
        launch.grant_epoch(),
        COMMUNICATIONS_QUERY_CAPABILITY_ID,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route exact Communications owner query");
    let response = ModuleClientResponseV1::decode(bytes.as_slice())
        .expect("decode Communications module response");
    assert_eq!(response.request_id, 1);
    assert!(response.error_code.is_empty());
    let query = CommunicationsQueryResponseV1::decode(response.response_payload.as_slice())
        .expect("decode Communications query response");
    assert!(matches!(query.result, Some(QueryResult::ListAccounts(accounts)) if accounts.accounts.is_empty()));
}

fn record_communications_registration(store: &SqliteControlStore, descriptor: &[u8]) -> u64 {
    let registration = ModuleRegistration::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_MODULE_ID,
        Sha256::digest(descriptor).into(),
        ModuleRegistrationState::Pending,
        1,
    );
    let capabilities = [
        COMMUNICATIONS_EVENTS_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_OBSERVE_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_STORAGE_CAPABILITY_ID.to_owned(),
        "communications.blob.v1".to_owned(),
        "communications.query.v1".to_owned(),
    ];
    let storage = ModuleStorageRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        8,
        5_000,
    );
    let recorded = communication_evidence_recorded_contract_reference_v1();
    let observed = hermes_communications_ingress::admission::communication_observed_contract_reference_v1();
    let routes = [
        communications_event_route(
            COMMUNICATIONS_EVENTS_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Event,
            &recorded,
            ModuleEventRouteDirectionV1::Publish,
        ),
        communications_event_route(
            COMMUNICATIONS_OBSERVE_CAPABILITY_ID,
            ModuleEventEnvelopeKindV1::Observation,
            &observed,
            ModuleEventRouteDirectionV1::Consume,
        ),
    ];
    store
        .create_pending_registration_with_requests(
            &registration,
            &capabilities,
            std::slice::from_ref(&storage),
            &routes,
            &[],
        )
        .expect("record Communications registration");
    store
        .approve_module_registration(COMMUNICATIONS_REGISTRATION, &capabilities)
        .expect("approve Communications capabilities")
        .grant_epoch()
}

fn record_communications_runtime_fixture(
    store: &SqliteControlStore,
    schema: &[u8],
    descriptor: &[u8],
    grant_epoch: u64,
) {
    let canonical_bundle = communications_storage_bundle_v1().encode_to_vec();
    let digest: [u8; 32] = Sha256::digest(&canonical_bundle).into();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new("communications", 1, digest, canonical_bundle)
                .expect("record Communications Storage bundle"),
        )
        .expect("persist Communications Storage bundle");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            COMMUNICATIONS_REGISTRATION,
            1,
            "hermes-managed-runtime-conformance",
            "domain.communications",
            Sha256::digest(std::fs::read(communications_binary()).expect("Communications binary bytes"))
                .into(),
            Sha256::digest(descriptor).into(),
            Some(Sha256::digest(schema).into()),
        ))
        .expect("record Communications release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            COMMUNICATIONS_REGISTRATION,
            COMMUNICATIONS_RUNTIME_INSTANCE_ID,
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record Communications reservation");
    store
        .record_platform_event_hub_topology(&communications_event_hub_topology())
        .expect("record Event Hub topology");
}

fn communications_event_route(
    capability: &str,
    kind: ModuleEventEnvelopeKindV1,
    contract: &hermes_runtime_protocol::v1::ContractReferenceV1,
    direction: ModuleEventRouteDirectionV1,
) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(hermes_kernel_control_store::ModuleEventRouteRequestInputV1 {
        registration_id: COMMUNICATIONS_REGISTRATION.to_owned(),
        capability_id: capability.to_owned(),
        envelope_kind: kind,
        contract_owner: contract.owner.clone(),
        contract_name: contract.name.clone(),
        contract_major: contract.major,
        contract_revision: contract.revision,
        contract_schema_sha256: contract.schema_sha256.as_slice().try_into().expect("contract digest"),
        direction,
        max_in_flight: 16,
        delivery_policy: matches!(direction, ModuleEventRouteDirectionV1::Consume).then(|| {
            ModuleEventDeliveryPolicyV1::new(ModuleEventSubscriptionRequirementV1::Required, 8, 30_000)
        }),
    })
}

fn communications_event_hub_topology() -> PlatformEventHubTopologyV1 {
    let budgets = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ]
    .into_iter()
    .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
    .collect();
    PlatformEventHubTopologyV1::new(
        1,
        required("HERMES_COMMUNICATIONS_LIVE_NATS_ENDPOINT"),
        COMMUNICATIONS_OWNER_ID,
        1,
        budgets,
    )
}

pub(super) fn installed_communications_release(root: &Path) -> InstalledSignedBundle {
    let schema = communications_settings_schema_bytes_v1();
    InstalledSignedBundle::install(
        root,
        &[
            SignedRuntimeArtifact::new(
                "platform.storage",
                storage_binary(),
                descriptor("storage").encode_to_vec(),
            ),
            SignedRuntimeArtifact::new(
                "platform.vault",
                vault_binary(),
                descriptor("vault").encode_to_vec(),
            ),
            SignedRuntimeArtifact::new(
                "domain.communications",
                communications_binary(),
                communications_module_descriptor_v1("managed-communications-live").encode_to_vec(),
            )
            .with_settings_schema(schema),
        ],
    )
    .expect("install signed Communications release")
}

fn communications_binary() -> PathBuf {
    binary("HERMES_COMMUNICATIONS_RUNTIME_BIN")
}

fn communications_stream_details(
    kind: event_topology::subject::EventStreamKindV1,
) -> (&'static str, &'static str) {
    match kind {
        event_topology::subject::EventStreamKindV1::Command => {
            ("HERMES_COMMAND_V1", "hermes.command.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Event => {
            ("HERMES_EVENT_V1", "hermes.event.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Observation => {
            ("HERMES_OBSERVATION_V1", "hermes.observation.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Result => {
            ("HERMES_RESULT_V1", "hermes.result.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Ack => {
            ("HERMES_ACK_V1", "hermes.ack.v1.>")
        }
    }
}

fn communications_stream_for_subject(subject: &str) -> &'static str {
    if subject.starts_with("hermes.command.") {
        "HERMES_COMMAND_V1"
    } else if subject.starts_with("hermes.event.") {
        "HERMES_EVENT_V1"
    } else if subject.starts_with("hermes.observation.") {
        "HERMES_OBSERVATION_V1"
    } else if subject.starts_with("hermes.result.") {
        "HERMES_RESULT_V1"
    } else {
        "HERMES_ACK_V1"
    }
}
