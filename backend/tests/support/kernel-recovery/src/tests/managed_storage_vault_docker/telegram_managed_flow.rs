//! Live managed Telegram process through Kernel leases into managed Communications.

use super::*;

use hermes_events_protocol::validation::envelope::decode_envelope_v1;
use hermes_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use hermes_telegram_api::{
    TelegramClientRequest, TelegramClientResponse, TelegramOperationState, TelegramProviderCommand,
    TelegramProviderQuery, TelegramProviderQueryResponse, TelegramRuntimeState,
    TelegramSendMessage, client_contract::TelegramClientContractV1,
};
use hermes_telegram_automation_api::{
    contract::{
        TELEGRAM_AUTOMATION_CONTRACT_MAJOR, TELEGRAM_AUTOMATION_CONTRACT_REVISION,
        TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1, TELEGRAM_AUTOMATION_MODULE_ID,
        TELEGRAM_AUTOMATION_OWNER_ID, TelegramAutomationContractV1,
    },
    wire::{
        AutomationCommandRequestV1, AutomationCommandResponseV1, AutomationFailureCodeV1,
        AutomationPolicyV1, AutomationPreviewReceiptV1, AutomationQueryRequestV1,
        AutomationQueryResponseV1, AutomationTemplateV1, AutomationVariableV1,
        GetAutomationPreviewReceiptQueryV1, ListAutomationPoliciesQueryV1,
        ListAutomationTemplatesQueryV1, PreviewAutomationPolicyCommandV1,
        UpsertAutomationPolicyCommandV1, UpsertAutomationTemplateCommandV1,
        automation_command_request_v1, automation_command_response_v1, automation_query_request_v1,
        automation_query_response_v1,
    },
};
use hermes_telegram_runtime::client_port::{
    TelegramClientPortError, decode_module_response, encode_module_request,
};
use prost::Message as _;
use sha2::Digest as _;

const AUTOMATION_TEMPLATE_ID: &str = "managed-template-1";
const AUTOMATION_POLICY_ID: &str = "managed-policy-1";
const AUTOMATION_PREVIEW_ID: &str = "managed-preview-1";
const AUTOMATION_CHAT_ID: &str = "telegram-chat-1";
const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

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

struct PreparedManagedTelegramFixture {
    root: PathBuf,
    data: PathBuf,
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    admitted_telegram: Option<AdmittedTelegramRuntime>,
}

impl PreparedManagedTelegramFixture {
    fn start_telegram(&mut self) -> StartedTelegramRuntime {
        start_telegram_runtime(
            &self.supervisor,
            &self.store,
            &self.data,
            &self.root.join("runtime"),
            self.admitted_telegram
                .take()
                .expect("prepared Telegram admission"),
        )
    }

    fn restart_telegram(&self, predecessor: StartedTelegramRuntime) -> StartedTelegramRuntime {
        restart_telegram_runtime(
            &self.supervisor,
            &self.store,
            &self.data,
            &self.root.join("runtime"),
            predecessor,
        )
    }
}

impl Drop for PreparedManagedTelegramFixture {
    fn drop(&mut self) {
        let _ = self.supervisor.shutdown();
        unsafe {
            std::env::remove_var("HERMES_TEST_KERNEL_EXECUTABLE");
        }
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.data);
    }
}

