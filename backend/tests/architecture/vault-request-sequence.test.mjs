import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

async function source(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('Vault request replay fencing uses one canonical audience sequence contract', async () => {
  const [
    protocol,
    runtime,
    managedClient,
    ownerDerivedKey,
    storageSession,
    kernelCredential,
    eventSession,
    blobSession,
    ownerProvisioning,
    managedEventCredential,
    schedulerCredential,
  ] = await Promise.all([
    source('src/platform/runtime_protocol/src/vault_request_id.rs'),
    source('src/platform/vault/runtime/src/transport/session.rs'),
    source('src/platform/vault/managed_client/src/lib.rs'),
    source('src/platform/vault/managed_client/src/owner_derived_key.rs'),
    source('src/platform/storage/vault/src/route/session.rs'),
    source('src/platform/storage/vault/src/route/kernel_credential.rs'),
    source('src/platform/events/jetstream/src/vault/session.rs'),
    source('src/platform/blob/runtime/src/vault/session.rs'),
    source('src/kernel/src/platform/vault/owner_provisioning/mod.rs'),
    source('src/platform/events/jetstream/src/connection/managed_runtime.rs'),
    source('src/platform/scheduler/jetstream/src/transport/credential.rs'),
  ]);

  assert.match(protocol, /REQUEST_STREAM_ID: OnceLock<Option<u64>>/);
  assert.match(protocol, /getrandom::fill/);
  assert.match(protocol, /AtomicU64/);
  assert.match(protocol, /checked_add\(1\)/);
  assert.match(runtime, /stream_high_watermarks: BTreeMap/);
  assert.match(runtime, /replay_stream_key\(audience, stream_id\)/);
  assert.match(runtime, /sequence <= \*high_watermark/);
  assert.match(runtime, /MAX_TRANSPORT_REPLAY_STREAMS/);

  for (const client of [
    managedClient,
    storageSession,
    kernelCredential,
    eventSession,
    blobSession,
    managedEventCredential,
    schedulerCredential,
  ]) {
    assert.match(client, /next_vault_transport_request_id_v1/);
    assert.doesNotMatch(client, /getrandom::fill/);
    assert.doesNotMatch(client, /random_request_id/);
  }
  assert.match(ownerProvisioning, /next_vault_transport_request_id_v1/);
  assert.doesNotMatch(ownerProvisioning, /lease_request_id = random_bytes/);
  assert.doesNotMatch(ownerProvisioning, /command_request_id = random_bytes/);
  assert.match(ownerDerivedKey, /next_request_id/);
  assert.doesNotMatch(ownerDerivedKey, /getrandom::fill/);
  assert.doesNotMatch(ownerDerivedKey, /random_request_id/);
});
