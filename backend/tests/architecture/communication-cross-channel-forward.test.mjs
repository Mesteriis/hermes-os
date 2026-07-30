import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('cross-channel forward is an explicit workflow rather than a domain or provider facade', async () => {
  const [adr, inventorySource] = await Promise.all([
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
  ]);
  const inventory = JSON.parse(inventorySource);
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
});
