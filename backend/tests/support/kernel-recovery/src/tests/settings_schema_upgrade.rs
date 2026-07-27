use hermes_kernel_control_store::{
    ModuleRegistration, ModuleRegistrationState, SettingsApplyState, SettingsInitialSnapshot,
    SettingsSchemaBinding, SettingsSchemaBindingInputV1,
};
use hermes_kernel_control_store_sqlite::SqliteControlStore;

#[test]
fn settings_schema_upgrade_commits_binding_artifact_and_successor_snapshot_atomically() {
    let path = fixture_path();
    let store =
        SqliteControlStore::create(&path, "instance-settings-upgrade", 1).expect("create store");
    let registration = ModuleRegistration::new(
        "registration-zulip",
        "integration-zulip",
        "owner-local",
        [7; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&registration, &["zulip.settings".to_owned()])
        .expect("create registration");
    store
        .approve_module_registration(
            registration.registration_id(),
            &["zulip.settings".to_owned()],
        )
        .expect("approve registration");
    let existing = binding(2, [2; 32], 0, 0);
    store
        .admit_settings_schema(&existing, b"schema-v2")
        .expect("admit existing schema");
    store
        .materialize_initial_settings_snapshot(&SettingsInitialSnapshot {
            registration_id: registration.registration_id().to_owned(),
            snapshot_bytes: b"snapshot-v1".to_vec(),
            complete: false,
        })
        .expect("materialize existing snapshot");
    let expected = store
        .settings_schema_binding(registration.registration_id())
        .expect("read existing binding")
        .expect("existing binding");
    let successor = binding(3, [3; 32], 2, 0);

    store
        .upgrade_settings_schema_with_successor(&expected, &successor, b"schema-v3", b"snapshot-v2")
        .expect("upgrade settings schema");

    assert_eq!(
        store
            .settings_schema_binding(registration.registration_id())
            .expect("read successor binding"),
        Some(successor),
    );
    assert_eq!(
        store
            .settings_schema_artifact(registration.registration_id())
            .expect("read successor schema"),
        Some(b"schema-v3".to_vec()),
    );
    assert_eq!(
        store
            .desired_settings_snapshot(registration.registration_id())
            .expect("read successor snapshot"),
        Some((2, b"snapshot-v2".to_vec())),
    );

    drop(store);
    std::fs::remove_file(path).expect("remove control store");
}

fn binding(
    schema_major: u32,
    schema_sha256: [u8; 32],
    desired_revision: u64,
    effective_revision: u64,
) -> SettingsSchemaBinding {
    SettingsSchemaBinding::new(SettingsSchemaBindingInputV1 {
        registration_id: "registration-zulip".to_owned(),
        schema_major,
        schema_revision: 1,
        schema_sha256,
        desired_revision,
        effective_revision,
        apply_state: if desired_revision == 0 {
            SettingsApplyState::Current
        } else {
            SettingsApplyState::BlockedConfig
        },
        sanitized_reason_code: (desired_revision > 0)
            .then(|| "required_settings_missing".to_owned()),
    })
}

fn fixture_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "hermes-settings-schema-upgrade-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
