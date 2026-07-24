import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const TELEGRAM_RUNTIME_ROOT = new URL('src/telegram-runtime/src/', BACKEND_ROOT);

test('Telegram runtime uses one correlated managed-control frame pump', async () => {
  const [managedControl, bootstrap, process, vaultCredentials] = await Promise.all([
    readFile(new URL('managed_control.rs', TELEGRAM_RUNTIME_ROOT), 'utf8'),
    readFile(new URL('bootstrap.rs', TELEGRAM_RUNTIME_ROOT), 'utf8'),
    readFile(new URL('process.rs', TELEGRAM_RUNTIME_ROOT), 'utf8'),
    readFile(new URL('vault_credentials.rs', TELEGRAM_RUNTIME_ROOT), 'utf8'),
  ]);
  const runtimeSources = [managedControl, bootstrap, process, vaultCredentials];

  assert.match(managedControl, /ManagedControlChannelV2<UnixStream>/);
  assert.match(bootstrap, /ManagedProviderCredentialClientV2/);
  assert.match(bootstrap, /request_managed_runtime_event_access_v2/);
  assert.match(process, /request_managed_blob_session_v2/);
  assert.match(vaultCredentials, /InheritedKernelVaultRouteV2/);

  for (const source of runtimeSources) {
    assert.doesNotMatch(source, /\.try_clone\(/);
    assert.doesNotMatch(source, /ManagedProviderCredentialClientV1/);
    assert.doesNotMatch(source, /request_managed_runtime_event_access\(/);
    assert.doesNotMatch(source, /request_managed_blob_session\(/);
    assert.doesNotMatch(source, /MSG_PEEK/);
  }
});
