import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const ZULIP_RUNTIME_ROOT = new URL('src/zulip-runtime/src/', BACKEND_ROOT);

test('Zulip runtime uses one correlated managed-control frame pump', async () => {
  const managed = await readFile(
    new URL('managed.rs', ZULIP_RUNTIME_ROOT),
    'utf8',
  );

  assert.match(managed, /ManagedControlChannelV2<UnixStream>/);
  assert.match(managed, /ManagedProviderCredentialClientV2/);
  assert.match(managed, /request_managed_runtime_event_access_v2/);
  assert.match(managed, /request_managed_blob_session_v2/);
  assert.match(managed, /InheritedKernelVaultRouteV2/);
  assert.equal(
    managed.match(/capability_id: ZULIP_BLOB_CAPABILITY_ID/g)?.length,
    2,
  );
  assert.doesNotMatch(managed, /\.try_clone\(/);
  assert.doesNotMatch(managed, /ManagedProviderCredentialClientV1/);
  assert.doesNotMatch(managed, /InheritedKernelVaultRouteV1/);
  assert.doesNotMatch(managed, /request_managed_runtime_event_access\(/);
  assert.doesNotMatch(managed, /request_managed_blob_session\(/);
  assert.doesNotMatch(managed, /capability_id: "blob\.content"/);
  assert.doesNotMatch(managed, /MSG_PEEK/);

  const eventAccess = managed.indexOf(
    'request_managed_runtime_event_access_v2',
  );
  const ready = managed.indexOf('.signal_ready(');
  assert.ok(eventAccess >= 0);
  assert.ok(ready > eventAccess);
});
