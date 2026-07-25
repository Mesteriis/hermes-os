import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const MANAGED_VAULT_CLIENT = new URL(
  'src/platform/vault/managed_client/src/lib.rs',
  BACKEND_ROOT,
);

test('correlated managed Vault client owns resolve create and replace actions', async () => {
  const source = await readFile(MANAGED_VAULT_CLIENT, 'utf8');
  const implementationStart = source.indexOf(
    "impl<'a> ManagedProviderCredentialClientV2<'a>",
  );
  const implementationEnd = source.indexOf(
    '\nfn audience(',
    implementationStart,
  );

  assert.notEqual(implementationStart, -1);
  assert.notEqual(implementationEnd, -1);

  const implementation = source.slice(implementationStart, implementationEnd);
  assert.match(implementation, /pub fn resolve\(/);
  assert.match(implementation, /pub fn store_once\(/);
  assert.match(implementation, /VaultActionV1::Create/);
  assert.match(implementation, /pub fn replace_once\(/);
  assert.match(implementation, /VaultActionV1::ReplaceCas/);
  assert.doesNotMatch(implementation, /\.try_clone\(/);
});
