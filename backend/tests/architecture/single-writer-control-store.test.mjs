import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const kernelSource = new URL('../../src/kernel/src/', import.meta.url);

async function source(path) {
  return readFile(new URL(path, kernelSource), 'utf8');
}

test('online control-plane IPC uses the one shared SQLite actor facade', async () => {
  const files = await Promise.all([
    source('modules/registration/ipc.rs'),
    source('identity/owner_control/mod.rs'),
    source('identity/owner_control/dispatch.rs'),
    source('runtime/external/ipc.rs'),
  ]);
  for (const contents of files) {
    assert.doesNotMatch(contents, /open_validated_control_store|Connection::open/);
    assert.match(contents, /SqliteControlStore/);
  }
  const coordinator = await source('platform/control_plane.rs');
  assert.match(coordinator, /Arc<SqliteControlStore>/);
  assert.match(coordinator, /Some\(store\)/);
});
