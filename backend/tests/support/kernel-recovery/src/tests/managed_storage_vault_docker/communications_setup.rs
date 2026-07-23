use super::*;

use hermes_communications_api::query_wire::{
    CommunicationsQueryRequestV1, CommunicationsQueryResponseV1, ListAccountsRequestV1,
    SearchCommunicationsRequestV1,
    communications_query_request_v1::Operation,
    communications_query_response_v1::Result as QueryResult,
};
use hermes_communications_persistence::{
    COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1, communications_storage_bundle_v1,
};
use hermes_communications_runtime::admission::{
    COMMUNICATIONS_BLOB_CAPABILITY_ID, COMMUNICATIONS_BLOB_QUOTA_BYTES,
    COMMUNICATIONS_EVENTS_CAPABILITY_ID, COMMUNICATIONS_MODULE_ID,
    COMMUNICATIONS_OBSERVE_CAPABILITY_ID, COMMUNICATIONS_OWNER_ID,
    COMMUNICATIONS_STORAGE_CAPABILITY_ID, COMMUNICATIONS_QUERY_CAPABILITY_ID,
    COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID,
    communications_module_descriptor_v1,
    communications_settings_schema_bytes_v1,
    communication_evidence_recorded_contract_reference_v1,
};
use hermes_communications_runtime::query_client_port::encode_module_query_request_v1;
use hermes_kernel_control_store::{
    ModuleBlobQuotaRequestV1, ModuleRegistrationState, PlatformStorageBindingStateV1,
};
use hermes_runtime_protocol::v1::{
    BlobDataOperationV1, ManagedRuntimeBlobSessionRequestV1, ModuleClientResponseV1,
};
use hermes_blob_client::BlobDataClient;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};

pub(super) const COMMUNICATIONS_REGISTRATION: &str = "communications-runtime";
const COMMUNICATIONS_RUNTIME_INSTANCE_ID: &str = "02020202020202020202020202020202";
const FIXTURE_SOURCE_REGISTRATION: &str = "fixture-source-integration";
const FIXTURE_SOURCE_CAPABILITY_ID: &str = "fixture-source.blob.v1";
const FIXTURE_SOURCE_RUNTIME_INSTANCE_ID: &str = "03030303030303030303030303030303";
const FIXTURE_SOURCE_RUNTIME_INSTANCE_ID_V2: &str = "04040404040404040404040404040404";

pub(super) fn configured_communications_store(root: &Path, kernel: &Path) -> SqliteControlStore {
    let store = configured_store(root, kernel);
    crate::platform::blob::binding::bind_installed_release(&store, kernel)
        .expect("bind signed Blob release");
    let schema = communications_settings_schema_bytes_v1();
    let descriptor = communications_module_descriptor_v1("managed-communications-live").encode_to_vec();
    let grant_epoch = record_communications_registration(&store, &descriptor);
    record_communications_runtime_fixture(&store, &schema, &descriptor, grant_epoch);
    store
}

