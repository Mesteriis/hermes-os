import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../../', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const files = {
  adr: new URL(
    'docs/adr/ADR-0379-mail-address-book-sync-and-contacts-command-boundary.md',
    PROJECT_ROOT,
  ),
  sourceAdr: new URL(
    'docs/adr/ADR-0381-contacts-target-bound-mail-sync-source-port.md',
    PROJECT_ROOT,
  ),
  providerAdr: new URL(
    'docs/adr/ADR-0382-mail-address-book-provider-execution-and-authority.md',
    PROJECT_ROOT,
  ),
  inventory: new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  apiManifest: new URL('src/contacts-command-api/Cargo.toml', BACKEND_ROOT),
  coreManifest: new URL('src/contacts-core/Cargo.toml', BACKEND_ROOT),
  proto: new URL(
    'src/contacts-command-api/proto/hermes/contacts/command/v1/contacts_command.proto',
    BACKEND_ROOT,
  ),
  api: new URL('src/contacts-command-api/src/lib.rs', BACKEND_ROOT),
  envelope: new URL('src/contacts-command-api/src/envelope.rs', BACKEND_ROOT),
  sourceApiManifest: new URL('src/contacts-mail-sync-source-api/Cargo.toml', BACKEND_ROOT),
  sourceProto: new URL(
    'src/contacts-mail-sync-source-api/proto/hermes/contacts/mail_sync_source/v1/mail_sync_source.proto',
    BACKEND_ROOT,
  ),
  sourceApi: new URL('src/contacts-mail-sync-source-api/src/lib.rs', BACKEND_ROOT),
  sourceEnvelope: new URL(
    'src/contacts-mail-sync-source-api/src/envelope.rs',
    BACKEND_ROOT,
  ),
  core: new URL('src/contacts-core/src/lib.rs', BACKEND_ROOT),
  identity: new URL('src/contacts-core/src/identity.rs', BACKEND_ROOT),
  upsert: new URL('src/contacts-core/src/upsert.rs', BACKEND_ROOT),
  persistenceManifest: new URL('src/contacts-persistence/Cargo.toml', BACKEND_ROOT),
  persistence: new URL('src/contacts-persistence/src/repository.rs', BACKEND_ROOT),
  migration: new URL(
    'src/contacts-persistence/migrations/0001_contacts.sql',
    BACKEND_ROOT,
  ),
  sourceMigration: new URL(
    'src/contacts-persistence/migrations/0002_mail_sync_source.sql',
    BACKEND_ROOT,
  ),
  runtimeManifest: new URL('src/contacts-runtime/Cargo.toml', BACKEND_ROOT),
  runtimeAdmission: new URL('src/contacts-runtime/src/admission.rs', BACKEND_ROOT),
  runtimeCommand: new URL('src/contacts-runtime/src/command.rs', BACKEND_ROOT),
  runtimeSource: new URL('src/contacts-runtime/src/source.rs', BACKEND_ROOT),
  managedRuntime: new URL('src/contacts-runtime/src/managed_runtime.rs', BACKEND_ROOT),
  assemblyManifest: new URL('src/contacts-assembly/Cargo.toml', BACKEND_ROOT),
  assembly: new URL('src/contacts-assembly/src/lib.rs', BACKEND_ROOT),
  developmentRelease: new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
  mailContractManifest: new URL('src/mail-address-book-contract/Cargo.toml', BACKEND_ROOT),
  mailContract: new URL(
    'src/mail-address-book-contract/proto/hermes/mail/address_book/v1/address_book.proto',
    BACKEND_ROOT,
  ),
  googlePeopleManifest: new URL('src/mail-google-people/Cargo.toml', BACKEND_ROOT),
  googlePeople: new URL('src/mail-google-people/src/lib.rs', BACKEND_ROOT),
  cardDavManifest: new URL('src/mail-carddav/Cargo.toml', BACKEND_ROOT),
  cardDav: new URL('src/mail-carddav/src/lib.rs', BACKEND_ROOT),
  workflowApiManifest: new URL('src/mail-contacts-sync-api/Cargo.toml', BACKEND_ROOT),
  workflowApi: new URL(
    'src/mail-contacts-sync-api/proto/hermes/mail_contacts_sync/v1/sync.proto',
    BACKEND_ROOT,
  ),
  workflowCoreManifest: new URL('src/mail-contacts-sync-core/Cargo.toml', BACKEND_ROOT),
  workflowCore: new URL('src/mail-contacts-sync-core/src/lib.rs', BACKEND_ROOT),
  workflowPersistenceManifest: new URL(
    'src/mail-contacts-sync-persistence/Cargo.toml',
    BACKEND_ROOT,
  ),
  workflowPersistence: new URL(
    'src/mail-contacts-sync-persistence/src/repository.rs',
    BACKEND_ROOT,
  ),
  workflowOrchestration: new URL(
    'src/mail-contacts-sync-persistence/src/orchestration.rs',
    BACKEND_ROOT,
  ),
  workflowRelay: new URL('src/mail-contacts-sync-persistence/src/relay.rs', BACKEND_ROOT),
  workflowRealtime: new URL(
    'src/mail-contacts-sync-persistence/src/realtime.rs',
    BACKEND_ROOT,
  ),
  workflowMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0001_mail_contacts_sync.sql',
    BACKEND_ROOT,
  ),
  workflowOrchestrationMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0002_mail_contacts_sync_orchestration.sql',
    BACKEND_ROOT,
  ),
  workflowReverseMigration: new URL(
    'src/mail-contacts-sync-persistence/migrations/0003_reverse_sync.sql',
    BACKEND_ROOT,
  ),
  workflowReversePersistence: new URL(
    'src/mail-contacts-sync-persistence/src/reverse_sync.rs',
    BACKEND_ROOT,
  ),
  workflowRuntimeManifest: new URL('src/mail-contacts-sync-runtime/Cargo.toml', BACKEND_ROOT),
  workflowRuntimeAdmission: new URL('src/mail-contacts-sync-runtime/src/admission.rs', BACKEND_ROOT),
  workflowManagedRuntime: new URL('src/mail-contacts-sync-runtime/src/managed_runtime.rs', BACKEND_ROOT),
  workflowRuntimeMain: new URL('src/mail-contacts-sync-runtime/src/main.rs', BACKEND_ROOT),
  workflowScheduler: new URL('src/mail-contacts-sync-runtime/src/scheduler_due.rs', BACKEND_ROOT),
  workflowReverseChange: new URL(
    'src/mail-contacts-sync-runtime/src/reverse_change.rs',
    BACKEND_ROOT,
  ),
  workflowSourceResults: new URL(
    'src/mail-contacts-sync-runtime/src/source_results.rs',
    BACKEND_ROOT,
  ),
};