fn prepare_managed_telegram_fixture() -> PreparedManagedTelegramFixture {
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
    PreparedManagedTelegramFixture {
        root,
        data,
        store,
        supervisor,
        admitted_telegram: Some(admitted_telegram),
    }
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_runtime_uses_kernel_leases_and_event_only_communications_handoff() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
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

    let telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    assert_telegram_account_started(&store, &fixture.supervisor, &telegram);

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
        let duplicate_observation =
            tokio::time::timeout(Duration::from_secs(1), observations.next())
                .await
                .expect("duplicate Telegram observation timeout")
                .expect("duplicate Telegram observation");
        let duplicate_observation = decode_envelope_v1(duplicate_observation.payload.as_ref())
            .expect("duplicate Telegram observation envelope");
        assert_eq!(
            duplicate_observation.message_id, observation.message_id,
            "the observer must drain the exact duplicate before the outage replay"
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate Telegram observation must not create a second Communications event"
        );
    });
    let initial_evidence_id = assert_communications_query_delivery(&store, &fixture.supervisor);

    set_authenticated_nats_container_running(false);
    const OUTAGE_OPERATION_ID: &str = "managed-telegram-outage-send-1";
    assert_telegram_command_accepted(
        &store,
        &fixture.supervisor,
        &telegram,
        OUTAGE_OPERATION_ID,
        "managed Telegram outage replay trigger",
    );
    assert_telegram_operation_completed(
        &store,
        &fixture.supervisor,
        &telegram,
        OUTAGE_OPERATION_ID,
    );
    std::thread::sleep(Duration::from_millis(2_500));
    assert_telegram_operation_completed(
        &store,
        &fixture.supervisor,
        &telegram,
        OUTAGE_OPERATION_ID,
    );
    set_authenticated_nats_container_running(true);

    let (replayed_observation, replayed_canonical) = event_runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .expect("replayed Telegram observation timeout")
            .expect("replayed Telegram observation");
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("replayed Communications event timeout")
            .expect("replayed Communications event");
        (observation, canonical)
    });
    let replayed_observation = decode_envelope_v1(replayed_observation.payload.as_ref())
        .expect("replayed Telegram observation envelope");
    let replayed_canonical = decode_envelope_v1(replayed_canonical.payload.as_ref())
        .expect("replayed Communications event envelope");
    assert_eq!(
        replayed_canonical.causation_message_id, replayed_observation.message_id,
        "Communications replay must retain typed Telegram causation"
    );
    assert_ne!(
        replayed_canonical.message_id, canonical.message_id,
        "the outage replay must deliver the second provider observation"
    );
    let replayed_evidence_id = assert_communications_query_delivery(&store, &fixture.supervisor);
    assert_ne!(
        replayed_evidence_id, initial_evidence_id,
        "Communications durable query must expose the replayed evidence"
    );
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Telegram binaries"]
fn managed_telegram_automation_route_is_durable_and_provider_side_effect_free() {
    let mut fixture = prepare_managed_telegram_fixture();
    let store = Arc::clone(&fixture.store);
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("Event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect automation event observer");
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

    let telegram = fixture.start_telegram();
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    let (baseline_observation, baseline_canonical) = event_runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .expect("baseline Telegram observation timeout")
            .expect("baseline Telegram observation");
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("baseline Communications event timeout")
            .expect("baseline Communications event");
        (observation, canonical)
    });
    let baseline_observation = decode_envelope_v1(baseline_observation.payload.as_ref())
        .expect("baseline Telegram observation envelope");
    let baseline_canonical = decode_envelope_v1(baseline_canonical.payload.as_ref())
        .expect("baseline Communications event envelope");
    assert_eq!(
        baseline_canonical.causation_message_id, baseline_observation.message_id,
        "baseline Communications event must derive from the startup Telegram observation"
    );

    let automation = assert_telegram_automation_management(&store, &fixture.supervisor, &telegram);
    event_runtime.block_on(async {
        assert!(
            tokio::time::timeout(Duration::from_millis(500), observations.next())
                .await
                .is_err(),
            "Telegram automation preview must not emit a provider observation"
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(500), canonical_events.next())
                .await
                .is_err(),
            "Telegram automation preview must not create Communications evidence"
        );
    });

    let stale_runtime = telegram.clone();
    let telegram = fixture.restart_telegram(telegram);
    assert_telegram_lifecycle_query(&store, &fixture.supervisor, &telegram);
    let replay = route_telegram_automation_until_ready(
        &store,
        &fixture.supervisor,
        &telegram,
        TelegramAutomationContractV1::Command,
        81,
        &automation.template_request,
    );
    assert_eq!(
        replay, automation.template_response,
        "Telegram automation retry must replay exact response bytes after process restart"
    );
    assert_automation_query_projection(
        &store,
        &fixture.supervisor,
        &telegram,
        &automation.template,
        &automation.policy,
        &automation.preview,
    );
    let stale_query = AutomationQueryRequestV1 {
        request: Some(automation_query_request_v1::Request::ListTemplates(
            ListAutomationTemplatesQueryV1 {
                limit: 10,
                after_template_id: String::new(),
            },
        )),
    }
    .encode_to_vec();
    assert!(matches!(
        route_telegram_automation_client(
            &store,
            &fixture.supervisor.relay_port(),
            &stale_runtime,
            TelegramAutomationContractV1::Query,
            90,
            &stale_query,
        ),
        Err(TelegramClientRouteError::Kernel(error))
            if error == "managed runtime fence is stale"
    ));
}

