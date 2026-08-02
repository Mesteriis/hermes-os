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
