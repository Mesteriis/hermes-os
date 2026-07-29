import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('managed module request RPC foundation is typed bounded and separate from query RPC', async () => {
  const [adr, inventorySource, protocol, validation, control, supervisor] =
    await Promise.all([
      readFile(
        new URL(
          'docs/adr/ADR-0339-capability-routed-module-request-rpc.md',
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
          'src/platform/runtime_protocol/src/validation/module_request.rs',
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
    ({ gate }) => gate === 'capability_routed_module_request_rpc_v1',
  );
  const bulkGate = inventory.slices.find(
    ({ gate }) => gate === 'communication_bulk_action_v1',
  );

  assert.deepEqual(platformGate, {
    gate: 'capability_routed_module_request_rpc_v1',
    role: 'platform',
    owner: 'kernel_capability_router',
    state: 'planned',
    dependsOn: ['module_control_plane_v1'],
  });
  assert.ok(
    bulkGate?.dependsOn.includes('capability_routed_module_request_rpc_v1'),
  );
  assert.match(adr, /Kernel не повторяет request автоматически/);
  assert.match(protocol, /message ManagedRuntimeModuleRequestRequestV1/);
  assert.match(protocol, /message ManagedRuntimeModuleRequestDeliveryV1/);
  assert.match(protocol, /message ManagedRuntimeModuleRequestResponseV1/);
  assert.match(protocol, /route_module_request = 12/);
  assert.match(protocol, /deliver_module_request = 13/);
  assert.match(
    validation,
    /MODULE_REQUEST_MAX_PAYLOAD_BYTES_V1: usize = 64 \* 1024/,
  );
  assert.match(
    validation,
    /MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1: u32 = 30_000/,
  );
  assert.match(control, /trait ManagedRuntimeModuleRequestHandler/);
  assert.match(supervisor, /configure_module_request_handler/);
  assert.doesNotMatch(
    `${protocol}\n${validation}\n${control}\n${supervisor}`,
    /hermes_(?:communications|mail|telegram|whatsapp|zulip)|Communications|Mail|Telegram|WhatsApp|Zulip/,
  );
});