test('mail contacts sync agreement keeps integration workflow and domain separate', async () => {
  const [adr, inventorySource, policySource] = await Promise.all([
    readFile(files.adr, 'utf8'),
    readFile(files.inventory, 'utf8'),
    readFile(files.policy, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const contactsGate = inventory.slices.find(
    ({ gate }) => gate === 'contacts_mail_identity_command_v1',
  );
  const workflowGate = inventory.slices.find(({ gate }) => gate === 'mail_contacts_sync_v1');

  assert.deepEqual(contactsGate, {
    gate: 'contacts_mail_identity_command_v1',
    role: 'domain',
    owner: 'contacts',
    state: 'implemented',
    dependsOn: ['client_gateway_v1'],
  });
  assert.deepEqual(workflowGate, {
    gate: 'mail_contacts_sync_v1',
    role: 'workflow',
    owner: 'mail_contacts_sync',
    state: 'planned',
    dependsOn: ['mail_account_lifecycle_v1', 'contacts_mail_identity_command_v1'],
  });
  assert.match(adr, /Mail integration владеет Google People\/CardDAV protocol/);
  assert.match(adr, /Contacts domain владеет person/);
  assert.match(adr, /Workflow\s+владеет направлением sync, correlation, checkpoints, retry/);
  assert.match(adr, /periodic polling[\s\S]*forbidden/i);
  assert.equal(
    policy.implementation.currentSlice,
    'mail_address_book_provider_adapters_v1',
  );
  assert(policy.implementation.ownerInventory.domains.includes('contacts'));
  assert(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'contacts.mail-identity.command.v1',
    ),
  );
  assert(policy.implementation.ownerInventory.workflows.includes('mail_contacts_sync'));
  assert(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'mail.address-book.provider.v1',
    ),
  );
});

test('managed sync runtime uses staged settings and exact event-only owner contracts', async () => {
  const [manifest, admission, runtime, main, scheduler, reverseChange, sourceResults] = await Promise.all([
    readFile(files.workflowRuntimeManifest, 'utf8'),
    readFile(files.workflowRuntimeAdmission, 'utf8'),
    readFile(files.workflowManagedRuntime, 'utf8'),
    readFile(files.workflowRuntimeMain, 'utf8'),
    readFile(files.workflowScheduler, 'utf8'),
    readFile(files.workflowReverseChange, 'utf8'),
    readFile(files.workflowSourceResults, 'utf8'),
  ]);
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "mail_contacts_sync"/);
  assert.match(manifest, /surface = "runtime"/);
  assert.doesNotMatch(manifest, /hermes-mail-(?:runtime|persistence)|hermes-contacts-(?:runtime|persistence)/);
  assert.match(admission, /SchedulerJobRequestV1/);
  assert.match(admission, /DurableEnvelopeKindV1::Ack/);
  assert.match(admission, /DurableEnvelopeKindV1::Result/);
  assert.match(runtime, /request_managed_runtime_event_access_v2/);
  assert.match(runtime, /StorageVaultLeaseAdapterV1/);
  assert.match(main, /configuration_instance_id/);
  assert.match(main, /settings_snapshot_bytes/);
  assert.match(runtime, /pump_client_realtime_once/);
  assert.doesNotMatch(runtime, /reqwest|provider_kind\s*==|SELECT .*contacts|SELECT .*mail/i);
  assert.match(scheduler, /JOB_EXECUTE_CAPABILITY_V1/);
  assert.match(scheduler, /configuration_instance_id/);
  assert.doesNotMatch(scheduler, /account_id|provider_kind|access_token|refresh_token/);
  assert.match(reverseChange, /consume_contact_changed_once_v1/);
  assert.match(reverseChange, /SyncDirectionV1::Bidirectional/);
  assert.match(reverseChange, /remote_write_enabled/);
  assert.match(sourceResults, /consume_source_prepared_once_v1/);
  assert.match(sourceResults, /build_upsert_mail_address_book_entry_command_v1/);
  assert.doesNotMatch(`${reverseChange}\n${sourceResults}`, /BlobDataClient|provider_kind\s*==|reqwest/);
});

