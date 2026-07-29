import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('managed client realtime keeps transport owner neutral and replay owner local', async () => {
  const [
    adr,
    inventorySource,
    protocol,
    validation,
    kernelRoute,
    routeStore,
    migration,
    ownerContract,
    ownerLedger,
    ownerAdapter,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0337-capability-routed-managed-client-realtime.md',
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
        'src/platform/runtime_protocol/proto/hermes/runtime/v1/managed_runtime_control.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/runtime_protocol/src/validation/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/kernel/src/platform/client_realtime.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/kernel/control_store/sqlite/src/module_state/client_realtime_route.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/migrations/0003_client_realtime_replay.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-api/proto/hermes/communication_delivery_intent/v1/delivery.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-persistence/src/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-delivery-intent-runtime/src/client_realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const platformGate = inventory.slices.find(
    ({ gate }) => gate === 'capability_routed_managed_client_realtime_v1',
  );
  const deliveryGate = inventory.slices.find(
    ({ gate }) => gate === 'communication_delivery_intent_v1',
  );
  const clientEvent = ownerContract.match(
    /message DeliveryIntentStatusChangedV1 \{[\s\S]*?\n\}/,
  )?.[0];

  assert.deepEqual(platformGate, {
    gate: 'capability_routed_managed_client_realtime_v1',
    role: 'platform',
    owner: 'kernel_capability_router',
    state: 'planned',
    dependsOn: ['client_gateway_v1', 'module_control_plane_v1'],
  });
  assert.equal(deliveryGate?.state, 'planned');
  assert.ok(
    deliveryGate?.dependsOn.includes(
      'capability_routed_managed_client_realtime_v1',
    ),
  );
  assert.match(adr, /bounded durable replay window/i);
  assert.match(protocol, /message ManagedRuntimeClientRealtimePublishRequestV1/);
  assert.match(protocol, /publish_client_realtime = 11/);
  assert.match(validation, /MAX_PAYLOAD_BYTES: usize = 64 \* 1024/);
  assert.match(kernelRoute, /current_managed_runtime_matches/);
  assert.match(kernelRoute, /approved_module_client_realtime_routes/);
  assert.match(kernelRoute, /initial_owner_identity/);
  assert.match(routeStore, /validate_client_realtime_routes/);
  assert.match(migration, /realtime_sequence BIGINT GENERATED ALWAYS AS IDENTITY/);
  assert.match(ownerLedger, /client_realtime_window/);
  assert.match(ownerLedger, /ORDER BY realtime_sequence ASC/);
  assert.match(ownerAdapter, /request_next_with_dispatch/);
  assert.match(ownerAdapter, /communication-delivery-intent\/\{\}/);
  assert.ok(clientEvent);
  assert.doesNotMatch(
    clientEvent,
    /body|provider|account|cursor|credential|envelope/i,
  );
  assert.doesNotMatch(
    `${protocol}\n${validation}\n${kernelRoute}\n${routeStore}`,
    /hermes_communication_delivery_intent|DeliveryIntentStatusChangedV1/,
  );
});