pub(super) fn issue_initial_communications_storage_binding(store: &SqliteControlStore) {
    let bundle = store
        .platform_storage_bundle("communications", u64::from(COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1))
        .expect("read Communications Storage bundle")
        .expect("Communications Storage bundle is present");
    let binding = issue_managed(
        store,
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_RUNTIME_INSTANCE_ID,
        1,
        COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
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
            grant_epoch: binding.grant_epoch(),
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
    let query = route_communications_query(store, supervisor, 1, &payload);
    assert!(matches!(query.result, Some(QueryResult::ListAccounts(accounts)) if accounts.accounts.is_empty()));
}

pub(super) fn assert_communications_search_query_delivery(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let payload = CommunicationsQueryRequestV1 {
        protocol_major: 1,
        operation: Some(Operation::SearchCommunications(SearchCommunicationsRequestV1 {
            query: "known-missing-token".to_owned(),
            limit: 16,
        })),
    }
    .encode_to_vec();
    let query = route_communications_query(store, supervisor, 2, &payload);
    assert!(matches!(query.result, Some(QueryResult::SearchCommunications(hits)) if hits.hits.is_empty()));
}

pub(super) fn assert_communications_ingress_delivery(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let draft = hermes_communications_ingress::new_scoped_communication_observation_draft(
        "managed-ingress-observation-1",
        hermes_communications_ingress::SourceEnvelope {
            provider: hermes_communications_ingress::ProviderProvenanceV1::MailImap,
            external_record_id: "integration-private-record-1".to_owned(),
            scope: Some(hermes_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-account-1".to_owned(),
                external_conversation_id: Some("integration-private-conversation-1".to_owned()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        hermes_communications_ingress::CommunicationEvidenceKindV1::EmailMessage,
        hermes_communications_ingress::BodyAvailabilityV1::MetadataOnly,
        hermes_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_000),
    )
    .expect("build typed integration ingress draft");
    let record = hermes_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &hermes_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build exact typed integration envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            use futures_util::StreamExt as _;

            let client = async_nats::connect(endpoint)
                .await
                .expect("connect disposable JetStream");
            let mut canonical_events = client
                .subscribe("hermes.event.v1.communications.communication_evidence_recorded.v1")
                .await
                .expect("subscribe to exact canonical event subject");
            let context = async_nats::jetstream::new(client);
            context
                .publish(
                    "hermes.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish exact typed integration envelope");
            let canonical = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                canonical_events.next(),
            )
            .await
            .unwrap_or_else(|_| panic!(
                "canonical Communications event timeout: active={:?}, failure={:?}",
                supervisor.is_active(COMMUNICATIONS_REGISTRATION),
                supervisor.last_failure(COMMUNICATIONS_REGISTRATION),
            ))
            .expect("canonical Communications event missing");
            let envelope = hermes_events_protocol::validation::envelope::decode_envelope_v1(
                canonical.payload.as_ref(),
            )
            .expect("canonical Communications envelope");
            assert!(matches!(
                envelope.contract.as_ref(),
                Some(contract)
                    if contract.owner == "communications"
                        && contract.name == "communication_evidence_recorded"
                        && contract.major == 1
                        && contract.revision == 1
            ));
            context
                .publish(
                    "hermes.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("republish exact typed integration envelope");
            assert!(
                tokio::time::timeout(
                    std::time::Duration::from_secs(1),
                    canonical_events.next(),
                )
                .await
                .is_err(),
                "duplicate ingress must not produce a second canonical event"
            );
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let payload = CommunicationsQueryRequestV1 {
            protocol_major: 1,
            operation: Some(Operation::ListAccounts(ListAccountsRequestV1 { limit: 16 })),
        }
        .encode_to_vec();
        let query = route_communications_query(store, supervisor, 3, &payload);
        if matches!(query.result, Some(QueryResult::ListAccounts(accounts)) if !accounts.accounts.is_empty()) {
            return;
        }
        assert!(std::time::Instant::now() < deadline, "typed integration ingress was not committed to Communications");
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn assert_communications_transferred_body_projection(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    kernel_data: &Path,
) {
    const OPAQUE_BLOB_REFERENCE: &str = "blob://fixture-source/admitted-body-1";
    let source_grant_epoch = record_fixture_source_integration(store);
    let plaintext = b"fixture source body for custody transfer";
    let plaintext_sha256: [u8; 32] = Sha256::digest(plaintext).into();
    let reference_id = [8; 16];
    let channel_binding = vec![6; 32];
    let delivery = BlobSessionHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
        kernel_data.to_path_buf(),
    )
    .issue_blob_session(
        &ManagedRuntimeExpectation::new(
            FIXTURE_SOURCE_REGISTRATION,
            FIXTURE_SOURCE_RUNTIME_INSTANCE_ID,
            "integration.fixture-source",
            1,
            source_grant_epoch,
            [3; 32],
            None,
        ),
        ManagedRuntimeBlobSessionRequestV1 {
            request_id: vec![4; 16],
            capability_id: FIXTURE_SOURCE_CAPABILITY_ID.to_owned(),
            operation: BlobDataOperationV1::BlobDataOperationWriteV1 as u32,
            channel_binding_sha256: Sha256::digest(&channel_binding).to_vec(),
            reference_id: reference_id.to_vec(),
            declared_size: u64::try_from(plaintext.len()).expect("fixture body size"),
            backup_class: 1,
            ttl_seconds: 30,
            receipt_sha256: plaintext_sha256.to_vec(),
            custody_source_proof: Vec::new(),
            evidence_id: Vec::new(),
            evidence_envelope_sha256: Vec::new(),
        },
    )
    .expect("issue source integration Blob write session");
    let source_proof = delivery.custody_transfer_source_proof;
    BlobDataClient::new(delivery.data_socket_path)
        .expect("open source Blob data client")
        .write(
            delivery.grant.expect("source Blob write grant"),
            channel_binding,
            plaintext.to_vec(),
        )
        .expect("write source integration Blob content");
    let rejected_draft = hermes_communications_ingress::new_scoped_communication_observation_draft(
        "managed-rejected-body-observation-1",
        hermes_communications_ingress::SourceEnvelope {
            provider: hermes_communications_ingress::ProviderProvenanceV1::Telegram,
            external_record_id: "integration-private-body-record-rejected-1".to_owned(),
            scope: Some(hermes_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-body-account-1".to_owned(),
                external_conversation_id: Some("integration-private-body-conversation-1".to_owned()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        hermes_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
        hermes_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
        hermes_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_000),
    )
    .expect("build rejected admitted-body ingress draft");
    let rejected_draft = hermes_communications_ingress::with_admitted_body_blob(
        rejected_draft,
        hermes_communications_ingress::BodyBlobReceiptV1 {
            blob_ref: OPAQUE_BLOB_REFERENCE.to_owned(),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
            sha256: [9; 32],
            custody_transfer_source_proof: source_proof.clone(),
        },
    )
    .expect("attach altered opaque Blob receipt");
    let rejected_record = hermes_communications_ingress::build_observation_outbox_record_v1(
        &rejected_draft,
        &hermes_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build altered admitted-body typed ingress envelope");
    let draft = hermes_communications_ingress::new_scoped_communication_observation_draft(
        "managed-admitted-body-observation-1",
        hermes_communications_ingress::SourceEnvelope {
            provider: hermes_communications_ingress::ProviderProvenanceV1::Telegram,
            external_record_id: "integration-private-body-record-1".to_owned(),
            scope: Some(hermes_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-body-account-1".to_owned(),
                external_conversation_id: Some("integration-private-body-conversation-1".to_owned()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        hermes_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
        hermes_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
        hermes_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_001),
    )
    .expect("build admitted-body ingress draft");
    let draft = hermes_communications_ingress::with_admitted_body_blob(
        draft,
        hermes_communications_ingress::BodyBlobReceiptV1 {
            blob_ref: OPAQUE_BLOB_REFERENCE.to_owned(),
            reference_id,
            declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
            sha256: plaintext_sha256,
            custody_transfer_source_proof: source_proof.clone(),
        },
    )
    .expect("attach admitted opaque Blob receipt");
    let record = hermes_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &hermes_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_001,
            recorded_at_nanos: 0,
        },
    )
    .expect("build admitted-body typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect disposable JetStream"),
            );
            context
                .publish(
                    "hermes.observation.v1.communications.communication_observed.v1",
                    rejected_record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish altered admitted-body typed ingress envelope");
            context
                .publish(
                    "hermes.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish admitted-body typed ingress envelope");
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut stale_source_published = false;
    let mut revoked_source_published = false;
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            4,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 { limit: 16 })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        let Some(account) = accounts.accounts.iter().find(|account| account.provider == 2) else {
            assert!(std::time::Instant::now() < deadline, "admitted body account was not projected");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let conversations = route_communications_query(
            store,
            supervisor,
            5,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversations(
                    hermes_communications_api::query_wire::ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
            panic!("Communications conversations query result");
        };
        let Some(conversation) = conversations.conversations.first() else {
            assert!(std::time::Instant::now() < deadline, "admitted body conversation was not projected");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let messages = route_communications_query(
            store,
            supervisor,
            6,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationMessages(
                    hermes_communications_api::query_wire::ListConversationMessagesRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
            panic!("Communications messages query result");
        };
        let transferred = messages.messages.iter().any(|message| message.body_state == 4);
        let rejected = messages
            .messages
            .iter()
            .filter(|message| message.body_state == 3)
            .count();
        if transferred && rejected >= 1 && !stale_source_published {
            store
                .record_managed_launch(&ManagedLaunchRecord::new(
                    FIXTURE_SOURCE_REGISTRATION,
                    FIXTURE_SOURCE_RUNTIME_INSTANCE_ID_V2,
                    1,
                    1,
                    2,
                    source_grant_epoch,
                ))
                .expect("record fixture source integration successor launch");
            let stale_draft = hermes_communications_ingress::new_scoped_communication_observation_draft(
                "managed-stale-body-observation-1",
                hermes_communications_ingress::SourceEnvelope {
                    provider: hermes_communications_ingress::ProviderProvenanceV1::Telegram,
                    external_record_id: "integration-private-body-record-stale-1".to_owned(),
                    scope: Some(hermes_communications_ingress::SourceScopeEnvelope {
                        external_account_id: "integration-private-body-account-1".to_owned(),
                        external_conversation_id: Some("integration-private-body-conversation-1".to_owned()),
                        external_participant_id: None,
                        external_media_id: None,
                        external_reply_to_record_id: None,
                        external_forward_origin_record_id: None,
                    }),
                },
                hermes_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
                hermes_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
                hermes_communications_ingress::CommunicationDirectionV1::Incoming,
                Some(1_783_024_002),
            )
            .expect("build stale admitted-body ingress draft");
            let stale_draft = hermes_communications_ingress::with_admitted_body_blob(
                stale_draft,
                hermes_communications_ingress::BodyBlobReceiptV1 {
                    blob_ref: OPAQUE_BLOB_REFERENCE.to_owned(),
                    reference_id,
                    declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
                    sha256: plaintext_sha256,
                    custody_transfer_source_proof: source_proof.clone(),
                },
            )
            .expect("attach stale source opaque Blob receipt");
            let stale_record = hermes_communications_ingress::build_observation_outbox_record_v1(
                &stale_draft,
                &hermes_communications_ingress::ObservationEnvelopeContextV1 {
                    runtime_instance_id: "integration-test-runtime-1".to_owned(),
                    runtime_generation: 1,
                    module_id: "integration-test-runtime".to_owned(),
                    recorded_at_unix_seconds: 1_783_024_002,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build stale source typed ingress envelope");
            let endpoint = store
                .platform_event_hub_topology()
                .expect("read Event Hub topology")
                .expect("Event Hub topology")
                .nats_endpoint()
                .to_owned();
            tokio::runtime::Runtime::new()
                .expect("Tokio runtime")
                .block_on(async move {
                    async_nats::jetstream::new(
                        async_nats::connect(endpoint)
                            .await
                            .expect("connect disposable JetStream"),
                    )
                    .publish(
                        "hermes.observation.v1.communications.communication_observed.v1",
                        stale_record.exact_bytes().to_vec().into(),
                    )
                    .await
                    .expect("publish stale source typed ingress envelope");
                });
            stale_source_published = true;
            continue;
        }
        if transferred && rejected >= 2 && !revoked_source_published {
            let current_reference_id = [9; 16];
            let current_channel_binding = vec![7; 32];
            let current_delivery = BlobSessionHandlerV1::new(
                Arc::clone(store),
                supervisor.relay_port(),
                kernel_data.to_path_buf(),
            )
            .issue_blob_session(
                &ManagedRuntimeExpectation::new(
                    FIXTURE_SOURCE_REGISTRATION,
                    FIXTURE_SOURCE_RUNTIME_INSTANCE_ID_V2,
                    "integration.fixture-source",
                    2,
                    source_grant_epoch,
                    [3; 32],
                    None,
                ),
                ManagedRuntimeBlobSessionRequestV1 {
                    request_id: vec![5; 16],
                    capability_id: FIXTURE_SOURCE_CAPABILITY_ID.to_owned(),
                    operation: BlobDataOperationV1::BlobDataOperationWriteV1 as u32,
                    channel_binding_sha256: Sha256::digest(&current_channel_binding).to_vec(),
                    reference_id: current_reference_id.to_vec(),
                    declared_size: u64::try_from(plaintext.len()).expect("fixture body size"),
                    backup_class: 1,
                    ttl_seconds: 30,
                    receipt_sha256: plaintext_sha256.to_vec(),
                    custody_source_proof: Vec::new(),
                    evidence_id: Vec::new(),
                    evidence_envelope_sha256: Vec::new(),
                },
            )
            .expect("issue successor source integration Blob write session");
            let current_source_proof = current_delivery.custody_transfer_source_proof;
            BlobDataClient::new(current_delivery.data_socket_path)
                .expect("open successor source Blob data client")
                .write(
                    current_delivery.grant.expect("successor source Blob write grant"),
                    current_channel_binding,
                    plaintext.to_vec(),
                )
                .expect("write successor source Blob content");
            store
                .transition_module_registration(
                    FIXTURE_SOURCE_REGISTRATION,
                    ModuleRegistrationState::Revoked,
                )
                .expect("revoke fixture source integration");
            let revoked_draft = hermes_communications_ingress::new_scoped_communication_observation_draft(
                "managed-revoked-body-observation-1",
                hermes_communications_ingress::SourceEnvelope {
                    provider: hermes_communications_ingress::ProviderProvenanceV1::Telegram,
                    external_record_id: "integration-private-body-record-revoked-1".to_owned(),
                    scope: Some(hermes_communications_ingress::SourceScopeEnvelope {
                        external_account_id: "integration-private-body-account-1".to_owned(),
                        external_conversation_id: Some("integration-private-body-conversation-1".to_owned()),
                        external_participant_id: None,
                        external_media_id: None,
                        external_reply_to_record_id: None,
                        external_forward_origin_record_id: None,
                    }),
                },
                hermes_communications_ingress::CommunicationEvidenceKindV1::ChatMessage,
                hermes_communications_ingress::BodyAvailabilityV1::AdmittedBlob,
                hermes_communications_ingress::CommunicationDirectionV1::Incoming,
                Some(1_783_024_002),
            )
            .expect("build revoked admitted-body ingress draft");
            let revoked_draft = hermes_communications_ingress::with_admitted_body_blob(
                revoked_draft,
                hermes_communications_ingress::BodyBlobReceiptV1 {
                    blob_ref: OPAQUE_BLOB_REFERENCE.to_owned(),
                    reference_id: current_reference_id,
                    declared_bytes: u64::try_from(plaintext.len()).expect("fixture body size"),
                    sha256: plaintext_sha256,
                    custody_transfer_source_proof: current_source_proof,
                },
            )
            .expect("attach revoked source opaque Blob receipt");
            let revoked_record = hermes_communications_ingress::build_observation_outbox_record_v1(
                &revoked_draft,
                &hermes_communications_ingress::ObservationEnvelopeContextV1 {
                    runtime_instance_id: "integration-test-runtime-1".to_owned(),
                    runtime_generation: 1,
                    module_id: "integration-test-runtime".to_owned(),
                    recorded_at_unix_seconds: 1_783_024_002,
                    recorded_at_nanos: 0,
                },
            )
            .expect("build revoked source typed ingress envelope");
            let endpoint = store
                .platform_event_hub_topology()
                .expect("read Event Hub topology")
                .expect("Event Hub topology")
                .nats_endpoint()
                .to_owned();
            tokio::runtime::Runtime::new()
                .expect("Tokio runtime")
                .block_on(async move {
                    async_nats::jetstream::new(
                        async_nats::connect(endpoint)
                            .await
                            .expect("connect disposable JetStream"),
                    )
                    .publish(
                        "hermes.observation.v1.communications.communication_observed.v1",
                        revoked_record.exact_bytes().to_vec().into(),
                    )
                    .await
                    .expect("publish revoked source typed ingress envelope");
                });
            revoked_source_published = true;
            continue;
        }
        if transferred && rejected >= 3 {
            let public_payload = CommunicationsQueryResponseV1 {
                result: Some(QueryResult::ListConversationMessages(messages)),
                error_code: String::new(),
            }
            .encode_to_vec();
            assert!(
                !public_payload
                    .windows(OPAQUE_BLOB_REFERENCE.len())
                    .any(|window| window == OPAQUE_BLOB_REFERENCE.as_bytes()),
                "public Communications query must not reveal an owner-private Blob reference",
            );
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "custody transfer must retain a policy-rejected body without blocking a valid body"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn assert_communications_attachment_anchor_projection(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    const PROVIDER_MEDIA_LOCATOR: &str = "integration-private-media-1";
    let draft = hermes_communications_ingress::new_scoped_communication_observation_draft(
        "managed-attachment-observation-1",
        hermes_communications_ingress::SourceEnvelope {
            provider: hermes_communications_ingress::ProviderProvenanceV1::MailImap,
            // A media mutation updates the message established by the earlier Mail observation.
            external_record_id: "integration-private-record-1".to_owned(),
            scope: Some(hermes_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-account-1".to_owned(),
                external_conversation_id: Some("integration-private-conversation-1".to_owned()),
                external_participant_id: None,
                external_media_id: Some(PROVIDER_MEDIA_LOCATOR.to_owned()),
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        hermes_communications_ingress::CommunicationEvidenceKindV1::MediaChanged,
        hermes_communications_ingress::BodyAvailabilityV1::MetadataOnly,
        hermes_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_002),
    )
    .expect("build attachment ingress draft");
    let draft = hermes_communications_ingress::with_attachment_descriptor(
        draft,
        hermes_communications_ingress::AttachmentDescriptorV1 {
            filename: Some("evidence.txt".to_owned()),
            media_type: "text/plain".to_owned(),
            declared_bytes: 32,
            sha256: Some([10; 32]),
            disposition: hermes_communications_ingress::AttachmentDispositionV1::Attachment,
        },
    )
    .expect("attach typed attachment descriptor");
    let record = hermes_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &hermes_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_002,
            recorded_at_nanos: 0,
        },
    )
    .expect("build attachment typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect disposable JetStream"),
            );
            context
                .publish(
                    "hermes.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish attachment typed ingress envelope");
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            7,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 { limit: 16 })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        let Some(account) = accounts.accounts.iter().find(|account| account.provider == 1) else {
            assert!(std::time::Instant::now() < deadline, "attachment account was not projected");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let conversations = route_communications_query(
            store,
            supervisor,
            8,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversations(
                    hermes_communications_api::query_wire::ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
            panic!("Communications conversations query result");
        };
        let Some(conversation) = conversations.conversations.first() else {
            assert!(std::time::Instant::now() < deadline, "attachment conversation was not projected");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let messages = route_communications_query(
            store,
            supervisor,
            9,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationMessages(
                    hermes_communications_api::query_wire::ListConversationMessagesRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
            panic!("Communications messages query result");
        };
        for message in messages.messages {
            let anchors = route_communications_query(
                store,
                supervisor,
                10,
                &CommunicationsQueryRequestV1 {
                    protocol_major: 1,
                    operation: Some(Operation::ListMessageAttachmentAnchors(
                        hermes_communications_api::query_wire::ListMessageAttachmentAnchorsRequestV1 {
                            message_id: message.message_id,
                            limit: 16,
                        },
                    )),
                }
                .encode_to_vec(),
            );
            let Some(QueryResult::ListMessageAttachmentAnchors(anchors)) = anchors.result else {
                panic!("Communications attachment anchors query result");
            };
            if let Some(anchor) = anchors.anchors.iter().find(|anchor| {
                anchor.has_descriptor
                    && anchor.filename == "evidence.txt"
                    && anchor.media_type == "text/plain"
                    && anchor.declared_bytes == 32
                    && anchor.sha256 == vec![10; 32]
                    && anchor.disposition == 1
            }) {
                let public_payload = CommunicationsQueryResponseV1 {
                    result: Some(QueryResult::ListMessageAttachmentAnchors(
                        hermes_communications_api::query_wire::ListMessageAttachmentAnchorsResponseV1 {
                            anchors: vec![anchor.clone()],
                        },
                    )),
                    error_code: String::new(),
                }
                .encode_to_vec();
                assert!(
                    !public_payload
                        .windows(PROVIDER_MEDIA_LOCATOR.len())
                        .any(|window| window == PROVIDER_MEDIA_LOCATOR.as_bytes()),
                    "public Communications anchor must not reveal a provider-local media locator",
                );
                return;
            }
        }
        assert!(std::time::Instant::now() < deadline, "attachment anchor was not projected");
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

pub(super) fn assert_communications_relationship_projection(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    const PRIVATE_PARTICIPANT_ID: &str = "integration-private-participant-1";
    const PRIVATE_REPLY_RECORD_ID: &str = "integration-private-reply-1";
    const PRIVATE_FORWARD_RECORD_ID: &str = "integration-private-forward-1";
    let draft = hermes_communications_ingress::new_scoped_communication_observation_draft(
        "managed-relationship-observation-1",
        hermes_communications_ingress::SourceEnvelope {
            provider: hermes_communications_ingress::ProviderProvenanceV1::Zulip,
            external_record_id: "integration-private-relationship-record-1".to_owned(),
            scope: Some(hermes_communications_ingress::SourceScopeEnvelope {
                external_account_id: "integration-private-relationship-account-1".to_owned(),
                external_conversation_id: Some("integration-private-relationship-conversation-1".to_owned()),
                external_participant_id: Some(PRIVATE_PARTICIPANT_ID.to_owned()),
                external_media_id: None,
                external_reply_to_record_id: Some(PRIVATE_REPLY_RECORD_ID.to_owned()),
                external_forward_origin_record_id: Some(PRIVATE_FORWARD_RECORD_ID.to_owned()),
            }),
        },
        hermes_communications_ingress::CommunicationEvidenceKindV1::EmailMessage,
        hermes_communications_ingress::BodyAvailabilityV1::MetadataOnly,
        hermes_communications_ingress::CommunicationDirectionV1::Incoming,
        Some(1_783_024_003),
    )
    .expect("build relationship ingress draft");
    let record = hermes_communications_ingress::build_observation_outbox_record_v1(
        &draft,
        &hermes_communications_ingress::ObservationEnvelopeContextV1 {
            runtime_instance_id: "integration-test-runtime-1".to_owned(),
            runtime_generation: 1,
            module_id: "integration-test-runtime".to_owned(),
            recorded_at_unix_seconds: 1_783_024_003,
            recorded_at_nanos: 0,
        },
    )
    .expect("build relationship typed ingress envelope");
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(endpoint)
                    .await
                    .expect("connect disposable JetStream"),
            );
            context
                .publish(
                    "hermes.observation.v1.communications.communication_observed.v1",
                    record.exact_bytes().to_vec().into(),
                )
                .await
                .expect("publish relationship typed ingress envelope");
        });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            11,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 { limit: 16 })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        let Some(account) = accounts.accounts.iter().find(|account| account.provider == 5) else {
            assert!(std::time::Instant::now() < deadline, "relationship account was not projected");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let conversations = route_communications_query(
            store,
            supervisor,
            12,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversations(
                    hermes_communications_api::query_wire::ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
            panic!("Communications conversations query result");
        };
        let Some(conversation) = conversations.conversations.first() else {
            assert!(std::time::Instant::now() < deadline, "relationship conversation was not projected");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let participants = route_communications_query(
            store,
            supervisor,
            13,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationParticipants(
                    hermes_communications_api::query_wire::ListConversationParticipantsRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationParticipants(participants)) = participants.result else {
            panic!("Communications participants query result");
        };
        let messages = route_communications_query(
            store,
            supervisor,
            14,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListConversationMessages(
                    hermes_communications_api::query_wire::ListConversationMessagesRequestV1 {
                        conversation_id: conversation.conversation_id.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
            panic!("Communications messages query result");
        };
        let Some(message) = messages.messages.first() else {
            assert!(std::time::Instant::now() < deadline, "relationship message was not projected");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        };
        let references = route_communications_query(
            store,
            supervisor,
            15,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListMessageReferences(
                    hermes_communications_api::query_wire::ListMessageReferencesRequestV1 {
                        message_id: message.message_id.clone(),
                        limit: 16,
                    },
                )),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListMessageReferences(references)) = references.result else {
            panic!("Communications references query result");
        };
        if !participants.participants.is_empty()
            && references.references.iter().any(|reference| reference.kind == 1)
            && references.references.iter().any(|reference| reference.kind == 2)
        {
            let participant_payload = CommunicationsQueryResponseV1 {
                result: Some(QueryResult::ListConversationParticipants(participants)),
                error_code: String::new(),
            }
            .encode_to_vec();
            let reference_payload = CommunicationsQueryResponseV1 {
                result: Some(QueryResult::ListMessageReferences(references)),
                error_code: String::new(),
            }
            .encode_to_vec();
            for private_id in [PRIVATE_PARTICIPANT_ID, PRIVATE_REPLY_RECORD_ID, PRIVATE_FORWARD_RECORD_ID] {
                assert!(
                    !participant_payload.windows(private_id.len()).any(|window| window == private_id.as_bytes())
                        && !reference_payload.windows(private_id.len()).any(|window| window == private_id.as_bytes()),
                    "public Communications relationships must not reveal provider-local identifiers",
                );
            }
            return;
        }
        assert!(std::time::Instant::now() < deadline, "relationship projections were not committed");
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn route_communications_query(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    request_id: u64,
    payload: &[u8],
) -> CommunicationsQueryResponseV1 {
    let request = encode_module_query_request_v1(request_id, payload)
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
    assert_eq!(response.request_id, request_id);
    assert!(response.error_code.is_empty());
    CommunicationsQueryResponseV1::decode(response.response_payload.as_slice())
        .expect("decode Communications query response")
}

fn record_communications_registration(store: &SqliteControlStore, descriptor: &[u8]) -> u64 {
    let registration = ModuleRegistration::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_MODULE_ID,
        COMMUNICATIONS_OWNER_ID,
        Sha256::digest(descriptor).into(),
        ModuleRegistrationState::Pending,
        1,
    );
    let capabilities = [
        "communications.blob.v1".to_owned(),
        COMMUNICATIONS_EVENTS_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_OBSERVE_CAPABILITY_ID.to_owned(),
        "communications.query.v1".to_owned(),
        COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID.to_owned(),
        COMMUNICATIONS_STORAGE_CAPABILITY_ID.to_owned(),
    ];
    let storage = ModuleStorageRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_STORAGE_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        8,
        5_000,
    );
    let blob = hermes_kernel_control_store::ModuleBlobQuotaRequestV1::new(
        COMMUNICATIONS_REGISTRATION,
        COMMUNICATIONS_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
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
            std::slice::from_ref(&blob),
        )
        .expect("record Communications registration");
    store
        .approve_module_registration(COMMUNICATIONS_REGISTRATION, &capabilities)
        .expect("approve Communications capabilities")
        .grant_epoch()
}

fn record_fixture_source_integration(store: &SqliteControlStore) -> u64 {
    let registration = ModuleRegistration::new(
        FIXTURE_SOURCE_REGISTRATION,
        "integration.fixture-source",
        COMMUNICATIONS_OWNER_ID,
        Sha256::digest(b"fixture-source-integration").into(),
        ModuleRegistrationState::Pending,
        1,
    );
    let capabilities = [FIXTURE_SOURCE_CAPABILITY_ID.to_owned()];
    let blob = ModuleBlobQuotaRequestV1::new(
        FIXTURE_SOURCE_REGISTRATION,
        FIXTURE_SOURCE_CAPABILITY_ID,
        COMMUNICATIONS_OWNER_ID,
        COMMUNICATIONS_BLOB_QUOTA_BYTES,
    );
    store
        .create_pending_registration_with_requests(
            &registration,
            &capabilities,
            &[],
            &[],
            std::slice::from_ref(&blob),
        )
        .expect("record fixture source integration registration");
    let grant_epoch = store
        .approve_module_registration(FIXTURE_SOURCE_REGISTRATION, &capabilities)
        .expect("approve fixture source integration capability")
        .grant_epoch();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            FIXTURE_SOURCE_REGISTRATION,
            1,
            "fixture-source-distribution",
            "integration.fixture-source",
            Sha256::digest(b"fixture-source-integration-binary").into(),
            Sha256::digest(b"fixture-source-integration").into(),
            None,
        ))
        .expect("record fixture source integration release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            FIXTURE_SOURCE_REGISTRATION,
            FIXTURE_SOURCE_RUNTIME_INSTANCE_ID,
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record fixture source integration launch");
    grant_epoch
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
            &PlatformStorageBundleV1::new(
                "communications", u64::from(COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1), digest, canonical_bundle,
            )
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
                "platform.blob",
                blob_binary(),
                blob_descriptor(),
            )
            .with_settings_schema(blob_settings_schema()),
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

fn blob_binary() -> PathBuf {
    binary("HERMES_BLOB_SERVICE_BIN")
}

fn blob_settings_schema() -> Vec<u8> {
    hermes_runtime_protocol::v1::SettingsSchemaV1 {
        major: 1,
        revision: 1,
        ..Default::default()
    }
    .encode_to_vec()
}

fn blob_descriptor() -> Vec<u8> {
    let schema = blob_settings_schema();
    hermes_runtime_protocol::v1::ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "blob".to_owned(),
        owner_id: "blob".to_owned(),
        module_kind: hermes_runtime_protocol::v1::ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "managed-communications-blob".to_owned(),
        settings_schema_ref: Some(hermes_runtime_protocol::v1::SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: schema.len() as u64,
            sha256: Sha256::digest(schema).to_vec(),
        }),
        ..Default::default()
    }
    .encode_to_vec()
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
