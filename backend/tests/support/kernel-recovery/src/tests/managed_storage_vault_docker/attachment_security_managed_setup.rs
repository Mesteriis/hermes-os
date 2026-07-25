//! Exact signed admission, Storage and settings setup for Attachment Security.

use super::*;

use hermes_attachment_security_persistence::{
    ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V1, attachment_security_storage_bundle_v1,
};
use hermes_attachment_security_runtime::{
    admission::{
        ATTACHMENT_SECURITY_BLOB_READ_CAPABILITY_ID,
        ATTACHMENT_SECURITY_CANDIDATE_OBSERVE_CAPABILITY_ID,
        ATTACHMENT_SECURITY_COMMUNICATIONS_STATE_OBSERVE_CAPABILITY_ID,
        ATTACHMENT_SECURITY_OWNER_ID, ATTACHMENT_SECURITY_STORAGE_CAPABILITY_ID,
        ATTACHMENT_SECURITY_VERDICT_PUBLISH_CAPABILITY_ID,
        attachment_security_module_descriptor_v1,
    },
    settings::attachment_security_settings_schema_bytes_v1,
};
use hermes_runtime_protocol::v1::{
    ManagedEngineRuntimeConfigurationV1, SettingValueV1, SettingsSnapshotV1, SettingsValueEntryV1,
    setting_value_v1::Value,
};

const ATTACHMENT_SECURITY_RELEASE_ARTIFACT_ID: &str = "engine.attachment-security";
const ATTACHMENT_SECURITY_SETTINGS_REVISION: u64 = 1;

pub(super) struct AdmittedAttachmentSecurityRuntime {
    registration_id: String,
}

pub(super) struct StartedAttachmentSecurityRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
}

pub(super) fn installed_communications_mail_attachment_security_release(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(mail_release_artifact());
    artifacts.push(attachment_security_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications, Mail and Attachment Security release")
}

pub(super) fn admit_attachment_security_runtime(
    store: &SqliteControlStore,
) -> AdmittedAttachmentSecurityRuntime {
    let descriptor = attachment_security_module_descriptor_v1("managed-attachment-security-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Attachment Security descriptor");
    let capability_ids = vec![
        ATTACHMENT_SECURITY_BLOB_READ_CAPABILITY_ID.to_owned(),
        ATTACHMENT_SECURITY_CANDIDATE_OBSERVE_CAPABILITY_ID.to_owned(),
        ATTACHMENT_SECURITY_COMMUNICATIONS_STATE_OBSERVE_CAPABILITY_ID.to_owned(),
        ATTACHMENT_SECURITY_STORAGE_CAPABILITY_ID.to_owned(),
        ATTACHMENT_SECURITY_VERDICT_PUBLISH_CAPABILITY_ID.to_owned(),
    ];
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Attachment Security capabilities");
    let schema = attachment_security_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "hermes-managed-runtime-conformance",
            ATTACHMENT_SECURITY_RELEASE_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(attachment_security_binary())
                    .expect("Attachment Security runtime binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Attachment Security release binding");
    let bundle = attachment_security_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ATTACHMENT_SECURITY_OWNER_ID,
                u64::from(ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record Attachment Security Storage bundle"),
        )
        .expect("persist Attachment Security Storage bundle");
    AdmittedAttachmentSecurityRuntime {
        registration_id: registration.registration_id().to_owned(),
    }
}

pub(super) fn prepare_attachment_security_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedAttachmentSecurityRuntime,
) -> AdmittedAttachmentSecurityRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Attachment Security managed launch");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let bundle = store
        .platform_storage_bundle(
            ATTACHMENT_SECURITY_OWNER_ID,
            u64::from(ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Attachment Security Storage bundle")
        .expect("Attachment Security Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        &runtime_instance_id,
        runtime_generation,
        ATTACHMENT_SECURITY_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Attachment Security Storage binding issue"),
    )
    .expect("issue Attachment Security Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Attachment Security Storage binding");
    admitted
}

pub(super) fn start_attachment_security_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedAttachmentSecurityRuntime,
    clamav_port: u16,
) -> StartedAttachmentSecurityRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Attachment Security managed launch reservation");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(
            &admitted.registration_id,
            ATTACHMENT_SECURITY_STORAGE_CAPABILITY_ID,
        )
        .expect("read Attachment Security Storage binding")
        .expect("Attachment Security Storage binding");
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Attachment Security Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let configuration = ManagedEngineRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: ATTACHMENT_SECURITY_OWNER_ID.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision: events.credential_revision(),
        settings_revision: ATTACHMENT_SECURITY_SETTINGS_REVISION,
    };
    managed_launch::start_reserved_engine(
        supervisor,
        runtime_dir,
        reservation,
        configuration,
        attachment_security_settings_snapshot(&admitted.registration_id, clamav_port)
            .encode_to_vec(),
    )
    .expect("start managed Attachment Security engine");
    StartedAttachmentSecurityRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
    }
}

fn attachment_security_release_artifact() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        ATTACHMENT_SECURITY_RELEASE_ARTIFACT_ID,
        attachment_security_binary(),
        attachment_security_module_descriptor_v1("managed-attachment-security-live")
            .encode_to_vec(),
    )
    .with_settings_schema(attachment_security_settings_schema_bytes_v1())
}

fn attachment_security_settings_snapshot(
    registration_id: &str,
    clamav_port: u16,
) -> SettingsSnapshotV1 {
    fn entry(setting_id: &str, value: u64) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 {
                value: Some(Value::UnsignedIntegerValue(value)),
            }),
        }
    }

    SettingsSnapshotV1 {
        target_id: registration_id.to_owned(),
        revision: ATTACHMENT_SECURITY_SETTINGS_REVISION,
        values: vec![
            entry("attachment_security.clamav.connect_timeout_millis", 500),
            entry("attachment_security.clamav.io_timeout_millis", 1_000),
            entry("attachment_security.max_scan_bytes", 16 * 1024 * 1024),
            entry("attachment_security.clamav.port", u64::from(clamav_port)),
        ],
    }
}

fn attachment_security_binary() -> PathBuf {
    binary("HERMES_ATTACHMENT_SECURITY_RUNTIME_BIN")
}
