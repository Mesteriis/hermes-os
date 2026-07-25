//! Exact admission, storage, Vault and release assembly for managed Mail conformance.

use super::*;

use hermes_mail_api::{
    MailCredentialPurpose,
    client_contract::{MAIL_MODULE_ID, MAIL_OWNER_ID, MailClientContractV1},
};
use hermes_mail_persistence::{MAIL_STORAGE_BUNDLE_REVISION_V2, mail_storage_bundle_v1};
use hermes_mail_runtime::{
    admission::{
        MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID, MAIL_BLOB_CAPABILITY_ID,
        MAIL_CREDENTIAL_LEASE_TTL_SECONDS, MAIL_EVENTS_CAPABILITY_ID,
        MAIL_IMAP_CREDENTIALS_CAPABILITY_ID, MAIL_STORAGE_CAPABILITY_ID, mail_module_descriptor_v1,
    },
    settings::mail_settings_schema_bytes_v1,
};
use hermes_vault_key_provider::WrappingKeyProvider;
use hermes_vault_key_provider_file::FileWrappingKeyProvider;
use hermes_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use hermes_vault_store_sqlcipher::{SecretRecordScope, VaultStore};

const MAIL_RELEASE_ARTIFACT_ID: &str = "integration.mail";
pub(super) const MAIL_ACCOUNT_ID: &str = "mail-account-1";

pub(super) struct AdmittedMailRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedMailRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
}

pub(super) fn installed_communications_mail_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(
        SignedRuntimeArtifact::new(
            MAIL_RELEASE_ARTIFACT_ID,
            mail_binary(),
            mail_module_descriptor_v1("managed-mail-live").encode_to_vec(),
        )
        .with_settings_schema(mail_settings_schema_bytes_v1()),
    );
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications and Mail release")
}

pub(super) fn seed_mail_vault(vault_dir: &Path) {
    let key = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("open Vault wrapping key");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("open initialized Vault");
    let purpose = MailCredentialPurpose::ImapPassword;
    let request = VaultPurposeRequestV1::new(
        purpose.as_str().to_owned(),
        MAIL_ACCOUNT_ID.to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Resolve],
        MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
    )
    .expect("Mail IMAP credential purpose");
    let scope = SecretRecordScope::new(
        MAIL_OWNER_ID.to_owned(),
        &request,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("Mail IMAP secret scope");
    store
        .store_secret(&scope, b"managed-mail-imap-password")
        .expect("store Mail IMAP test credential");
}

pub(super) fn admit_mail_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    let descriptor = mail_module_descriptor_v1("managed-mail-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Mail descriptor");
    let capability_ids = vec![
        MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID.to_owned(),
        MAIL_BLOB_CAPABILITY_ID.to_owned(),
        MAIL_EVENTS_CAPABILITY_ID.to_owned(),
        MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
        MAIL_STORAGE_CAPABILITY_ID.to_owned(),
        MailClientContractV1::Sync.capability_id().to_owned(),
    ];
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Mail IMAP sync capabilities");
    let schema = mail_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "hermes-managed-runtime-conformance",
            MAIL_RELEASE_ARTIFACT_ID,
            Sha256::digest(std::fs::read(mail_binary()).expect("Mail runtime binary bytes")).into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Mail release binding");
    let bundle = mail_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                MAIL_OWNER_ID,
                u64::from(MAIL_STORAGE_BUNDLE_REVISION_V2),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record Mail Storage bundle"),
        )
        .expect("persist Mail Storage bundle");
    AdmittedMailRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_mail_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedMailRuntime,
) -> AdmittedMailRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Mail managed launch");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let bundle = store
        .platform_storage_bundle(MAIL_OWNER_ID, u64::from(MAIL_STORAGE_BUNDLE_REVISION_V2))
        .expect("read Mail Storage bundle")
        .expect("Mail Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        &runtime_instance_id,
        runtime_generation,
        MAIL_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(MAIL_STORAGE_BUNDLE_REVISION_V2),
            *bundle.digest(),
        )
        .expect("Mail Storage binding issue"),
    )
    .expect("issue Mail Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Storage binding");
    admitted
}

pub(super) fn start_mail_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    imap_port: u16,
) -> StartedMailRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Mail managed launch reservation");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(&admitted.registration_id, MAIL_STORAGE_CAPABILITY_ID)
        .expect("read Mail Storage binding")
        .expect("Mail Storage binding");
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
    .expect("build Mail Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let configuration = hermes_runtime_protocol::v1::ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: MAIL_OWNER_ID.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision: events.credential_revision(),
        configuration_instance_id: MAIL_ACCOUNT_ID.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
    };
    managed_launch::start_reserved_integration(
        supervisor,
        kernel_data,
        runtime_dir,
        reservation,
        managed_launch::ManagedIntegrationLaunchConfiguration {
            runtime: configuration,
            settings_snapshot_bytes: mail_settings_snapshot(imap_port).encode_to_vec(),
            granted_capability_ids: &admitted.capability_ids,
        },
    )
    .expect("start managed Mail integration");
    StartedMailRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
    }
}

fn mail_settings_snapshot(imap_port: u16) -> hermes_runtime_protocol::v1::SettingsSnapshotV1 {
    use hermes_runtime_protocol::v1::{
        SettingValueV1, SettingsValueEntryV1, setting_value_v1::Value,
    };

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }

    hermes_runtime_protocol::v1::SettingsSnapshotV1 {
        target_id: MAIL_ACCOUNT_ID.to_owned(),
        revision: 1,
        values: vec![
            entry(
                "mail.connection_id",
                Value::StringValue(MAIL_ACCOUNT_ID.to_owned()),
            ),
            entry("mail.imap.host", Value::StringValue("localhost".to_owned())),
            entry(
                "mail.imap.password_revision",
                Value::UnsignedIntegerValue(1),
            ),
            entry(
                "mail.imap.port",
                Value::UnsignedIntegerValue(u64::from(imap_port)),
            ),
            entry(
                "mail.imap.username",
                Value::StringValue("owner@example.test".to_owned()),
            ),
            entry("mail.inbound.kind", Value::StringValue("imap".to_owned())),
            entry("mail.smtp.enabled", Value::BooleanValue(false)),
            entry("mail.sync.window", Value::UnsignedIntegerValue(1)),
            entry("mail.sync.windows", Value::UnsignedIntegerValue(1)),
        ],
    }
}

fn mail_binary() -> PathBuf {
    binary("HERMES_MAIL_RUNTIME_BIN")
}
