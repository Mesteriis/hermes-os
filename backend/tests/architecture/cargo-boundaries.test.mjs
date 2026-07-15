import assert from 'node:assert/strict';
import test from 'node:test';

import {
  validateCargoMetadata,
  validateWorkspaceManifestCoverage,
  validateWorkspacePackageRoots,
} from '../../scripts/lib/cargo-boundaries.mjs';
import { associateSqlWithWorkspace } from '../../scripts/lib/cargo-workspace.mjs';
import { canonicalPolicyForTests } from './support/canonical-policy.mjs';

function dependency(name, kind = null) {
  return { name, kind };
}

function workspacePackage(name, hermes, dependencies = []) {
  return {
    id: `path+file:///workspace/${name}#0.1.0`,
    name,
    metadata: { hermes },
    dependencies,
  };
}

function metadata(packages) {
  return {
    packages,
    workspace_members: packages.map(({ id }) => id),
  };
}

function kernel(dependencies = [], metadataOverrides = {}) {
  return workspacePackage(
    'hermes-kernel',
    {
      role: 'core',
      owner: 'kernel',
      surface: 'runtime',
      components: [
        'supervisor',
        'module_registry',
        'capability_router',
        'core_gateway',
        'event_hub',
        'telemetry_control',
      ],
      ...metadataOverrides,
    },
    dependencies,
  );
}

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('accepts a minimal clean-room workspace', () => {
  const packages = [
    kernel([dependency('hermes-events-contracts')]),
    workspacePackage('hermes-events-contracts', {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
    }),
    workspacePackage('hermes-contacts-contracts', {
      role: 'domain',
      owner: 'contacts',
      surface: 'contract',
    }),
    workspacePackage('hermes-ai-runtime', {
      role: 'domain',
      owner: 'ai',
      surface: 'runtime',
    }),
    workspacePackage('hermes-telemetry-collector', {
      role: 'platform',
      owner: 'telemetry',
      surface: 'runtime',
      components: ['telemetry_collector'],
    }),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('rejects a production Cargo manifest hidden outside workspace membership', () => {
  const violations = validateWorkspaceManifestCoverage(
    ['kernel/Cargo.toml', 'modules/tasks/Cargo.toml'],
    [],
    ['kernel/Cargo.toml'],
  );

  assert.equal(violations.length, 1);
  assert.equal(violations[0].code, 'orphan_cargo_manifest');
  assert.equal(violations[0].location, 'modules/tasks/Cargo.toml');
});

test('accepts production Cargo manifests registered in the workspace', () => {
  assert.deepEqual(
    validateWorkspaceManifestCoverage(
      ['kernel/Cargo.toml', 'modules/tasks/Cargo.toml'],
      [],
      ['kernel/Cargo.toml', 'modules/tasks/Cargo.toml'],
    ),
    [],
  );
});

test('rejects a workspace package outside configured production roots', () => {
  const violations = validateWorkspaceManifestCoverage(
    ['kernel/Cargo.toml'],
    [],
    ['kernel/Cargo.toml', 'experiments/hidden/Cargo.toml'],
  );

  assert.equal(violations.length, 1);
  assert.equal(violations[0].code, 'unscoped_workspace_package');
  assert.equal(violations[0].location, 'experiments/hidden/Cargo.toml');
});

test('accepts test-support manifests in the dedicated test-only workspace root', () => {
  assert.deepEqual(
    validateWorkspaceManifestCoverage(
      ['src/kernel/Cargo.toml'],
      ['tests/support/hermes-test-support/Cargo.toml'],
      ['src/kernel/Cargo.toml', 'tests/support/hermes-test-support/Cargo.toml'],
    ),
    [],
  );
});

test('rejects production roles in the test-only workspace root', () => {
  const violations = validateWorkspacePackageRoots(
    canonicalPolicyForTests(),
    [{
      manifest: 'tests/support/hermes-tasks/Cargo.toml',
      role: 'domain',
    }],
    new Set(),
    new Set(['tests/support/hermes-tasks/Cargo.toml']),
  );

  assert.ok(codes(violations).has('production_package_in_test_root'));
});

test('rejects test roles in the production workspace root', () => {
  const violations = validateWorkspacePackageRoots(
    canonicalPolicyForTests(),
    [{
      manifest: 'src/test-support/Cargo.toml',
      role: 'test',
    }],
    new Set(['src/test-support/Cargo.toml']),
    new Set(),
  );

  assert.ok(codes(violations).has('test_package_in_production_root'));
});

test('associates SQL with the owning workspace package metadata', () => {
  const tasks = workspacePackage('hermes-tasks-persistence', {
    role: 'domain',
    owner: 'tasks',
    surface: 'persistence',
  });
  tasks.manifest_path = '/workspace/modules/tasks/Cargo.toml';

  assert.deepEqual(
    associateSqlWithWorkspace(
      [{ path: 'modules/tasks/migrations/0001.sql', content: 'CREATE TABLE tasks_items ();' }],
      metadata([tasks]),
      '/workspace',
      'hermes',
    ),
    [{
      path: 'modules/tasks/migrations/0001.sql',
      content: 'CREATE TABLE tasks_items ();',
      packageName: 'hermes-tasks-persistence',
      owner: 'tasks',
      surface: 'persistence',
    }],
  );
});

test('allows an integration to publish the neutral Communications contract only', () => {
  const communicationsContract = workspacePackage('hermes-communications-contracts', {
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  });
  const allowed = [
    kernel(),
    communicationsContract,
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-communications-contracts')],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const tasksContract = workspacePackage('hermes-tasks-contracts', {
    role: 'domain',
    owner: 'tasks',
    surface: 'contract',
  });
  const forbidden = [
    kernel(),
    tasksContract,
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-tasks-contracts')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('forbidden_dependency'));
});

for (const owner of ['relationships', 'projects', 'obligations', 'decisions', 'knowledge', 'review']) {
  test(`rejects a Cargo package owned by blocked domain ${owner}`, () => {
    const packages = [
      kernel(),
      workspacePackage(`hermes-${owner}-runtime`, {
        role: 'domain',
        owner,
        surface: 'runtime',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_domain'));
  });
}

test('rejects a singular blocked domain hidden in metadata owner', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-generic-runtime', {
      role: 'integration',
      owner: 'project',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_domain'));
});

for (const provider of ['mail', 'telegram', 'whatsapp', 'zulip']) {
  test(`rejects provider ${provider} as a business domain`, () => {
    const packages = [
      kernel(),
      workspacePackage(`hermes-${provider}-runtime`, {
        role: 'domain',
        owner: provider,
        surface: 'runtime',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_domain'));
  });
}

test('prevents an integration from claiming an enabled business domain identity', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-telegram-runtime', {
      role: 'integration',
      owner: 'communications',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('invalid_owner'));
});

for (const owner of ['graph', 'timeline', 'search', 'context']) {
  test(`rejects a Cargo package for blocked projection ${owner}`, () => {
    const packages = [
      kernel(),
      workspacePackage(`hermes-${owner}-runtime`, {
        role: 'engine',
        owner,
        surface: 'runtime',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_projection'));
  });
}

test('rejects missing and unknown package roles', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-no-role', {
      owner: 'events',
      surface: 'contract',
    }),
    workspacePackage('hermes-many-roles', {
      role: ['platform', 'domain'],
      owner: 'events',
      surface: 'contract',
    }),
  ];

  const resultCodes = codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)));
  assert.ok(resultCodes.has('invalid_role'));
});

for (const kind of [null, 'build', 'dev']) {
  test(`rejects a direct ${kind ?? 'normal'} dependency between domains`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'hermes-tasks-runtime',
        { role: 'domain', owner: 'tasks', surface: 'runtime' },
        [dependency('hermes-contacts-contracts', kind)],
      ),
      workspacePackage('hermes-contacts-contracts', {
        role: 'domain',
        owner: 'contacts',
        surface: 'contract',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_dependency'));
  });
}

test('rejects the blocked projection role independently of its owner name', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-derived-reader', {
      role: 'projection',
      owner: 'derived_reader',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_projection'));
});

test('rejects singular aliases of blocked domains in package names', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-project-runtime', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('blocked_domain'));
});

