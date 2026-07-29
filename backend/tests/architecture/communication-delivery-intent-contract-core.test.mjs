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
  contract: new URL(
    'src/communication-delivery-intent-api/proto/hermes/communication_delivery_intent/v1/delivery.proto',
    BACKEND_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0330-provider-neutral-communication-delivery-intent-workflow.md',
    PROJECT_ROOT,
  ),
};

test('delivery intent contract/core is an exact non-admitted workflow slice', async () => {
  const [policySource, reconstructionSource, apiManifest, coreManifest, contract, adr] =
    await Promise.all([
      readFile(paths.policy, 'utf8'),
      readFile(paths.reconstruction, 'utf8'),
      readFile(paths.apiManifest, 'utf8'),
      readFile(paths.coreManifest, 'utf8'),
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
    'communication_delivery_intent_contract_core_v1',
  );
  assert.deepEqual(policy.implementation.ownerInventory.workflows, ['communications_export']);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_delivery_intent')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'hermes-communication-delivery-intent-api:contract',
      'hermes-communication-delivery-intent-core:implementation',
    ],
  );
  assert.equal(deliverySlice?.state, 'planned');
  assert.equal(exportSlice?.state, 'implemented');
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.match(coreManifest, /hermes-communications-api/);
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}`,
    /hermes-(?:mail|telegram|whatsapp|zulip|communications-domain|communications-persistence)/,
  );
  assert.match(contract, /bytes conversation_id/);
  assert.match(contract, /optional bytes reply_to_message_id/);
  assert.doesNotMatch(contract, /\b(?:map|Any|provider_id|account_id)\b/);
  assert.match(adr, /Kernel[\s\S]*не декодирует request body/);
  assert.match(adr, /остаётся `planned`/);
});
