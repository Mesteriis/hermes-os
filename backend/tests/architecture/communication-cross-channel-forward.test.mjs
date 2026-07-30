import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('cross-channel forward contract and core are isolated provider-neutral workflow units', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    apiManifest,
    coreManifest,
    api,
    core,
    contract,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0346-cross-channel-communication-forward-workflow.md',
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
      new URL('src/communication-cross-channel-forward-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-api/proto/hermes/communication_cross_channel_forward/v1/forward.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'communication_cross_channel_forward_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communication_cross_channel_forward_v1',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    state: 'planned',
    dependsOn: [
      'communication_delivery_intent_v1',
      'communications_content_read_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });
  assert.match(adr, /Caller не передаёт provider, account, provider chat locator или plaintext body/);
  assert.match(adr, /target-bound Blob delegation/);
  assert.match(adr, /не\s+импортирует provider contracts, SDK, runtime или persistence/);
  assert.match(adr, /Kernel[\s\S]*не декодирует source metadata, content или delivery payload/);
  assert.match(adr, /Core capability router[\s\S]*не содержит cross-channel business method/);
  assert.match(adr, /не хранит plaintext body/);
  assert.match(adr, /Принятый ADR сам по себе не открывает/);
  assert.equal(
    policy.implementation.currentSlice,
    'communication_cross_channel_forward_contract_core_v1',
  );
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_cross_channel_forward')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'hermes-communication-cross-channel-forward-api:contract',
      'hermes-communication-cross-channel-forward-core:implementation',
    ],
  );
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}`,
    /hermes-(?:communications-domain|mail|telegram|whatsapp|zulip|kernel)/,
  );
  assert.match(api, /COMMUNICATION_CROSS_CHANNEL_FORWARD_CAPABILITY_ID_V1/);
  assert.match(core, /CrossChannelForwardTransitionV1/);
  assert.match(core, /RevisionExhausted/);
  assert.doesNotMatch(
    contract,
    /provider_id|account_id|body_utf8|blob_reference|\bAny\b|\bmap\s*</,
  );
});
