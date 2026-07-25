import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('WhatsApp managed admission is wired as an integration-owned conformance slice', async () => {
  const [manifest, harness, runner] = await Promise.all([
    readFile(
      new URL('tests/support/kernel-recovery/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  for (const packageName of [
    'hermes-whatsapp-api',
    'hermes-whatsapp-persistence',
    'hermes-whatsapp-runtime',
  ]) {
    assert.match(manifest, new RegExp(`^${packageName} = `, 'm'));
  }

  for (const supportModule of [
    'whatsapp_managed_setup',
    'whatsapp_managed_fixture',
    'whatsapp_managed_flow',
  ]) {
    assert.match(harness, new RegExp(`mod ${supportModule};`));
  }

  assert.match(runner, /'-p',\s*'hermes-whatsapp-runtime'/);
  assert.match(
    runner,
    /managed_whatsapp_runtime_uses_signed_kernel_admission_and_host_route_fencing/,
  );
  assert.match(runner, /HERMES_WHATSAPP_RUNTIME_BIN:/);
});

test('WhatsApp managed launch receives an exact Kernel-fenced private host route', async () => {
  const [setup, managedRuntime, persistence] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/whatsapp_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-runtime/src/managed.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-persistence/src/durable.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(setup, /ManagedIntegrationHostBridgeConfigurationV1/);
  assert.match(
    setup,
    /managed_launch::start_staged_with_host_bridge_configuration/,
  );
  assert.match(setup, /reservation\.runtime_generation\(\)/);
  assert.match(setup, /reservation\.grant_epoch\(\)/);
  assert.match(setup, /route_binding_sha256/);
  assert.doesNotMatch(setup, /hermes_communications_(?:runtime|persistence)/);
  assert.doesNotMatch(
    managedRuntime,
    /durable\s*\.\s*initialize\s*\(/,
    'Storage Control applies the admitted bundle; WhatsApp runtime cannot run DDL',
  );
  assert.match(persistence, /\.database\(binding\.access\(\)\.pool_alias\(\)\)/);
  assert.match(
    persistence,
    /max_connections\(u32::from\(\s*binding\.access\(\)\.effective_budgets\(\)\.max_connections\(\)/,
  );
});