test('Mail address-book providers are separate bounded integration adapters', async () => {
  const [adr, policySource, googleManifest, google, cardDavManifest, cardDav] =
    await Promise.all([
      readFile(files.providerAdr, 'utf8'),
      readFile(files.policy, 'utf8'),
      readFile(files.googlePeopleManifest, 'utf8'),
      readFile(files.googlePeople, 'utf8'),
      readFile(files.cardDavManifest, 'utf8'),
      readFile(files.cardDav, 'utf8'),
    ]);
  const policy = JSON.parse(policySource);
  const packages = new Map(
    policy.implementation.productionPackages.map((descriptor) => [descriptor.name, descriptor]),
  );

  assert.match(adr, /hermes-mail-google-people/);
  assert.match(adr, /hermes-mail-carddav/);
  assert.match(adr, /никогда не выводит provider из[\s\S]*hostname, email suffix/);
  assert.match(adr, /mail_icloud_carddav_password/);
  assert.deepEqual(packages.get('hermes-mail-google-people'), {
    name: 'hermes-mail-google-people',
    role: 'integration',
    owner: 'mail',
    surface: 'implementation',
  });
  assert.deepEqual(packages.get('hermes-mail-carddav'), {
    name: 'hermes-mail-carddav',
    role: 'integration',
    owner: 'mail',
    surface: 'implementation',
  });

  for (const manifest of [googleManifest, cardDavManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "mail"/);
    assert.match(manifest, /surface = "implementation"/);
    assert.doesNotMatch(
      manifest,
      /hermes-(?:contacts|communications|mail-contacts-sync|events|storage|vault)/,
    );
  }
  assert.match(google, /GOOGLE_PEOPLE_API_HOST_V1: &str = "people.googleapis.com"/);
  assert.match(google, /GOOGLE_PEOPLE_CONTACTS_SCOPE_V1/);
  assert.match(google, /OutcomeUnknown/);
  assert.match(google, /expected_etag/);
  assert.match(google, /take\(\(MAX_RESPONSE_BYTES \+ 1\) as u64\)/);
  assert.doesNotMatch(google, /reqwest|sqlx|async_nats|hermes_contacts/i);

  assert.match(cardDav, /ICLOUD_CARDDAV_HOST_V1: &str = "contacts.icloud.com"/);
  assert.match(cardDav, /ICLOUD_CARDDAV_CREDENTIAL_PURPOSE_V1/);
  assert.match(cardDav, /ReadOnlyProvider/);
  assert.match(cardDav, /supports_remote_write/);
  assert.match(cardDav, /take\(\(MAX_RESPONSE_BYTES \+ 1\) as u64\)/);
  assert.doesNotMatch(cardDav, /reqwest|sqlx|async_nats|hermes_contacts/i);
});