struct TelegramAutomationConformanceState {
    template: AutomationTemplateV1,
    policy: AutomationPolicyV1,
    preview: AutomationPreviewReceiptV1,
    template_request: Vec<u8>,
    template_response: Vec<u8>,
}

fn assert_telegram_automation_management(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> TelegramAutomationConformanceState {
    let operations_before = telegram_operation_count(store, supervisor, telegram);
    let (template, template_request, template_response) =
        assert_automation_template(store, supervisor, telegram);
    let policy = assert_automation_policy(store, supervisor, telegram);
    let preview = assert_automation_preview(store, supervisor, telegram);
    assert_automation_query_projection(store, supervisor, telegram, &template, &policy, &preview);
    assert_stale_automation_template_revision_is_rejected(store, supervisor, telegram);
    assert_eq!(
        telegram_operation_count(store, supervisor, telegram),
        operations_before,
        "Telegram automation management and preview must not create provider operations"
    );
    TelegramAutomationConformanceState {
        template,
        policy,
        preview,
        template_request,
        template_response,
    }
}

fn assert_automation_template(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> (AutomationTemplateV1, Vec<u8>, Vec<u8>) {
    let request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::UpsertTemplate(
            UpsertAutomationTemplateCommandV1 {
                mutation_id: "managed-template-mutation-1".to_owned(),
                expected_revision: 0,
                template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                name: "Managed greeting".to_owned(),
                body_template: "Hello {{name}}".to_owned(),
                required_variables: vec!["name".to_owned()],
            },
        )),
    }
    .encode_to_vec();
    let first = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        81,
        &request,
    );
    let replay = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        81,
        &request,
    );
    assert_eq!(
        replay, first,
        "an exact Telegram automation mutation retry must replay exact response bytes"
    );
    let response = decode_automation_command_response(81, &first);
    let Some(automation_command_response_v1::Response::Template(template)) = response.response
    else {
        panic!("Telegram automation template upsert returned the wrong response type");
    };
    assert_eq!(template.template_id, AUTOMATION_TEMPLATE_ID);
    assert_eq!(template.revision, 1);
    assert_eq!(template.required_variables, ["name"]);

    let conflicting_request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::UpsertTemplate(
            UpsertAutomationTemplateCommandV1 {
                mutation_id: "managed-template-mutation-1".to_owned(),
                expected_revision: 0,
                template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                name: "Conflicting greeting".to_owned(),
                body_template: "Different {{name}}".to_owned(),
                required_variables: vec!["name".to_owned()],
            },
        )),
    }
    .encode_to_vec();
    let conflict = decode_automation_command_response(
        82,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Command,
            82,
            &conflicting_request,
        ),
    );
    assert_automation_failure(
        conflict.response,
        AutomationFailureCodeV1::AutomationFailureCodeIdempotencyConflict,
        "idempotency_key",
    );
    (template, request, first)
}

fn assert_automation_policy(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> AutomationPolicyV1 {
    let request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::UpsertPolicy(
            UpsertAutomationPolicyCommandV1 {
                mutation_id: "managed-policy-mutation-1".to_owned(),
                expected_revision: 0,
                policy_id: AUTOMATION_POLICY_ID.to_owned(),
                template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                name: "Managed preview policy".to_owned(),
                enabled: true,
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_ids: vec![AUTOMATION_CHAT_ID.to_owned()],
                expires_at_unix_seconds: None,
            },
        )),
    }
    .encode_to_vec();
    let response = decode_automation_command_response(
        83,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Command,
            83,
            &request,
        ),
    );
    let Some(automation_command_response_v1::Response::Policy(policy)) = response.response else {
        panic!("Telegram automation policy upsert returned the wrong response type");
    };
    assert_eq!(policy.policy_id, AUTOMATION_POLICY_ID);
    assert_eq!(policy.revision, 1);
    assert_eq!(policy.provider_chat_ids, [AUTOMATION_CHAT_ID]);
    policy
}

