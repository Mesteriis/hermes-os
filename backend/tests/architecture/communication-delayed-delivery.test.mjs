import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('delayed delivery admits separate contract and policy units while its runtime gate stays planned', async () => {
  const [
    adr,
    inventorySource,
    schedulerAdr,
    apiManifest,
    apiSource,
    apiProto,
    coreManifest,
    coreSource,
    persistenceManifest,
    persistenceSource,
    persistenceMigration,
  ] = await Promise.all([
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
    readFile(
      new URL('src/communication-delayed-delivery-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-api/proto/hermes/communication_delayed_delivery/v1/delivery.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-delayed-delivery-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/src/lib.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delayed-delivery-persistence/migrations/0001_delayed_delivery_state.sql',
        BACKEND_ROOT,
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
  assert.match(adr, /Состояние реализации: частично реализовано/);
  assert.match(
    apiManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "contract"/,
  );
  assert.match(
    coreManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "implementation"/,
  );
  assert.match(apiSource, /COMMUNICATION_DELAYED_DELIVERY_MAX_REQUEST_BYTES_V1/);
  assert.match(apiProto, /rpc Schedule/);
  assert.match(apiProto, /rpc Cancel/);
  assert.match(apiProto, /rpc GetStatus/);
  assert.match(coreSource, /MIN_DELIVERY_DELAY_MILLIS_V1: u64 = 5_000/);
  assert.match(coreSource, /MAX_DELIVERY_DELAY_MILLIS_V1/);
  assert.match(coreSource, /SchedulerCancelOutcomeV1::TooLate/);
  assert.match(
    persistenceManifest,
    /owner = "communication_delayed_delivery"[\s\S]*surface = "persistence"/,
  );
  assert.match(persistenceSource, /DelayedDeliveryBodyReceiptV1/);
  assert.match(persistenceSource, /SchedulerExecutionFenceV1/);
  assert.match(
    persistenceMigration,
    /communication_delayed_delivery_scheduler_inbox/,
  );
  assert.match(persistenceMigration, /communication_delayed_delivery_outbox/);
  assert.doesNotMatch(persistenceMigration, /body_utf8|provider_id|account_id/);
  assert.doesNotMatch(apiProto, /provider_id|account_id|map</);
  for (const source of [apiSource, coreSource]) {
    assert.doesNotMatch(source, /async_nats|sqlx|kernel::/);
  }
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
