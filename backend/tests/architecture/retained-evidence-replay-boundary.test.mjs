import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const backendRoot = new URL('../../', import.meta.url);

async function read(path) {
  return readFile(new URL(path, backendRoot), 'utf8');
}

test('retained evidence replay protocol is an isolated workflow contract', async () => {
  const [manifest, policySource] = await Promise.all([
    read('src/attachment-preview-evidence-replay-protocol/Cargo.toml'),
    read('architecture/policy.json'),
  ]);
  const policy = JSON.parse(policySource);
  const descriptor = policy.implementation.productionPackages.find(
    ({ name }) => name === 'hermes-retained-evidence-replay-protocol',
  );

  assert.deepEqual(descriptor, {
    name: 'hermes-retained-evidence-replay-protocol',
    role: 'workflow',
    owner: 'attachment_preview_evidence_replay',
    surface: 'contract',
  });
  assert.match(manifest, /owner = "attachment_preview_evidence_replay"/);
  assert.doesNotMatch(manifest, /hermes-kernel/);
  assert.doesNotMatch(manifest, /hermes-events-jetstream/);
  assert.doesNotMatch(manifest, /sqlx/);
  assert.ok(
    policy.implementation.ownerInventory.workflows.includes(
      'attachment_preview_evidence_replay',
    ),
  );
});

test('replay selector is exact bounded and carries no generic query surface', async () => {
  const proto = await read(
    'src/attachment-preview-evidence-replay-protocol/proto/hermes/events/replay/v1/retained_evidence_replay.proto',
  );
  const implementation = await read(
    'src/attachment-preview-evidence-replay-protocol/src/lib.rs',
  );

  assert.match(proto, /string producer_registration_id = 5;/);
  assert.match(proto, /uint64 producer_runtime_generation = 6;/);
  assert.match(proto, /uint64 producer_grant_epoch = 7;/);
  assert.match(proto, /repeated bytes original_message_ids = 9;/);
  assert.match(implementation, /RETAINED_EVIDENCE_REPLAY_MAX_MESSAGES_V1: usize = 16/);
  assert.doesNotMatch(proto, /subject/);
  assert.doesNotMatch(proto, /predicate/);
  assert.doesNotMatch(proto, /payload_bytes/);
  assert.doesNotMatch(proto, /map</);
});

test('Communications retained replay persistence is an owner-local build unit', async () => {
  const [manifest, repository, migration, storageBundle, policySource] = await Promise.all([
    read('src/communications-retained-evidence-replay-persistence/Cargo.toml'),
    read('src/communications-retained-evidence-replay-persistence/src/repository.rs'),
    read(
      'src/communications-retained-evidence-replay-persistence/migrations/0001_retained_evidence_replay.sql',
    ),
    read('src/communications-runtime/src/storage_bundle.rs'),
    read('architecture/policy.json'),
  ]);
  const policy = JSON.parse(policySource);
  const descriptor = policy.implementation.productionPackages.find(
    ({ name }) =>
      name === 'hermes-communications-retained-evidence-replay-persistence',
  );

  assert.deepEqual(descriptor, {
    name: 'hermes-communications-retained-evidence-replay-persistence',
    role: 'domain',
    owner: 'communications',
    surface: 'persistence',
  });
  assert.match(manifest, /owner = "communications"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.doesNotMatch(manifest, /hermes-(?:mail|attachment-security|kernel)/);
  assert.match(repository, /communications_domain_outbox/);
  assert.match(repository, /OutboxRecordV1::accept/);
  assert.match(repository, /decode_envelope_v1/);
  assert.match(repository, /COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256/);
  assert.match(repository, /ON CONFLICT \(operation_id, original_message_id, logical_attempt, phase\) DO NOTHING/);
  assert.match(storageBundle, /append_communications_retained_evidence_replay_storage_v1/);
  assert.match(migration, /REFERENCES hermes_data\.communications_domain_outbox/);
  assert.doesNotMatch(migration, /REFERENCES hermes_data\.(?:mail|attachment_security)_/);
  assert.doesNotMatch(migration, /\b(?:UPDATE|DELETE)\b/);
});

test('Communications replay storage is an additive exact revision 17 successor', async () => {
  const schema = await read(
    'src/communications-retained-evidence-replay-persistence/src/schema.rs',
  );

  assert.match(
    schema,
    /COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1: u32 = 17/,
  );
  assert.match(schema, /predecessor\.owner_id != "communications"/);
  assert.match(schema, /predecessor\.bundle_id != "communications_state"/);
  assert.match(schema, /predecessor\.steps\.push\(StorageMigrationStepV1/);
});