test('Mail provider contract and sync workflow foundation preserve owner boundaries', async () => {
  const [
    mailContractManifest,
    mailContract,
    workflowApiManifest,
    workflowApi,
    workflowCoreManifest,
    workflowCore,
  ] = await Promise.all([
    readFile(files.mailContractManifest, 'utf8'),
    readFile(files.mailContract, 'utf8'),
    readFile(files.workflowApiManifest, 'utf8'),
    readFile(files.workflowApi, 'utf8'),
    readFile(files.workflowCoreManifest, 'utf8'),
    readFile(files.workflowCore, 'utf8'),
  ]);

  assert.match(mailContractManifest, /role = "integration"/);
  assert.match(mailContractManifest, /owner = "mail"/);
  assert.doesNotMatch(mailContractManifest, /hermes-contacts|hermes-communications/);
  assert.match(mailContract, /FetchMailAddressBookPageCommandV1/);
  assert.match(mailContract, /MailAddressBookEntryObservedV1/);
  assert.match(mailContract, /UpsertMailAddressBookEntryCommandV1/);
  assert.match(mailContract, /contact_snapshot_reference_id/);
  assert.match(mailContract, /contact_snapshot_custody_source_proof/);
  const upsertCommand = mailContract
    .split('message UpsertMailAddressBookEntryCommandV1')[1]
    .split('message MailAddressBookEntryUpsertedV1')[0];
  assert.match(upsertCommand, /reserved 9, 10/);
  assert.doesNotMatch(
    upsertCommand,
    /provider_kind|provider_entry_id|expected_provider_etag|display_name|email|phone/,
  );
  assert.match(mailContract, /outcome_unknown/);
  assert.doesNotMatch(
    mailContract,
    /access_token|refresh_token|password|cookie|raw_json|raw_xml|map</,
  );

  for (const manifest of [workflowApiManifest, workflowCoreManifest]) {
    assert.match(manifest, /role = "workflow"/);
    assert.match(manifest, /owner = "mail_contacts_sync"/);
    assert.doesNotMatch(manifest, /hermes-mail-(?:runtime|persistence)|hermes-contacts-(?:runtime|persistence)/);
  }
  assert.match(workflowApi, /rpc Start/);
  assert.match(workflowApi, /rpc Get/);
  assert.match(workflowApi, /MailContactsSyncStatusChangedV1/);
  assert.doesNotMatch(workflowApi, /Poll|provider_entry_id|provider_etag|credential|map</);
  assert.match(workflowCore, /MailContactsSyncStateV1/);
  assert.match(workflowCore, /ReconcilingOutcome/);
  assert.match(workflowCore, /MAIL_CONTACTS_SYNC_MAX_CURSOR_BYTES_V1/);
  assert.doesNotMatch(workflowCore, /reqwest|sqlx|provider sdk|oauth|gateway|nats/i);
});