test('allows a workflow to use contracts but not implementations', () => {
  const contactsContract = workspacePackage('hermes-contacts-contracts', {
    role: 'domain',
    owner: 'contacts',
    surface: 'contract',
  });
  const contactsRuntime = workspacePackage('hermes-contacts-runtime', {
    role: 'domain',
    owner: 'contacts',
    surface: 'runtime',
  });

  const allowed = [
    kernel(),
    contactsContract,
    workspacePackage(
      'hermes-contact-import-workflow',
      { role: 'workflow', owner: 'contact_import', surface: 'runtime' },
      [dependency('hermes-contacts-contracts')],
    ),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const forbidden = [
    kernel(),
    contactsRuntime,
    workspacePackage(
      'hermes-contact-import-workflow',
      { role: 'workflow', owner: 'contact_import', surface: 'runtime' },
      [dependency('hermes-contacts-runtime')],
    ),
  ];
  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('implementation_dependency'));
});

test('keeps a contract independent from its owner runtime and persistence', () => {
  for (const targetSurface of ['runtime', 'persistence']) {
    const targetName = `hermes-contacts-${targetSurface}`;
    const packages = [
      kernel(),
      workspacePackage(
        'hermes-contacts-contracts',
        { role: 'domain', owner: 'contacts', surface: 'contract' },
        [dependency(targetName)],
      ),
      workspacePackage(targetName, {
        role: 'domain',
        owner: 'contacts',
        surface: targetSurface,
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_dependency'));
  }
});

test('keeps domain implementation independent from persistence while runtime composes both', () => {
  const implementation = workspacePackage(
    'hermes-contacts-implementation',
    { role: 'domain', owner: 'contacts', surface: 'implementation' },
    [dependency('hermes-contacts-persistence')],
  );
  const persistence = workspacePackage('hermes-contacts-persistence', {
    role: 'domain',
    owner: 'contacts',
    surface: 'persistence',
  });
  const forbidden = [kernel(), implementation, persistence];
  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('forbidden_dependency'));

  const allowed = [
    kernel(),
    workspacePackage(
      'hermes-contacts-runtime',
      { role: 'domain', owner: 'contacts', surface: 'runtime' },
      [
        dependency('hermes-contacts-implementation'),
        dependency('hermes-contacts-persistence'),
      ],
    ),
    workspacePackage('hermes-contacts-implementation', {
      role: 'domain',
      owner: 'contacts',
      surface: 'implementation',
    }),
    persistence,
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);
});

for (const sqlClient of ['sqlx', 'tokio-postgres', 'postgres', 'diesel', 'sea-orm']) {
  test(`allows ${sqlClient} only in a persistence surface`, () => {
    const forbidden = [
      kernel(),
      workspacePackage(
        'hermes-contacts-runtime',
        { role: 'domain', owner: 'contacts', surface: 'runtime' },
        [dependency(sqlClient)],
      ),
    ];
    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('storage_dependency'));

    const allowed = [
      kernel(),
      workspacePackage(
        'hermes-contacts-persistence',
        { role: 'domain', owner: 'contacts', surface: 'persistence' },
        [dependency(sqlClient)],
      ),
    ];
    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);
  });
}

test('keeps Event Hub and telemetry control exclusive to Kernel', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-events-runtime', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
      components: ['event_hub'],
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('exclusive_kernel_component'));
});

test('rejects an Event Hub package outside Kernel even without component metadata', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-event-hub', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('exclusive_kernel_component'));
});

test('requires every Kernel component declared by policy', () => {
  const packages = [
    kernel([], {
      components: ['supervisor', 'module_registry', 'capability_router', 'core_gateway'],
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('missing_kernel_component'));
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
