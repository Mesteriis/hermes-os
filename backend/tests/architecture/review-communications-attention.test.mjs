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
    'review_communications_attention_contract_core_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.domains, [
    'communications',
    'review',
  ]);
  assert.match(adr, /Review packages не зависят от Communications packages/);
  assert.match(adr, /Он не открывает `review_communications_attention_v1`/);
  assert.match(adr, /live managed proof through Gateway and shared SSE/);
});
