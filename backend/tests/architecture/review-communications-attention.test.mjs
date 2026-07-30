import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('Review attention contract and core are separate owner units', async () => {
  const [apiManifest, coreManifest, api, proto, core] = await Promise.all([
    backendSource('src/review-attention-api/Cargo.toml'),
    backendSource('src/review-attention-core/Cargo.toml'),
    backendSource('src/review-attention-api/src/lib.rs'),
    backendSource(
      'src/review-attention-api/proto/hermes/review/attention/client/v1/client.proto',
    ),
    backendSource('src/review-attention-core/src/lib.rs'),
  ]);

  for (const manifest of [apiManifest, coreManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "review"/);
    assert.doesNotMatch(
      manifest,
      /communications-|mail-|telegram-|whatsapp-|zulip-|sqlx|kernel|gateway/,
    );
  }
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(api, /review\.communication-attention\.command\.v1/);
  assert.match(api, /review\.communication-attention\.query\.v1/);
  assert.match(api, /review\.communication-attention\.realtime\.v1/);
  assert.match(proto, /oneof operation/);
  assert.match(proto, /bytes source_evidence_id = 3/);
  assert.match(proto, /uint64 expected_revision = 4/);
  const coreProduction = core.replace(/#\[cfg\(test\)\][\s\S]*$/u, '');
  assert.doesNotMatch(
    `${proto}\n${coreProduction}`,
    /provider_call|provider_account|message_body|email_address|phone_number|google\.protobuf\.Any|map</,
  );
});

test('Review attention core owns optimistic revision and bounded snooze invariants', async () => {
  const core = await backendSource('src/review-attention-core/src/lib.rs');
  assert.match(core, /current\.revision != request\.expected_revision/);
  assert.match(core, /current\.source_evidence_id != request\.source_evidence_id/);
  assert.match(core, /MAX_SNOOZE_SECONDS_V1/);
  assert.match(core, /DismissedAttention/);
  assert.match(core, /attention\.pinned = false/);
  assert.match(core, /attention\.snoozed_until = None/);
  assert.match(core, /if changed \{/);
  assert.doesNotMatch(core.replace(/#\[cfg\(test\)\][\s\S]*$/u, ''), /serde_json|sqlx|tokio|prost/);
});

test('Review attention persistence is owner-local atomic and operation-idempotent', async () => {
  const [manifest, repository, schema, migration] = await Promise.all([
    backendSource('src/review-attention-persistence/Cargo.toml'),
    backendSource('src/review-attention-persistence/src/repository.rs'),
    backendSource('src/review-attention-persistence/src/schema.rs'),
    backendSource(
      'src/review-attention-persistence/migrations/0001_review_attention.sql',
    ),
  ]);

  assert.match(manifest, /role = "domain"/);
  assert.match(manifest, /owner = "review"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /hermes-review-attention-core/);
  assert.match(manifest, /hermes-storage-protocol/);
  assert.doesNotMatch(manifest, /communications-|mail-|telegram-|whatsapp-|zulip-/);
  assert.match(repository, /\.pool\s*\.begin\(\)/);
  assert.match(repository, /ON CONFLICT \(logical_owner_id, operation_id\) DO NOTHING/);
  assert.match(repository, /request_sha256/);
  assert.match(repository, /stored_sha256\.as_slice\(\) != request_sha256/);
  assert.match(repository, /FOR UPDATE/);
  assert.match(repository, /outcome\.changed/);
  assert.match(repository, /transaction\.commit\(\)/);
  assert.match(schema, /owner_id: "review"/);
  assert.match(migration, /hermes_data\.review_attention_state/);
  assert.match(migration, /hermes_data\.review_attention_operations/);
  assert.match(migration, /expected_revision BIGINT NOT NULL/);
  assert.doesNotMatch(
    `${repository}\n${migration}`,
    /communications_|mail_|telegram_|provider_|message_body|subject|email_address|phone_number/,
  );
});

test('Review owner admission does not prematurely open the managed gate', async () => {
  const [adr, inventorySource, policySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0351-review-communications-attention-owner-admission.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    backendSource('architecture/communications-settings-reconstruction.json'),
    backendSource('architecture/policy.json'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'review_communications_attention_v1',
  );

  assert.deepEqual(gate, {
    gate: 'review_communications_attention_v1',
    role: 'domain',
    owner: 'review',
    state: 'planned',
    dependsOn: ['communications_canonical_read_v2'],
  });
  assert.equal(
    policy.implementation.currentSlice,
    'review_communications_attention_persistence_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.domains, [
    'communications',
    'review',
  ]);
  assert.match(adr, /Review packages не зависят от Communications packages/);
  assert.match(adr, /Он не открывает `review_communications_attention_v1`/);
  assert.match(adr, /operation ID вместе с exact request hash/);
  assert.match(adr, /live managed proof through Gateway and shared SSE/);
});
