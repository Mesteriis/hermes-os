import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import test from 'node:test';

const here = path.dirname(fileURLToPath(import.meta.url));
const storeRoot = path.resolve(here, '../../src/platform/vault/store_sqlcipher/src');

test('Vault persistence owns one bounded actor and centralizes SQLCipher connection opening', async () => {
  const [actor, connection, store] = await Promise.all([
    readFile(path.join(storeRoot, 'actor/handle.rs'), 'utf8'),
    readFile(path.join(storeRoot, 'database/connection.rs'), 'utf8'),
    readFile(path.join(storeRoot, 'database/store.rs'), 'utf8'),
  ]);

  assert.match(actor, /ACTOR_QUEUE_CAPACITY: usize = 64/);
  assert.match(actor, /OPERATION_DEADLINE: Duration = Duration::from_secs\(2\)/);
  assert.match(actor, /std::thread::Builder/);
  assert.match(actor, /TrySendError::Full\(_\) => VaultStoreError::QueueFull/);
  assert.match(actor, /RecvTimeoutError::Timeout\) => Err\(VaultStoreError::DeadlineExceeded\)/);
  assert.match(connection, /Connection::open_with_flags/);
  assert.doesNotMatch(store, /Connection::open/);
});
