import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('delayed delivery stays a planned workflow behind Scheduler and event-only schedule control', async () => {
  const [adr, inventorySource, schedulerAdr] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0341-scheduled-communication-delivery-workflow.md',
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
    readFile(
      new URL(
        'docs/adr/ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
  ]);

  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find(
    (capability) => capability.gate === 'communication_delayed_delivery_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communication_delayed_delivery_v1',
    role: 'workflow',
    owner: 'communication_delayed_delivery',
    state: 'planned',
    dependsOn: [
      'communication_delivery_intent_v1',
      'scheduler_module_schedule_control_v1',
    ],
  });
  assert.match(adr, /Состояние реализации: не реализовано/);
  assert.match(adr, /scheduler\.schedule\.command\.v1/);
  assert.match(adr, /scheduler\.schedule\.result\.v1/);
  assert.match(adr, /ScheduledJobCommandV1/);
  assert.match(adr, /communication\.delivery_intent\.command/);
  assert.match(adr, /DurableEnvelopeV1/);
  assert.match(adr, /workflow-owned encrypted Blob custody/);
  assert.match(adr, /не вызывает Gateway/);
  assert.match(adr, /не импортирует Communications implementation/);
  assert.match(schedulerAdr, /gate реализован/);
  assert.doesNotMatch(
    adr,
    /direct (?:domain|integration|Scheduler) (?:call|socket|SQL)/i,
  );
});
