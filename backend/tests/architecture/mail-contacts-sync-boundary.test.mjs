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
  core: new URL('src/contacts-core/src/lib.rs', BACKEND_ROOT),
  identity: new URL('src/contacts-core/src/identity.rs', BACKEND_ROOT),
  upsert: new URL('src/contacts-core/src/upsert.rs', BACKEND_ROOT),
  persistenceManifest: new URL('src/contacts-persistence/Cargo.toml', BACKEND_ROOT),
  persistence: new URL('src/contacts-persistence/src/repository.rs', BACKEND_ROOT),
  migration: new URL(
    'src/contacts-persistence/migrations/0001_contacts.sql',
    BACKEND_ROOT,
  ),
  runtimeManifest: new URL('src/contacts-runtime/Cargo.toml', BACKEND_ROOT),
  runtimeAdmission: new URL('src/contacts-runtime/src/admission.rs', BACKEND_ROOT),
  runtimeCommand: new URL('src/contacts-runtime/src/command.rs', BACKEND_ROOT),
  managedRuntime: new URL('src/contacts-runtime/src/managed_runtime.rs', BACKEND_ROOT),
  assemblyManifest: new URL('src/contacts-assembly/Cargo.toml', BACKEND_ROOT),
  assembly: new URL('src/contacts-assembly/src/lib.rs', BACKEND_ROOT),
  developmentRelease: new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
  mailContractManifest: new URL('src/mail-address-book-contract/Cargo.toml', BACKEND_ROOT),
  mailContract: new URL(
    'src/mail-address-book-contract/proto/hermes/mail/address_book/v1/address_book.proto',
    BACKEND_ROOT,
  ),
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
  workflowRuntimeManifest: new URL('src/mail-contacts-sync-runtime/Cargo.toml', BACKEND_ROOT),
  workflowRuntimeAdmission: new URL('src/mail-contacts-sync-runtime/src/admission.rs', BACKEND_ROOT),
  workflowManagedRuntime: new URL('src/mail-contacts-sync-runtime/src/managed_runtime.rs', BACKEND_ROOT),
  workflowRuntimeMain: new URL('src/mail-contacts-sync-runtime/src/main.rs', BACKEND_ROOT),
  workflowScheduler: new URL('src/mail-contacts-sync-runtime/src/scheduler_due.rs', BACKEND_ROOT),
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
    'mail_contacts_sync_runtime_admission_v1',
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
  const [manifest, admission, runtime, main, scheduler] = await Promise.all([
    readFile(files.workflowRuntimeManifest, 'utf8'),
    readFile(files.workflowRuntimeAdmission, 'utf8'),
    readFile(files.workflowManagedRuntime, 'utf8'),
    readFile(files.workflowRuntimeMain, 'utf8'),
    readFile(files.workflowScheduler, 'utf8'),
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
  assert.match(mailContract, /expected_provider_etag/);
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

test('sync persistence owns atomic state relay and SSE replay without foreign storage', async () => {
  const [manifest, repository, orchestration, relay, realtime, migration, orchestrationMigration] = await Promise.all([
    readFile(files.workflowPersistenceManifest, 'utf8'),
    readFile(files.workflowPersistence, 'utf8'),
    readFile(files.workflowOrchestration, 'utf8'),
    readFile(files.workflowRelay, 'utf8'),
    readFile(files.workflowRealtime, 'utf8'),
    readFile(files.workflowMigration, 'utf8'),
    readFile(files.workflowOrchestrationMigration, 'utf8'),
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
  assert.doesNotMatch(`${migration}\n${orchestrationMigration}`, /hermes_data\.(?:contacts_|mail_accounts|communications_)/);
  assert.doesNotMatch(`${repository}\n${orchestration}\n${relay}\n${realtime}`, /reqwest|oauth|provider sdk/i);
});

test('staged Contacts slice keeps five functional build units isolated', async () => {
  const [
    apiManifest,
    coreManifest,
    proto,
    api,
    envelope,
    core,
    identity,
    upsert,
    persistenceManifest,
    persistence,
    migration,
    runtimeManifest,
    runtimeAdmission,
    runtimeCommand,
    managedRuntime,
    assemblyManifest,
    assembly,
    developmentRelease,
  ] =
    await Promise.all([
      readFile(files.apiManifest, 'utf8'),
      readFile(files.coreManifest, 'utf8'),
      readFile(files.proto, 'utf8'),
      readFile(files.api, 'utf8'),
      readFile(files.envelope, 'utf8'),
      readFile(files.core, 'utf8'),
      readFile(files.identity, 'utf8'),
      readFile(files.upsert, 'utf8'),
      readFile(files.persistenceManifest, 'utf8'),
      readFile(files.persistence, 'utf8'),
      readFile(files.migration, 'utf8'),
      readFile(files.runtimeManifest, 'utf8'),
      readFile(files.runtimeAdmission, 'utf8'),
      readFile(files.runtimeCommand, 'utf8'),
      readFile(files.managedRuntime, 'utf8'),
      readFile(files.assemblyManifest, 'utf8'),
      readFile(files.assembly, 'utf8'),
      readFile(files.developmentRelease, 'utf8'),
    ]);

  for (const manifest of [apiManifest, coreManifest]) {
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
  assert.match(persistence, /persist_contact/);
  assert.match(persistence, /insert_outbox/);
  assert.match(migration, /contacts_mail_entry_inbox/);
  assert.match(migration, /contacts_provider_links/);
  assert.match(migration, /contacts_outbox/);
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
