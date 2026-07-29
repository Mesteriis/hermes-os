import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('managed module query RPC foundation is typed bounded and owner neutral', async () => {
  const [adr, inventorySource, protocol, validation, control, supervisor] =
    await Promise.all([
      readFile(
        new URL(
          'docs/adr/ADR-0336-capability-routed-module-query-rpc.md',
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
          'src/platform/runtime_protocol/src/validation/module_query.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(
        new URL('src/kernel/src/runtime/lifecycle/control.rs', BACKEND_ROOT),
        'utf8',
      ),
      readFile(
        new URL(
          'src/kernel/src/runtime/lifecycle/supervisor.rs',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
    ]);
  const inventory = JSON.parse(inventorySource);
  const platformGate = inventory.slices.find(
    ({ gate }) => gate === 'capability_routed_module_query_rpc_v1',
  );
  const deliveryGate = inventory.slices.find(
    ({ gate }) => gate === 'communication_delivery_intent_v1',
  );

  assert.deepEqual(platformGate, {
    gate: 'capability_routed_module_query_rpc_v1',
    role: 'platform',
    owner: 'kernel_capability_router',
    state: 'planned',
    dependsOn: ['module_control_plane_v1'],
  });
  assert.equal(deliveryGate?.state, 'planned');
  assert.ok(
    deliveryGate?.dependsOn.includes('capability_routed_module_query_rpc_v1'),
  );
  assert.match(adr, /caller не передаёт target registration/i);
  assert.match(protocol, /message ManagedRuntimeModuleQueryRequestV1/);
  assert.match(protocol, /message ManagedRuntimeModuleQueryDeliveryV1/);
  assert.match(protocol, /message ManagedRuntimeModuleQueryResponseV1/);
  assert.match(protocol, /route_module_query = 9/);
  assert.match(protocol, /deliver_module_query = 10/);
  assert.match(validation, /MODULE_QUERY_MAX_PAYLOAD_BYTES_V1: usize = 256 \* 1024/);
  assert.match(validation, /MODULE_QUERY_MAX_DEADLINE_MILLIS_V1: u32 = 10_000/);
  assert.match(validation, /response\.request_id/);
  assert.match(control, /trait ManagedRuntimeModuleQueryHandler/);
  assert.match(supervisor, /configure_module_query_handler/);
  assert.doesNotMatch(
    `${protocol}\n${validation}\n${control}\n${supervisor}`,
    /hermes_(?:communications|mail|telegram|whatsapp|zulip)|Communications|Mail|Telegram|WhatsApp|Zulip/,
  );
});
