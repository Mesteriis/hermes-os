import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  reconstruction: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  apiManifest: new URL('src/communication-delivery-intent-api/Cargo.toml', BACKEND_ROOT),
  coreManifest: new URL('src/communication-delivery-intent-core/Cargo.toml', BACKEND_ROOT),
  persistenceManifest: new URL(
    'src/communication-delivery-intent-persistence/Cargo.toml',
    BACKEND_ROOT,
  ),
  persistence: new URL(
    'src/communication-delivery-intent-persistence/src/intents.rs',
    BACKEND_ROOT,
  ),
  migration: new URL(
    'src/communication-delivery-intent-persistence/migrations/0001_delivery_intent_state.sql',
    BACKEND_ROOT,
  ),
  contract: new URL(
    'src/communication-delivery-intent-api/proto/hermes/communication_delivery_intent/v1/delivery.proto',
    BACKEND_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0330-provider-neutral-communication-delivery-intent-workflow.md',
    PROJECT_ROOT,
  ),
};

test('delivery intent persistence is an exact non-admitted workflow slice', async () => {
  const [
    policySource,
    reconstructionSource,
    apiManifest,
    coreManifest,
    persistenceManifest,
    persistence,
    migration,
    contract,
    adr,
  ] =
    await Promise.all([
      readFile(paths.policy, 'utf8'),
      readFile(paths.reconstruction, 'utf8'),
      readFile(paths.apiManifest, 'utf8'),
      readFile(paths.coreManifest, 'utf8'),
      readFile(paths.persistenceManifest, 'utf8'),
      readFile(paths.persistence, 'utf8'),
      readFile(paths.migration, 'utf8'),
      readFile(paths.contract, 'utf8'),
      readFile(paths.adr, 'utf8'),
    ]);
  const policy = JSON.parse(policySource);
  const reconstruction = JSON.parse(reconstructionSource);
  const deliverySlice = reconstruction.slices.find(
    ({ gate }) => gate === 'communication_delivery_intent_v1',
  );
  const exportSlice = reconstruction.slices.find(
    ({ gate }) => gate === 'communications_export_v1',
  );

  assert.equal(
    policy.implementation.currentSlice,
    'communication_delivery_intent_persistence_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.workflows, ['communications_export']);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_delivery_intent')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'hermes-communication-delivery-intent-api:contract',
      'hermes-communication-delivery-intent-core:implementation',
      'hermes-communication-delivery-intent-persistence:persistence',
    ],
  );
  assert.equal(deliverySlice?.state, 'planned');
  assert.equal(exportSlice?.state, 'implemented');
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.match(
    persistenceManifest,
    /role = "workflow"[\s\S]*surface = "persistence"/,
  );
  assert.match(coreManifest, /hermes-communications-api/);
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}\n${persistenceManifest}`,
    /hermes-(?:mail|telegram|whatsapp|zulip|communications-domain|communications-persistence)/,
  );
  assert.doesNotMatch(persistence, /PlannedDeliveryIntentV1|pub body_utf8/);
  assert.match(persistence, /SealedDeliveryBodyV1/);
  assert.match(persistence, /ON CONFLICT \(logical_owner_id, intent_id\)/);
  assert.match(
    persistence,
    /jobs\.logical_owner_id = candidate\.logical_owner_id/,
  );
  assert.match(migration, /PRIMARY KEY \(logical_owner_id, intent_id\)/);
  assert.match(migration, /body_ciphertext/);
  assert.doesNotMatch(migration, /body_utf8|communications_messages|mail_|telegram_/);
  assert.match(contract, /bytes conversation_id/);
  assert.match(contract, /optional bytes reply_to_message_id/);
  assert.doesNotMatch(contract, /\b(?:map|Any|provider_id|account_id)\b/);
  assert.match(adr, /Kernel[\s\S]*не декодирует request body/);
  assert.match(adr, /Persistence unit не принимает `PlannedDeliveryIntentV1`/);
  assert.match(adr, /остаётся `planned`/);
});
