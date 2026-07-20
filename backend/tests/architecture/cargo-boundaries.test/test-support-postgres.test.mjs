import assert from 'node:assert/strict';
import test from 'node:test';

import { validateCargoMetadata } from '../../../scripts/lib/cargo-boundaries.mjs';
import {
  codes,
  dependency,
  kernel,
  metadata,
  runtimeProtocol,
  workspacePackage,
} from '../support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from '../support/canonical-policy.mjs';

test('allows sqlx only for exact test-only PostgreSQL conformance kits', () => {
  for (const name of ['hermes-events-jetstream-testkit', 'hermes-scheduler-testkit']) {
    const allowed = [...basePackages(), testSupport([dependency('sqlx', 'dev')], name)];
    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);
  }

  for (const [name, metadataOverrides, kind] of [
    ['hermes-scheduler-testkit', {}, null],
    ['hermes-scheduler-testkit', {}, 'build'],
    ['hermes-events-jetstream-testkit', { surface: 'implementation' }, 'dev'],
    ['hermes-other-testkit', {}, 'dev'],
  ]) {
    const packages = [
      ...basePackages(),
      testSupport([dependency('sqlx', kind)], name, metadataOverrides),
    ];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('storage_dependency'),
    );
  }
});

function basePackages() {
  return [
    kernel(),
    workspacePackage('hermes-events-protocol', {
      role: 'platform', owner: 'events', surface: 'contract',
    }),
    runtimeProtocol(),
  ];
}

function testSupport(dependencies, name = 'hermes-events-jetstream-testkit', overrides = {}) {
  return workspacePackage(
    name,
    { role: 'test', owner: 'test', surface: 'test_support', ...overrides },
    dependencies,
  );
}