fn assert_automation_preview(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> AutomationPreviewReceiptV1 {
    let request = AutomationCommandRequestV1 {
        command: Some(automation_command_request_v1::Command::PreviewPolicy(
            PreviewAutomationPolicyCommandV1 {
                preview_id: AUTOMATION_PREVIEW_ID.to_owned(),
                policy_id: AUTOMATION_POLICY_ID.to_owned(),
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                provider_chat_id: AUTOMATION_CHAT_ID.to_owned(),
                variables: vec![AutomationVariableV1 {
                    name: "name".to_owned(),
                    value: "Ada".to_owned(),
                }],
            },
        )),
    }
    .encode_to_vec();
    let first = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        84,
        &request,
    );
    let replay = route_telegram_automation_until_ready(
        store,
        supervisor,
        telegram,
        TelegramAutomationContractV1::Command,
        84,
        &request,
    );
    assert_eq!(
        replay, first,
        "an exact Telegram automation preview retry must replay exact response bytes"
    );
    let response = decode_automation_command_response(84, &first);
    let Some(automation_command_response_v1::Response::Preview(preview)) = response.response else {
        panic!("Telegram automation preview returned the wrong response type");
    };
    assert_eq!(preview.preview_id, AUTOMATION_PREVIEW_ID);
    assert_eq!(preview.rendered_text, "Hello Ada");
    assert_eq!(
        preview.rendered_sha256,
        sha2::Sha256::digest(b"Hello Ada").as_slice()
    );
    preview
}

