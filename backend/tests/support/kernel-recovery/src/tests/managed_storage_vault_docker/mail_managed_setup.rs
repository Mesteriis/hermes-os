//! Exact admission, storage, Vault and release assembly for managed Mail conformance.

use super::*;

use hermes_mail_api::{
    MailCredentialPurpose,
    client_contract::{MAIL_MODULE_ID, MAIL_OWNER_ID, MailClientContractV1},
};
use hermes_mail_persistence::{MAIL_STORAGE_BUNDLE_REVISION_V3, mail_storage_bundle_v1};
use hermes_mail_runtime::{
    admission::{
        MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID,
        MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID,
        MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID, MAIL_BLOB_CAPABILITY_ID,
        MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID, MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
        MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID, MAIL_IMAP_CREDENTIALS_CAPABILITY_ID,
        MAIL_SMTP_CREDENTIALS_CAPABILITY_ID, MAIL_STORAGE_CAPABILITY_ID, mail_module_descriptor_v1,
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

pub(super) struct MailSmtpFixtureSettingsV1 {
    pub(super) port: u16,
    pub(super) ca_certificate_pem: String,
}

pub(super) struct MailGmailFixtureSettingsV1 {
    pub(super) port: u16,
    pub(super) ca_certificate_pem: String,
}

#[derive(Clone, Copy)]
enum MailAdmissionProfileV1 {
    ImapSync,
    SmtpDelivery,
    GmailDelivery,
}

enum MailSettingsProfileV1 {
    Imap {
        port: u16,
        smtp: Option<MailSmtpFixtureSettingsV1>,
    },
    Gmail(MailGmailFixtureSettingsV1),
}

pub(super) fn installed_communications_mail_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(mail_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications and Mail release")
}

pub(super) fn mail_release_artifact() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        MAIL_RELEASE_ARTIFACT_ID,
        mail_binary(),
        mail_module_descriptor_v1("managed-mail-live").encode_to_vec(),
    )
    .with_settings_schema(mail_settings_schema_bytes_v1())
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
    for (purpose, secret) in [
        (
            MailCredentialPurpose::ImapPassword,
            b"managed-mail-imap-password".as_slice(),
        ),
        (
            MailCredentialPurpose::SmtpPassword,
            b"managed-mail-smtp-password".as_slice(),
        ),
        (
            MailCredentialPurpose::GmailAccessToken,
            b"managed-mail-gmail-access-token".as_slice(),
        ),
    ] {
        let request = VaultPurposeRequestV1::new(
            purpose.as_str().to_owned(),
            MAIL_ACCOUNT_ID.to_owned(),
            vec![SecretClassV1::ProviderCredential],
            vec![VaultActionV1::Resolve],
            MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
        )
        .expect("Mail credential purpose");
        let scope = SecretRecordScope::new(
            MAIL_OWNER_ID.to_owned(),
            &request,
            SecretClassV1::ProviderCredential,
            1,
        )
        .expect("Mail secret scope");
        store
            .store_secret(&scope, secret)
            .expect("store Mail test credential");
    }
}

pub(super) fn admit_mail_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::ImapSync)
}

pub(super) fn admit_mail_delivery_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::SmtpDelivery)
}

pub(super) fn admit_mail_gmail_delivery_runtime(store: &SqliteControlStore) -> AdmittedMailRuntime {
    admit_mail_runtime_profile(store, MailAdmissionProfileV1::GmailDelivery)
}

fn admit_mail_runtime_profile(
    store: &SqliteControlStore,
    profile: MailAdmissionProfileV1,
) -> AdmittedMailRuntime {
    let descriptor = mail_module_descriptor_v1("managed-mail-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Mail descriptor");
    let mut capability_ids = match profile {
        MailAdmissionProfileV1::ImapSync => vec![
            MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_BLOB_CAPABILITY_ID.to_owned(),
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::Sync.capability_id().to_owned(),
        ],
        MailAdmissionProfileV1::SmtpDelivery => vec![
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_IMAP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_SMTP_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::Delivery.capability_id().to_owned(),
            MailClientContractV1::DeliveryQuery
                .capability_id()
                .to_owned(),
        ],
        MailAdmissionProfileV1::GmailDelivery => vec![
            MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID.to_owned(),
            MAIL_GMAIL_CREDENTIALS_CAPABILITY_ID.to_owned(),
            MAIL_STORAGE_CAPABILITY_ID.to_owned(),
            MailClientContractV1::Delivery.capability_id().to_owned(),
            MailClientContractV1::DeliveryQuery
                .capability_id()
                .to_owned(),
        ],
    };
    capability_ids.sort();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Mail profile capabilities");
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
                u64::from(MAIL_STORAGE_BUNDLE_REVISION_V3),
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
        .platform_storage_bundle(MAIL_OWNER_ID, u64::from(MAIL_STORAGE_BUNDLE_REVISION_V3))
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
            u64::from(MAIL_STORAGE_BUNDLE_REVISION_V3),
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
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::Imap {
            port: imap_port,
            smtp: None,
        },
    )
}

