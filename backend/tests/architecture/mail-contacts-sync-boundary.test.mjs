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
    state: 'planned',
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
    'contacts_mail_identity_command_persistence_v1',
  );
  assert(policy.implementation.ownerInventory.domains.includes('contacts'));
  assert(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'contacts.mail-identity.command.v1',
    ),
  );
});

test('staged Contacts slice keeps contract core and persistence as separate units', async () => {
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
});
