import assert from 'node:assert/strict';
import test from 'node:test';

import { validateManifest } from '../../scripts/verify-macos-release-manifest.mjs';

const digest = '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';

function validManifest() {
  return {
    schema_version: 1,
    deployment_profile: 'macos_tauri_embedded_v1',
    runtime_lifecycle: 'managed_child',
    signing: { team_id: 'AB12CD34EF' },
    artifacts: {
      tauri_bundle: { path: '/Applications/Hermes Hub.app' },
      kernel_sidecar: {
        path: '/Applications/Hermes Hub.app/Contents/MacOS/hermes-kernel-aarch64-apple-darwin',
        sha256: digest,
      },
    },
  };
}

test('accepts a macOS managed-child release manifest with an exact Kernel digest', () => {
  assert.deepEqual(validateManifest(validManifest()), []);
});

test('rejects non-macOS profiles, malformed Team IDs and missing Kernel digest', () => {
  const wrongLifecycle = validManifest();
  wrongLifecycle.runtime_lifecycle = 'external_compose';
  assert.ok(validateManifest(wrongLifecycle).some((error) => error.includes('runtime_lifecycle')));

  const weakIdentity = validManifest();
  weakIdentity.signing.team_id = 'unverified';
  assert.ok(validateManifest(weakIdentity).some((error) => error.includes('team_id')));

  const noDigest = validManifest();
  delete noDigest.artifacts.kernel_sidecar.sha256;
  assert.ok(validateManifest(noDigest).some((error) => error.includes('sha256')));
});