pub(super) fn start_mail_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    imap_port: u16,
    smtp: MailSmtpFixtureSettingsV1,
) -> StartedMailRuntime {
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::Imap {
            port: imap_port,
            smtp: Some(smtp),
        },
    )
}

pub(super) fn start_mail_gmail_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    gmail: MailGmailFixtureSettingsV1,
) -> StartedMailRuntime {
    start_mail_runtime_with_settings(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        admitted,
        MailSettingsProfileV1::Gmail(gmail),
    )
}

#[allow(clippy::too_many_arguments)]
fn start_mail_runtime_with_settings(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedMailRuntime,
    settings: MailSettingsProfileV1,
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
            settings_snapshot_bytes: mail_settings_snapshot(settings).encode_to_vec(),
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

fn mail_settings_snapshot(
    profile: MailSettingsProfileV1,
) -> hermes_runtime_protocol::v1::SettingsSnapshotV1 {
    use hermes_runtime_protocol::v1::{
        SettingValueV1, SettingsValueEntryV1, setting_value_v1::Value,
    };

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }

    let mut values = vec![
        entry(
            "mail.connection_id",
            Value::StringValue(MAIL_ACCOUNT_ID.to_owned()),
        ),
        entry("mail.sync.window", Value::UnsignedIntegerValue(1)),
        entry("mail.sync.windows", Value::UnsignedIntegerValue(1)),
    ];
    match profile {
        MailSettingsProfileV1::Imap { port, smtp } => {
            values.extend([
                entry("mail.imap.host", Value::StringValue("localhost".to_owned())),
                entry(
                    "mail.imap.password_revision",
                    Value::UnsignedIntegerValue(1),
                ),
                entry(
                    "mail.imap.port",
                    Value::UnsignedIntegerValue(u64::from(port)),
                ),
                entry(
                    "mail.imap.username",
                    Value::StringValue("owner@example.test".to_owned()),
                ),
                entry("mail.inbound.kind", Value::StringValue("imap".to_owned())),
                entry("mail.smtp.enabled", Value::BooleanValue(smtp.is_some())),
            ]);
            if let Some(smtp) = smtp {
                values.extend([
                    entry(
                        "mail.smtp.ca_certificate_pem",
                        Value::StringValue(smtp.ca_certificate_pem),
                    ),
                    entry(
                        "mail.smtp.from_address",
                        Value::StringValue("owner@example.test".to_owned()),
                    ),
                    entry("mail.smtp.host", Value::StringValue("localhost".to_owned())),
                    entry(
                        "mail.smtp.password_revision",
                        Value::UnsignedIntegerValue(1),
                    ),
                    entry(
                        "mail.smtp.port",
                        Value::UnsignedIntegerValue(u64::from(smtp.port)),
                    ),
                    entry(
                        "mail.smtp.username",
                        Value::StringValue("owner@example.test".to_owned()),
                    ),
                ]);
            }
        }
        MailSettingsProfileV1::Gmail(gmail) => values.extend([
            entry(
                "mail.gmail.access_token_revision",
                Value::UnsignedIntegerValue(1),
            ),
            entry(
                "mail.gmail.api_host",
                Value::StringValue("localhost".to_owned()),
            ),
            entry(
                "mail.gmail.api_port",
                Value::UnsignedIntegerValue(u64::from(gmail.port)),
            ),
            entry(
                "mail.gmail.ca_certificate_pem",
                Value::StringValue(gmail.ca_certificate_pem),
            ),
            entry(
                "mail.gmail.from_address",
                Value::StringValue("owner@example.test".to_owned()),
            ),
            entry("mail.gmail.user_id", Value::StringValue("me".to_owned())),
            entry("mail.inbound.kind", Value::StringValue("gmail".to_owned())),
            entry("mail.smtp.enabled", Value::BooleanValue(false)),
        ]),
    }
    values.sort_by(|left, right| left.setting_id.cmp(&right.setting_id));
    hermes_runtime_protocol::v1::SettingsSnapshotV1 {
        target_id: MAIL_ACCOUNT_ID.to_owned(),
        revision: 1,
        values,
    }
}

fn mail_binary() -> PathBuf {
    binary("HERMES_MAIL_RUNTIME_BIN")
}