test('sync persistence owns atomic state relay, reverse operations and SSE replay without foreign storage', async () => {
  const [
    manifest,
    repository,
    orchestration,
    reversePersistence,
    relay,
    realtime,
    migration,
    orchestrationMigration,
    reverseMigration,
  ] = await Promise.all([
    readFile(files.workflowPersistenceManifest, 'utf8'),
    readFile(files.workflowPersistence, 'utf8'),
    readFile(files.workflowOrchestration, 'utf8'),
    readFile(files.workflowReversePersistence, 'utf8'),
    readFile(files.workflowRelay, 'utf8'),
    readFile(files.workflowRealtime, 'utf8'),
    readFile(files.workflowMigration, 'utf8'),
    readFile(files.workflowOrchestrationMigration, 'utf8'),
    readFile(files.workflowReverseMigration, 'utf8'),
  ]);
  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "mail_contacts_sync"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.doesNotMatch(manifest, /hermes-mail-(?:runtime|persistence)|hermes-contacts-(?:runtime|persistence)/);
  assert.match(repository, /create_run/);
  assert.match(repository, /apply_transition/);
  assert.match(repository, /mail_contacts_sync_inbox/);
  assert.match(repository, /insert_outbox/);
  assert.match(repository, /insert_realtime/);
  assert.match(relay, /unpublished_commands/);
  assert.match(relay, /mark_command_published/);
  assert.match(realtime, /client_realtime_window/);
  assert.match(orchestration, /accept_provider_entry/);
  assert.match(orchestration, /accept_provider_page/);
  assert.match(orchestration, /accept_contact_outcome/);
  assert.match(orchestration, /account_pending_outcomes/);
  assert.match(reversePersistence, /accept_contact_changed_for_mail_sync/);
  assert.match(reversePersistence, /complete_contact_mail_sync_source/);
  for (const table of [
    'mail_contacts_sync_runs',
    'mail_contacts_sync_inbox',
    'mail_contacts_sync_outbox',
    'mail_contacts_sync_realtime',
  ]) {
    assert.match(migration, new RegExp(table));
  }
  assert.match(orchestrationMigration, /mail_contacts_sync_pages/);
  assert.match(orchestrationMigration, /mail_contacts_sync_entries/);
  assert.match(orchestrationMigration, /outcome_accounted/);
  assert.match(reverseMigration, /mail_contacts_sync_reverse_inbox/);
  assert.match(reverseMigration, /mail_contacts_sync_reverse_operations/);
  assert.doesNotMatch(
    `${migration}\n${orchestrationMigration}\n${reverseMigration}`,
    /hermes_data\.(?:contacts_state|contacts_provider_links|mail_accounts|communications_)/,
  );
  assert.doesNotMatch(
    `${repository}\n${orchestration}\n${reversePersistence}\n${relay}\n${realtime}`,
    /reqwest|oauth|provider sdk/i,
  );
});

