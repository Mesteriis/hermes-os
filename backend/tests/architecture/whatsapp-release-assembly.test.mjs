import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('WhatsApp descriptor, storage and release assembly remain separate owner units', async () => {
  const [
    workspace,
    persistenceManifest,
    persistenceSchema,
    runtimeManifest,
    admission,
    settings,
    runtimeMain,
    managedRuntime,
    assemblyManifest,
    assemblySource,
  ] = await Promise.all([
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/whatsapp-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-persistence/src/schema.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/whatsapp-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/whatsapp-runtime/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-runtime/src/settings.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/whatsapp-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/whatsapp-runtime/src/managed.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-assembly/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/whatsapp-assembly/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(workspace, /"src\/whatsapp-assembly"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistenceManifest, /hermes-storage-protocol/);
  assert.match(persistenceSchema, /owner_id: "whatsapp"\.to_owned\(\)/);
  assert.match(
    persistenceSchema,
    /forward_sql_utf8: WHATSAPP_SCHEMA_V1\.as_bytes\(\)\.to_vec\(\)/,
  );
  assert.match(
    persistenceSchema,
    /sha256: Sha256::digest\(WHATSAPP_SCHEMA_V1\.as_bytes\(\)\)\.to_vec\(\)/,
  );
  assert.match(persistenceSchema, /hermes_data\.whatsapp_/);
  assert.doesNotMatch(
    persistenceSchema,
    /CREATE TABLE[^\n]*hermes_data\.communications_/,
  );

  assert.match(admission, /whatsapp_module_descriptor_v1/);
  assert.match(admission, /ModuleKindV1::Integration/);
  assert.match(admission, /WhatsAppClientContractV1::Command/);
  assert.match(admission, /WhatsAppClientContractV1::Query/);
  assert.match(admission, /Request::HostCapability/);
  assert.match(admission, /communication_observed_publish_request_v1/);
  assert.match(settings, /"whatsapp\.account_id"/);
  assert.match(settings, /SettingClientVisibilityV1::Hidden/);
  assert.match(runtimeMain, /settings::decode\(&snapshot\)/);
  assert.match(managedRuntime, /account_id: settings\.account_id\.clone\(\)/);
  assert.match(
    managedRuntime,
    /provider_command_account_id\(command\) != self\.account_id/,
  );
  assert.match(managedRuntime, /envelope\.account_id != self\.account_id/);
  assert.match(managedRuntime, /account_id != self\.account_id/);
  assert.match(
    managedRuntime,
    /status\.filter\(\|value\| value\.account_id == self\.account_id\)/,
  );

  assert.match(assemblyManifest, /role = "integration"/);
  assert.match(assemblyManifest, /owner = "whatsapp"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  for (const dependency of [
    'hermes-whatsapp-runtime',
    'hermes-whatsapp-persistence',
    'hermes-runtime-protocol',
    'hermes-storage-protocol',
  ]) {
    assert.match(assemblyManifest, new RegExp(dependency));
  }
  for (const forbiddenDependency of [
    'hermes-kernel',
    'hermes-gateway',
    'hermes-communications',
    'ring',
    'sha2',
  ]) {
    assert.doesNotMatch(assemblyManifest, new RegExp(forbiddenDependency));
  }
  assert.doesNotMatch(runtimeManifest, /hermes-whatsapp-assembly/);
  assert.doesNotMatch(persistenceManifest, /hermes-whatsapp-assembly/);

  assert.match(
    assemblySource,
    /whatsapp_module_descriptor_v1\(build_id\)/,
  );
  assert.match(assemblySource, /whatsapp_settings_schema_v1\(\)/);
  assert.match(assemblySource, /whatsapp_storage_bundle_v1\(\)/);
  assert.match(assemblySource, /"module_runtime"\.to_owned\(\)/);
  assert.match(assemblySource, /"storage_bundle"\.to_owned\(\)/);
  assert.doesNotMatch(assemblySource, /SigningKey|sign_manifest/);
});
