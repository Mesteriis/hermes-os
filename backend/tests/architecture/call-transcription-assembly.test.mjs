import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('call transcription assembly emits only unsigned workflow release inputs', async () => {
  const [policySource, manifest, assembly, runtimeManifest] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/call-transcription-assembly/Cargo.toml'),
    read('backend/src/call-transcription-assembly/src/lib.rs'),
    read('backend/src/call-transcription-runtime/Cargo.toml'),
  ]);
  const policy = JSON.parse(policySource);
  const packages = new Map(
    policy.implementation.productionPackages.map((descriptor) => [descriptor.name, descriptor]),
  );

  assert.equal(policy.implementation.currentSlice, 'call_transcription_managed_conformance_v1');
  assert.deepEqual(packages.get('hermes-call-transcription-assembly'), {
    name: 'hermes-call-transcription-assembly',
    role: 'workflow',
    owner: 'call_transcription',
    surface: 'assembly',
  });
  assert.match(manifest, /owner = "call_transcription"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.match(manifest, /hermes-call-transcription-runtime/);
  assert.match(manifest, /hermes-call-transcription-persistence/);
  assert.doesNotMatch(
    manifest,
    /hermes-communications|hermes-desktop-call-recording|hermes-speech-to-text|hermes-whisper/,
  );
  assert.match(assembly, /module_runtime/);
  assert.match(assembly, /storage_bundle/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(assembly, /mode\(0o600\)/);
  assert.match(assembly, /file_type\(\)\.is_symlink\(\)/);
  assert.doesNotMatch(assembly, /Command::|serve-inherited|tokio|sqlx|async_nats|JetStream/);
  assert.match(runtimeManifest, /\[\[bin\]\]/);
});
