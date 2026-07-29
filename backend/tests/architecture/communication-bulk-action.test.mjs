import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('bulk delivery is a bounded workflow gate separate from domains and integrations', async () => {
  const [adr, inventorySource] = await Promise.all([
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
  ]);
  const inventory = JSON.parse(inventorySource);
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
});
