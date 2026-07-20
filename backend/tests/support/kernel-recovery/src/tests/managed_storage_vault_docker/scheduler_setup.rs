use super::*;

pub(super) fn configured_scheduler_store(root: &Path, kernel: &Path) -> SqliteControlStore {
    let store = configured_store(root, kernel);
    let schema = scheduler_schema();
    let descriptor = scheduler_descriptor(&schema);
    let grant_epoch = record_scheduler_registration(&store, &descriptor);
    record_scheduler_runtime_fixture(&store, &schema, &descriptor, grant_epoch);
    store
}

fn record_scheduler_registration(store: &SqliteControlStore, descriptor: &[u8]) -> u64 {
    let registration = ModuleRegistration::new(
        SCHEDULER_REGISTRATION,
        "scheduler",
        "scheduler",
        Sha256::digest(descriptor).into(),
        ModuleRegistrationState::Pending,
        1,
    );
    let capabilities = [
        ACK_CAPABILITY.to_owned(),
        DISPATCH_CAPABILITY.to_owned(),
        RESULT_CAPABILITY.to_owned(),
        STORAGE_CAPABILITY.to_owned(),
    ];
    let storage = ModuleStorageRequestV1::new(
        SCHEDULER_REGISTRATION,
        STORAGE_CAPABILITY,
        "scheduler",
        4,
        5_000,
    );
    let routes = [
        event_route(
            DISPATCH_CAPABILITY,
            ModuleEventEnvelopeKindV1::Command,
            "platform",
            "maintenance",
            ModuleEventRouteDirectionV1::Publish,
        ),
        event_route(
            ACK_CAPABILITY,
            ModuleEventEnvelopeKindV1::Ack,
            "scheduler",
            "job_receipt",
            ModuleEventRouteDirectionV1::Consume,
        ),
        event_route(
            RESULT_CAPABILITY,
            ModuleEventEnvelopeKindV1::Result,
            "scheduler",
            "job_receipt",
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
        .expect("record Scheduler registration");
    store
        .approve_module_registration(SCHEDULER_REGISTRATION, &capabilities)
        .expect("approve Scheduler capabilities")
        .grant_epoch()
}

fn record_scheduler_runtime_fixture(
    store: &SqliteControlStore,
    schema: &[u8],
    descriptor: &[u8],
    grant_epoch: u64,
) {
    let canonical_bundle =
        hermes_scheduler_persistence::scheduler_storage_bundle_v1().encode_to_vec();
    let digest: [u8; 32] = Sha256::digest(&canonical_bundle).into();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new("scheduler", 7, digest, canonical_bundle)
                .expect("record Scheduler Storage bundle"),
        )
        .expect("persist Scheduler Storage bundle");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            SCHEDULER_REGISTRATION,
            1,
            "hermes-managed-runtime-conformance",
            "platform.scheduler",
            Sha256::digest(std::fs::read(scheduler_binary()).expect("Scheduler binary bytes"))
                .into(),
            Sha256::digest(descriptor).into(),
            Some(Sha256::digest(schema).into()),
        ))
        .expect("record Scheduler release binding");
    store
        .record_managed_launch(&ManagedLaunchRecord::new(
            SCHEDULER_REGISTRATION,
            "01010101010101010101010101010101",
            1,
            1,
            1,
            grant_epoch,
        ))
        .expect("record Scheduler reservation");
    store
        .record_platform_event_hub_topology(&event_hub_topology())
        .expect("record Event Hub topology");
}

pub(super) fn issue_initial_scheduler_storage_binding(store: &SqliteControlStore) {
    let bundle = store
        .platform_storage_bundle("scheduler", 7)
        .expect("read Scheduler Storage bundle")
        .expect("Scheduler Storage bundle is present");
    let binding = issue_managed(
        store,
        SCHEDULER_REGISTRATION,
        "01010101010101010101010101010101",
        1,
        STORAGE_CAPABILITY,
        StorageBindingIssueV1::new(1, 1, 7, *bundle.digest())
            .expect("initial Scheduler Storage issue"),
    )
    .expect("issue Scheduler Storage binding");
    assert_eq!(binding.runtime_generation(), 1);
}

fn event_route(
    capability: &str,
    kind: ModuleEventEnvelopeKindV1,
    owner: &str,
    contract: &str,
    direction: ModuleEventRouteDirectionV1,
) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(
        hermes_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: SCHEDULER_REGISTRATION.to_owned(),
            capability_id: capability.to_owned(),
            envelope_kind: kind,
            contract_owner: owner.to_owned(),
            contract_name: contract.to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [7; 32],
            direction,
            max_in_flight: 16,
            delivery_policy: matches!(direction, ModuleEventRouteDirectionV1::Consume).then(|| {
                ModuleEventDeliveryPolicyV1::new(
                    ModuleEventSubscriptionRequirementV1::Required,
                    3,
                    2_000,
                )
            }),
        },
    )
}

fn event_hub_topology() -> PlatformEventHubTopologyV1 {
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
        required("HERMES_SCHEDULER_LIVE_NATS_ENDPOINT"),
        "scheduler",
        1,
        budgets,
    )
}

fn scheduler_schema() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        ..Default::default()
    }
    .encode_to_vec()
}
fn scheduler_descriptor(schema: &[u8]) -> Vec<u8> {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "scheduler".to_owned(),
        owner_id: "scheduler".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "managed-scheduler-lifecycle".to_owned(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: schema.len() as u64,
            sha256: Sha256::digest(schema).to_vec(),
        }),
        ..Default::default()
    }
    .encode_to_vec()
}
pub(super) fn installed_scheduler_release(root: &Path) -> InstalledSignedBundle {
    let schema = scheduler_schema();
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
                "platform.scheduler",
                scheduler_binary(),
                scheduler_descriptor(&schema),
            )
            .with_settings_schema(schema),
        ],
    )
    .expect("install signed Scheduler release")
}
fn scheduler_binary() -> PathBuf {
    binary("HERMES_SCHEDULER_RUNTIME_BIN")
}
