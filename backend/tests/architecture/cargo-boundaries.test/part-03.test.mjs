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



for (const target of [
  { name: 'hermes-contacts-contracts', role: 'domain', owner: 'contacts' },
  { name: 'hermes-mail-analysis-contracts', role: 'workflow', owner: 'mail_analysis' },
  { name: 'hermes-telegram-contracts', role: 'integration', owner: 'telegram' },
]) {
  test(`prevents AI from acquiring cross-owner context through ${target.role} ${target.owner}`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'hermes-ai-runtime',
        { role: 'domain', owner: 'ai', surface: 'runtime' },
        [dependency(target.name)],
      ),
      workspacePackage(target.name, {
        role: target.role,
        owner: target.owner,
        surface: 'contract',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_dependency'));
  });
}



test('allows a use-case workflow to assemble AI context from explicit owner contracts', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-ai-contracts', {
      role: 'domain',
      owner: 'ai',
      surface: 'contract',
    }),
    workspacePackage('hermes-contacts-contracts', {
      role: 'domain',
      owner: 'contacts',
      surface: 'contract',
    }),
    workspacePackage(
      'hermes-contact-summary-workflow',
      { role: 'workflow', owner: 'contact_summary', surface: 'runtime' },
      [
        dependency('hermes-ai-contracts'),
        dependency('hermes-contacts-contracts'),
      ],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});



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

test('allows the Kernel runtime to compose only the exact Core Gateway adapters', () => {
  const gatewaySession = workspacePackage('hermes-gateway-session', {
    role: 'api',
    owner: 'gateway',
    surface: 'implementation',
  });
  const gatewayRuntime = workspacePackage('hermes-gateway-runtime', {
    role: 'api',
    owner: 'gateway',
    surface: 'implementation',
  });
  const packages = [
    kernel([dependency('hermes-gateway-session'), dependency('hermes-gateway-runtime')]),
    gatewaySession,
    gatewayRuntime,
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
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



for (const sqlClient of ['sqlx']) {
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



for (const alternativeClient of ['tokio-postgres', 'postgres', 'diesel', 'sea-orm']) {
  test(`rejects unselected PostgreSQL client ${alternativeClient} in owner persistence`, () => {
    const packages = [
      kernel(),
      workspacePackage(
        'hermes-contacts-persistence',
        { role: 'domain', owner: 'contacts', surface: 'persistence' },
        [dependency(alternativeClient)],
      ),
    ];

    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('storage_dependency'),
    );
  });
}



test('isolates the Kernel SQLite client in its persistence adapter', () => {
  const contract = workspacePackage('hermes-kernel-control-store', {
    role: 'core',
    owner: 'kernel',
    surface: 'contract',
  });
  const sqliteAdapter = workspacePackage(
    'hermes-kernel-control-store-sqlite',
    { role: 'core', owner: 'kernel', surface: 'persistence' },
    [dependency('hermes-kernel-control-store'), dependency('rusqlite')],
  );
  const allowed = [
    kernel([
      dependency('hermes-kernel-control-store'),
      dependency('hermes-kernel-control-store-sqlite'),
    ]),
    contract,
    sqliteAdapter,
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const directRuntimeDependency = [
    kernel([dependency('rusqlite')]),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(directRuntimeDependency)))
      .has('sqlite_dependency'),
  );

  const moduleBypass = [
    kernel(),
    sqliteAdapter,
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-kernel-control-store-sqlite')],
    ),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(moduleBypass)))
      .has('kernel_dependency'),
  );
});



test('rejects an unregistered core-owned package', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-kernel-unlisted-helper', {
      role: 'core',
      owner: 'kernel',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_core_package'),
  );
});



test('keeps Event Hub, telemetry control and settings registry exclusive to Kernel', () => {
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



test('keeps settings registry exclusive to Kernel', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-runtime-settings', {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'runtime',
      components: ['settings_registry'],
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('exclusive_kernel_component'));
});



test('rejects a settings registry package outside Kernel without component metadata', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-settings-registry', {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'runtime',
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

test('rejects Kernel components outside the constitutional registry', () => {
  const packages = [
    kernel([], {
      components: ['supervisor', 'unapproved_component'],
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('unknown_kernel_component'));
});
