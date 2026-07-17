import assert from 'node:assert/strict';
import test from 'node:test';

import { validateCargoMetadata } from '../../../scripts/lib/cargo-boundaries.mjs';
import {
  codes,
  dependency,
  kernel,
  metadata as fixtureMetadata,
  runtimeProtocol,
  storagePackages,
  storageProtocol,
  vaultPackages,
  vaultProtocol,
  workspacePackage,
} from '../support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from '../support/canonical-policy.mjs';

import { eventsProtocol, metadata } from './support.mjs';

test('allows a phase-specific subset of constitutional Kernel components', () => {
  const packages = [
    kernel([
      dependency('hermes-events-protocol'),
      dependency('hermes-runtime-protocol'),
    ], {
      components: ['supervisor', 'core_gateway'],
    }),
    workspacePackage('hermes-events-protocol', {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
    }),
    runtimeProtocol(),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});


for (const forbiddenDependency of ['async-nats', 'nats', 'sqlx', 'tokio-postgres', 'postgres', 'diesel', 'sea-orm']) {
  test(`keeps Telemetry Collector independent of ${forbiddenDependency}`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'hermes-telemetry-collector',
        {
          role: 'platform',
          owner: 'telemetry',
          surface: 'runtime',
          components: ['telemetry_collector'],
        },
        [dependency(forbiddenDependency)],
      ),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('telemetry_dependency'));
  });
}



test('prevents a telemetry implementation helper from bypassing collector dependency rules', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-telemetry-exporter',
      {
        role: 'platform',
        owner: 'telemetry',
        surface: 'implementation',
      },
      [dependency('sqlx')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('telemetry_dependency'));
});



test('prevents Kernel from linking Telemetry Collector implementation', () => {
  const collector = workspacePackage('hermes-telemetry-collector', {
    role: 'platform',
    owner: 'telemetry',
    surface: 'runtime',
    components: ['telemetry_collector'],
  });
  const packages = [kernel([dependency('hermes-telemetry-collector')]), collector];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('implementation_dependency'));
});



test('allows test support only through a dev dependency', () => {
  const support = workspacePackage('hermes-test-support', {
    role: 'test',
    owner: 'test',
    surface: 'test_support',
  });

  const allowed = [
    kernel(),
    support,
    workspacePackage(
      'hermes-contacts-runtime',
      { role: 'domain', owner: 'contacts', surface: 'runtime' },
      [dependency('hermes-test-support', 'dev')],
    ),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const forbidden = [
    kernel(),
    support,
    workspacePackage(
      'hermes-contacts-runtime',
      { role: 'domain', owner: 'contacts', surface: 'runtime' },
      [dependency('hermes-test-support')],
    ),
  ];
  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('production_test_dependency'));
});



test('rejects production use of test support from build dependencies', () => {
  const support = workspacePackage('hermes-test-support', {
    role: 'test',
    owner: 'test',
    surface: 'test_support',
  });
  const packages = [
    kernel(),
    support,
    workspacePackage(
      'hermes-contacts-runtime',
      { role: 'domain', owner: 'contacts', surface: 'runtime' },
      [dependency('hermes-test-support', 'build')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('production_test_dependency'));
});