fn assert_automation_query_projection(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    template: &AutomationTemplateV1,
    policy: &AutomationPolicyV1,
    preview: &AutomationPreviewReceiptV1,
) {
    let templates = decode_automation_query_response(
        85,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Query,
            85,
            &AutomationQueryRequestV1 {
                request: Some(automation_query_request_v1::Request::ListTemplates(
                    ListAutomationTemplatesQueryV1 {
                        limit: 10,
                        after_template_id: String::new(),
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    let Some(automation_query_response_v1::Response::Templates(templates)) = templates.response
    else {
        panic!("Telegram automation template query returned the wrong response type");
    };
    assert_eq!(templates.items.as_slice(), std::slice::from_ref(template));

    let policies = decode_automation_query_response(
        86,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Query,
            86,
            &AutomationQueryRequestV1 {
                request: Some(automation_query_request_v1::Request::ListPolicies(
                    ListAutomationPoliciesQueryV1 {
                        limit: 10,
                        after_policy_id: String::new(),
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    let Some(automation_query_response_v1::Response::Policies(policies)) = policies.response else {
        panic!("Telegram automation policy query returned the wrong response type");
    };
    assert_eq!(policies.items.as_slice(), std::slice::from_ref(policy));

    let receipt = decode_automation_query_response(
        87,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Query,
            87,
            &AutomationQueryRequestV1 {
                request: Some(automation_query_request_v1::Request::GetPreviewReceipt(
                    GetAutomationPreviewReceiptQueryV1 {
                        preview_id: AUTOMATION_PREVIEW_ID.to_owned(),
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    let Some(automation_query_response_v1::Response::PreviewReceipt(receipt)) = receipt.response
    else {
        panic!("Telegram automation preview receipt query returned the wrong response type");
    };
    assert_eq!(receipt, *preview);
}

fn assert_stale_automation_template_revision_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) {
    let response = decode_automation_command_response(
        88,
        &route_telegram_automation_until_ready(
            store,
            supervisor,
            telegram,
            TelegramAutomationContractV1::Command,
            88,
            &AutomationCommandRequestV1 {
                command: Some(automation_command_request_v1::Command::UpsertTemplate(
                    UpsertAutomationTemplateCommandV1 {
                        mutation_id: "managed-template-stale-1".to_owned(),
                        expected_revision: 0,
                        template_id: AUTOMATION_TEMPLATE_ID.to_owned(),
                        name: "Stale update".to_owned(),
                        body_template: "Stale {{name}}".to_owned(),
                        required_variables: vec!["name".to_owned()],
                    },
                )),
            }
            .encode_to_vec(),
        ),
    );
    assert_automation_failure(
        response.response,
        AutomationFailureCodeV1::AutomationFailureCodeRevisionConflict,
        "expected_revision",
    );
}

fn assert_automation_failure(
    response: Option<automation_command_response_v1::Response>,
    expected_code: AutomationFailureCodeV1,
    expected_field: &str,
) {
    let Some(automation_command_response_v1::Response::Failure(failure)) = response else {
        panic!("Telegram automation command did not return a typed failure");
    };
    assert_eq!(failure.code, expected_code as i32);
    assert_eq!(failure.field, expected_field);
}

fn telegram_operation_count(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
) -> usize {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_client(
            store,
            &relay,
            telegram,
            TelegramClientContractV1::Query,
            89,
            &TelegramClientRequest::Query(TelegramProviderQuery::Operations {
                account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                limit: 100,
            }),
        ) {
            Ok(TelegramClientResponse::Query(TelegramProviderQueryResponse::Operations(
                operations,
            ))) => return operations.len(),
            Ok(_) => panic!("Telegram operation query returned the wrong response type"),
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram operation query remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram operation query failed: {error:?}"),
        }
    }
}

fn route_telegram_automation_until_ready(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    contract: TelegramAutomationContractV1,
    request_id: u64,
    request_payload: &[u8],
) -> Vec<u8> {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match route_telegram_automation_client(
            store,
            &relay,
            telegram,
            contract,
            request_id,
            request_payload,
        ) {
            Ok(response) => return response,
            Err(error) if error.is_retryable() => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Telegram automation route remained busy"
                );
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("Telegram automation route failed: {error:?}"),
        }
    }
}

fn route_telegram_automation_client(
    store: &SqliteControlStore,
    relay: &crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort,
    telegram: &StartedTelegramRuntime,
    contract: TelegramAutomationContractV1,
    request_id: u64,
    request_payload: &[u8],
) -> Result<Vec<u8>, TelegramClientRouteError> {
    let request = ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: TELEGRAM_AUTOMATION_MODULE_ID.to_owned(),
        owner_id: TELEGRAM_AUTOMATION_OWNER_ID.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: TELEGRAM_AUTOMATION_OWNER_ID.to_owned(),
            name: contract.contract_name().to_owned(),
            major: TELEGRAM_AUTOMATION_CONTRACT_MAJOR,
            revision: TELEGRAM_AUTOMATION_CONTRACT_REVISION,
            schema_sha256: sha2::Sha256::digest(TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1).to_vec(),
        }),
        request_id,
        request_payload: request_payload.to_vec(),
    }
    .encode_to_vec();
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
    let response = ModuleClientResponseV1::decode(bytes.as_slice()).map_err(|error| {
        TelegramClientRouteError::Client(TelegramClientPortError::Codec(error.to_string()))
    })?;
    if !response.error_code.is_empty() {
        return Err(TelegramClientRouteError::Client(
            TelegramClientPortError::Protocol(response.error_code),
        ));
    }
    Ok(bytes)
}

fn decode_automation_command_response(
    request_id: u64,
    bytes: &[u8],
) -> AutomationCommandResponseV1 {
    let payload = decode_automation_response_payload(request_id, bytes);
    AutomationCommandResponseV1::decode(payload.as_slice())
        .expect("decode Telegram automation command response")
}

fn decode_automation_query_response(request_id: u64, bytes: &[u8]) -> AutomationQueryResponseV1 {
    let payload = decode_automation_response_payload(request_id, bytes);
    AutomationQueryResponseV1::decode(payload.as_slice())
        .expect("decode Telegram automation query response")
}

fn decode_automation_response_payload(request_id: u64, bytes: &[u8]) -> Vec<u8> {
    let response =
        ModuleClientResponseV1::decode(bytes).expect("decode Telegram automation module response");
    assert_eq!(response.protocol_major, MODULE_CLIENT_PROTOCOL_MAJOR);
    assert_eq!(response.request_id, request_id);
    assert!(response.error_code.is_empty());
    assert!(!response.response_payload.is_empty());
    response.response_payload
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

fn assert_telegram_command_accepted(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    operation_id: &str,
    text: &str,
) {
    let relay = supervisor.relay_port();
    let command =
        TelegramClientRequest::Command(TelegramProviderCommand::SendText(TelegramSendMessage {
            operation_id: operation_id.to_owned(),
            account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
            provider_chat_id: "9001".to_owned(),
            text: text.to_owned(),
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
    assert_eq!(operation.operation_id, operation_id);
    assert_eq!(
        operation.state,
        TelegramOperationState::Accepted,
        "accepted receipt is distinct from provider completion"
    );
}

fn assert_telegram_operation_completed(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    telegram: &StartedTelegramRuntime,
    operation_id: &str,
) {
    let relay = supervisor.relay_port();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
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
            .find(|operation| operation.operation_id == operation_id)
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
