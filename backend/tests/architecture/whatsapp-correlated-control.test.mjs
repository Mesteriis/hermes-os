import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const WHATSAPP_RUNTIME_ROOT = new URL(
  'src/whatsapp-runtime/src/',
  BACKEND_ROOT,
);

test('WhatsApp runtime uses one correlated managed-control frame pump', async () => {
  const managed = await readFile(
    new URL('managed.rs', WHATSAPP_RUNTIME_ROOT),
    'utf8',
  );

  assert.match(managed, /ManagedControlChannelV2<UnixStream>/);
  assert.match(managed, /request_managed_runtime_event_access_v2/);
  assert.match(managed, /InheritedKernelVaultRouteV2/);
  assert.doesNotMatch(managed, /\.try_clone\(/);
  assert.doesNotMatch(managed, /InheritedKernelVaultRouteV1/);
  assert.doesNotMatch(
    managed,
    /request_managed_runtime_event_access\(/,
  );
  assert.doesNotMatch(
    managed,
    /ManagedRuntimeControlResponseV1::decode/,
  );
  assert.doesNotMatch(managed, /fn read_frame\(/);
  assert.doesNotMatch(managed, /fn write_frame\(/);
  assert.doesNotMatch(managed, /MSG_PEEK/);

  const eventAccess = managed.indexOf(
    'request_managed_runtime_event_access_v2',
  );
  const ready = managed.indexOf('.signal_ready(');
  assert.ok(eventAccess >= 0);
  assert.ok(ready > eventAccess);
});