test('staged Contacts slice keeps six functional build units isolated', async () => {
  const [
    sourceAdr,
    apiManifest,
    sourceApiManifest,
    coreManifest,
    proto,
    sourceProto,
    api,
    envelope,
    sourceApi,
    sourceEnvelope,
    core,
    identity,
    upsert,
    persistenceManifest,
    persistence,
    migration,
    sourceMigration,
    runtimeManifest,
    runtimeAdmission,
    runtimeCommand,
    runtimeSource,
    managedRuntime,
    assemblyManifest,
    assembly,
    developmentRelease,
  ] =
    await Promise.all([
      readFile(files.sourceAdr, 'utf8'),
      readFile(files.apiManifest, 'utf8'),
      readFile(files.sourceApiManifest, 'utf8'),
      readFile(files.coreManifest, 'utf8'),
      readFile(files.proto, 'utf8'),
      readFile(files.sourceProto, 'utf8'),
      readFile(files.api, 'utf8'),
      readFile(files.envelope, 'utf8'),
      readFile(files.sourceApi, 'utf8'),
      readFile(files.sourceEnvelope, 'utf8'),
      readFile(files.core, 'utf8'),
      readFile(files.identity, 'utf8'),
      readFile(files.upsert, 'utf8'),
      readFile(files.persistenceManifest, 'utf8'),
      readFile(files.persistence, 'utf8'),
      readFile(files.migration, 'utf8'),
      readFile(files.sourceMigration, 'utf8'),
      readFile(files.runtimeManifest, 'utf8'),
      readFile(files.runtimeAdmission, 'utf8'),
      readFile(files.runtimeCommand, 'utf8'),
      readFile(files.runtimeSource, 'utf8'),
      readFile(files.managedRuntime, 'utf8'),
      readFile(files.assemblyManifest, 'utf8'),
      readFile(files.assembly, 'utf8'),
      readFile(files.developmentRelease, 'utf8'),
    ]);

  for (const manifest of [apiManifest, sourceApiManifest, coreManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "contacts"/);
    assert.doesNotMatch(manifest, /hermes-mail|hermes-communications|sqlx|reqwest/);
  }
  assert.match(proto, /message UpsertContactFromMailAddressBookEntryCommandV1/);
  assert.match(proto, /message ContactUpsertedFromMailAddressBookEntryV1/);
  assert.match(proto, /message ContactUpsertFromMailAddressBookEntryRejectedV1/);
  assert.doesNotMatch(proto, /map<|bytes payload|token|password|cookie/);
  assert.match(api, /contacts\.mail-identity\.command\.v1/);
  assert.match(envelope, /validate_envelope_v1/);
  assert.match(sourceAdr, /шестая Contacts-owned unit/);
  assert.match(sourceProto, /message ContactChangedForMailSyncV1/);
  assert.match(sourceProto, /message PrepareContactMailSyncSourceCommandV1/);
  assert.match(sourceProto, /message ContactMailSyncSourceContentV1/);
  const changedEvent = sourceProto
    .split('message ContactChangedForMailSyncV1')[1]
    .split('message PrepareContactMailSyncSourceCommandV1')[0];
  assert.doesNotMatch(changedEvent, /display_name|email|phone|provider_kind|provider_entry_id|etag/);
  assert.match(sourceApi, /CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str = "mail"/);
  assert.match(sourceApi, /CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str = "hermes-mail-runtime"/);
  assert.match(sourceApi, /mail\.address-book\.contact-source\.blob\.v1/);
  assert.match(sourceEnvelope, /Semantics::Event\(EventMetadataV1/);
  assert.match(sourceEnvelope, /Semantics::Command\(CommandMetadataV1/);
  assert.match(core, /mod identity;[\s\S]*mod model;[\s\S]*mod upsert;/);
  assert.match(identity, /normalize_email_v1/);
  assert.match(identity, /normalize_phone_v1/);
  assert.match(upsert, /IdentityAmbiguous/);
  assert.match(upsert, /ProviderLinkConflict/);
  assert.doesNotMatch(
    `${core}\n${identity}\n${upsert}`,
    /provider sdk|oauth|postgres|gateway|nats|communications/i,
  );
  assert.match(persistenceManifest, /role = "domain"/);
  assert.match(persistenceManifest, /owner = "contacts"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.doesNotMatch(persistenceManifest, /hermes-mail|hermes-communications/);
  assert.match(persistence, /reserve_inbox/);
  assert.match(persistence, /reserve_contact_mail_sync_source/);
  assert.match(persistence, /persist_contact_mail_sync_source_result/);
  assert.match(persistence, /persist_contact/);
  assert.match(persistence, /insert_outbox/);
  assert.match(migration, /contacts_mail_entry_inbox/);
  assert.match(migration, /contacts_provider_links/);
  assert.match(migration, /contacts_outbox/);
  assert.match(sourceMigration, /contacts_mail_sync_source_inbox/);
  assert.doesNotMatch(migration, /mail_credential|communications_|tasks_|review_/);
  for (const manifest of [runtimeManifest, assemblyManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "contacts"/);
    assert.doesNotMatch(manifest, /hermes-mail|hermes-communications/);
  }
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(runtimeAdmission, /ModuleKindV1::Domain/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::DurableConsumer/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::DurablePublisher/);
  assert.match(runtimeAdmission, /StorageNamespaceRequestV1/);
  assert.doesNotMatch(runtimeAdmission, /ClientRpc|RequestRpc|QueryRpc/);
  assert.match(runtimeCommand, /consume_contacts_command_once_v1/);
  assert.match(runtimeCommand, /reject_mail_entry/);
  assert.match(runtimeCommand, /delivery\.acknowledge\(\)\.await/);
  assert.match(runtimeSource, /consume_contact_mail_sync_source_once_v1/);
  assert.ok(
    runtimeSource.indexOf('reserve_contact_mail_sync_source')
      < runtimeSource.indexOf('contact_mail_sync_source_snapshot'),
  );
  assert.match(runtimeSource, /request_managed_blob_session_v2/);
  assert.match(runtimeSource, /BlobDataOperationV1::BlobDataOperationWriteV1/);
  assert.match(runtimeSource, /CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1/);
  assert.match(managedRuntime, /StorageVaultLeaseAdapterV1/);
  assert.match(managedRuntime, /connect_runtime_with_jwt/);
  assert.match(managedRuntime, /signal_ready/);
  assert.doesNotMatch(managedRuntime, /hermes_mail|hermes_communications/);
  assert.match(assembly, /Unsigned Contacts release assembly/);
  assert.match(assembly, /contacts_storage_bundle_v1/);
  assert.match(assembly, /materialize_contacts_release_assembly_v1/);
  assert.doesNotMatch(assembly, /sign_release|launch_managed|KernelReleaseAuthorityV1/);
  assert.match(developmentRelease, /--package hermes-contacts-runtime/);
  assert.match(developmentRelease, /--package hermes-contacts-assembly/);
  assert.match(
    developmentRelease,
    /--artifact-fragment "\$contacts_assembly\/contacts\.release-artifacts\.json"/,
  );
});
