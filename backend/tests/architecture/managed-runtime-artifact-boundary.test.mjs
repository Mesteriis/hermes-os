import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('runtime artifact binding is one owner-neutral private bootstrap contract', async () => {
  const [binding, integration, workflow, engine, adr] = await Promise.all([
    backendSource(
      'src/platform/runtime_protocol/proto/hermes/runtime/v1/managed_runtime_artifact.proto',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/hermes/runtime/v1/managed_integration_runtime.proto',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/hermes/runtime/v1/managed_workflow_runtime.proto',
    ),
    backendSource(
      'src/platform/runtime_protocol/proto/hermes/runtime/v1/managed_engine_runtime.proto',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0372-kernel-staged-runtime-resources-for-managed-workflows.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(binding, /message ManagedRuntimeArtifactBindingV1/);
  assert.match(binding, /RuntimeArtifactUseV1 use = 2/);
  assert.doesNotMatch(integration, /message ManagedRuntimeArtifactBindingV1/);
  for (const configuration of [integration, workflow, engine]) {
    assert.match(configuration, /import "hermes\/runtime\/v1\/managed_runtime_artifact.proto"/);
    assert.match(configuration, /repeated ManagedRuntimeArtifactBindingV1 runtime_artifacts/);
  }
  assert.match(adr, /Gateway, Event Hub, Settings Registry, client API, health и telemetry этот\n+binding не видят/);
});

test('runtime resource types are exact and domains cannot request them', async () => {
  const [recovery, distribution, descriptor, validator] = await Promise.all([
    backendSource('src/platform/runtime_protocol/proto/hermes/runtime/v1/recovery.proto'),
    backendSource('src/platform/runtime_protocol/proto/hermes/runtime/v1/distribution.proto'),
    backendSource('src/platform/runtime_protocol/src/validation/descriptor.rs'),
    backendSource(
      'src/platform/runtime_protocol/src/validation/managed_runtime_artifact.rs',
    ),
  ]);

  assert.match(recovery, /RUNTIME_ARTIFACT_USE_V1_NATIVE_DYNAMIC_LIBRARY = 1/);
  assert.match(recovery, /RUNTIME_ARTIFACT_USE_V1_NATIVE_EXECUTABLE = 2/);
  assert.match(recovery, /RUNTIME_ARTIFACT_USE_V1_READ_ONLY_DATA = 3/);
  assert.match(distribution, /MODULE_RUNTIME_NATIVE_EXECUTABLE = 7/);
  assert.match(distribution, /MODULE_RUNTIME_READ_ONLY_DATA = 8/);
  assert.match(
    descriptor,
    /ModuleKindV1::Integration \| ModuleKindV1::Workflow \| ModuleKindV1::Engine/,
  );
  assert.match(descriptor, /workflow\.module_kind = ModuleKindV1::Domain/);
  assert.match(validator, /paths\.insert\(artifact\.staged_path\.as_str\(\)\)/);
  assert.match(validator, /artifact\.sha256\.iter\(\)\.any/);
});
