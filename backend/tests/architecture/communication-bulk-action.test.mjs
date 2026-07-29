import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('bulk delivery contract and core are bounded workflow units separate from domains and integrations', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    apiManifest,
    coreManifest,
    api,
    core,
    contract,
  ] =
    await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0340-bounded-communication-bulk-delivery-workflow.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'architecture/communications-settings-reconstruction.json',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
      readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('src/communication-bulk-action-api/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/communication-bulk-action-core/Cargo.toml', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/communication-bulk-action-api/src/lib.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL('src/communication-bulk-action-core/src/lib.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'src/communication-bulk-action-api/proto/hermes/communication_bulk_action/v1/bulk_action.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
    ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'communication_bulk_action_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communication_bulk_action_v1',
    role: 'workflow',
    owner: 'communication_bulk_action',
    state: 'planned',
    dependsOn: [
      'communication_delivery_intent_v1',
      'capability_routed_module_request_rpc_v1',
    ],
  });
  assert.match(adr, /`1\.\.=100` targets/);
  assert.match(adr, /64 KiB module `request_rpc`/);
  assert.match(adr, /Kernel не retry-ит mutation/);
  assert.match(adr, /одну bounded lease/);
  assert.match(adr, /Private body[\s\S]*не попадает в logs\/events\/errors\/status/);
  assert.match(adr, /Принятый ADR сам по себе gate не открывает/);
  assert.equal(
    policy.implementation.currentSlice,
    'communication_bulk_action_contract_core_v1',
  );
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_bulk_action')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'hermes-communication-bulk-action-api:contract',
      'hermes-communication-bulk-action-core:implementation',
    ],
  );
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}`,
    /hermes-(?:communications-domain|mail|telegram|whatsapp|zulip|kernel)/,
  );
  assert.match(api, /COMMUNICATION_BULK_ACTION_MAX_TARGETS_V1: usize = 100/);
  assert.match(core, /MAX_TARGET_BODY_BYTES_V1: usize = 64 \* 1024/);
  assert.match(core, /DuplicateTargetId/);
  assert.doesNotMatch(contract, /provider_id|account_id|\bAny\b|\bmap\s*</);
});
