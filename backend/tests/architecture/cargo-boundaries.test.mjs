import assert from 'node:assert/strict';
import test from 'node:test';

import { validateCargoMetadata } from '../../scripts/lib/cargo-boundaries.mjs';
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
} from './support/cargo-fixtures.mjs';
import { canonicalPolicyForTests } from './support/canonical-policy.mjs';

function eventsProtocol(dependencies = [], metadataOverrides = {}) {
  return workspacePackage(
    'hermes-events-protocol',
    {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
      ...metadataOverrides,
    },
    dependencies,
  );
}

function metadata(packages) {
  const requiredProtocols = [eventsProtocol(), runtimeProtocol()]
    .filter((protocol) => !packages.some(({ name }) => name === protocol.name));
  return fixtureMetadata([...requiredProtocols, ...packages]);
}

test('requires the canonical events protocol package when a workspace exists', () => {
  const violations = validateCargoMetadata(
    canonicalPolicyForTests(),
    fixtureMetadata([kernel(), runtimeProtocol()]),
  );

  assert.ok(codes(violations).has('missing_events_protocol_package'));
});

test('requires the canonical runtime protocol package when a workspace exists', () => {
  const violations = validateCargoMetadata(
    canonicalPolicyForTests(),
    fixtureMetadata([kernel(), eventsProtocol()]),
  );

  assert.ok(codes(violations).has('missing_runtime_protocol_package'));
});

test('accepts only the exact canonical runtime protocol metadata', () => {
  const mutations = [
    { role: 'core' },
    { owner: 'events' },
    { surface: 'implementation' },
    { components: ['module_registry'] },
  ];

  for (const metadataOverrides of mutations) {
    const packages = [kernel(), runtimeProtocol([], metadataOverrides)];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_runtime_protocol_package'),
    );
  }
});

test('allows only one package to claim the canonical runtime protocol owner', () => {
  const packages = [
    kernel(),
    runtimeProtocol(),
    workspacePackage('hermes-runtime-protocol-alias', {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('runtime_protocol_owner'),
  );
});

for (const kind of [null, 'build', 'dev']) {
  for (const forbiddenDependency of [
    'async-nats',
    'nats',
    'sqlx',
    'tokio-postgres',
    'postgres',
    'diesel',
    'sea-orm',
    'rusqlite',
    'serde_json',
  ]) {
    test(`keeps runtime protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const packages = [
        kernel(),
        runtimeProtocol([dependency(forbiddenDependency, kind)]),
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('runtime_protocol_dependency'),
      );
    });

  }
}

test('allows protobuf-only dependencies in the canonical runtime protocol', () => {
  const packages = [
    kernel(),
    runtimeProtocol([dependency('prost'), dependency('bytes')]),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('requires the complete canonical Vault package set once Vault ownership appears', () => {
  const packages = [kernel(), vaultProtocol()];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('missing_vault_package'),
  );
});

test('rejects a dependency-only reference to an undeclared Vault package', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-vault-protocol')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('missing_vault_package'),
  );
});

test('accepts the exact canonical Vault package set', () => {
  const packages = [kernel(), ...vaultPackages()];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('rejects incorrect metadata on every canonical Vault package surface', () => {
  const mutations = [
    { protocol: { surface: 'implementation' } },
    { keyProvider: { components: ['vault_service'] } },
    { runtime: { components: [] } },
    { store: { surface: 'implementation' } },
    { keychainMacos: { owner: 'storage' } },
  ];

  for (const overrides of mutations) {
    const packages = [kernel(), ...vaultPackages({ overrides })];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_vault_package'),
    );
  }
});

test('rejects undeclared packages claiming the Vault owner', () => {
  const packages = [
    kernel(),
    ...vaultPackages(),
    workspacePackage('hermes-vault-compat', {
      role: 'platform',
      owner: 'vault',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_vault_package'),
  );
});

test('keeps the Vault service component exclusive to the canonical Vault runtime', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-telemetry-vault-helper', {
      role: 'platform',
      owner: 'telemetry',
      surface: 'runtime',
      components: ['vault_service'],
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_vault_package'),
  );
});

for (const kind of [null, 'build', 'dev']) {
  for (const forbiddenDependency of [
    'async-nats',
    'nats',
    'sqlx',
    'tokio-postgres',
    'postgres',
    'diesel',
    'sea-orm',
    'rusqlite',
    'serde_json',
  ]) {
    test(`keeps Vault protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const [, ...remainingVaultPackages] = vaultPackages();
      const packages = [
        kernel(),
        vaultProtocol([dependency(forbiddenDependency, kind)]),
        ...remainingVaultPackages,
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('vault_protocol_dependency'),
      );
    });

    test(`keeps the private Vault key-provider contract independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const packages = [
        kernel(),
        ...vaultPackages({
          keyProviderDependencies: [dependency(forbiddenDependency, kind)],
        }),
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('vault_protocol_dependency'),
      );
    });
  }
}

test('rejects undeclared external hermes-vault package dependencies', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-vault-compat')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_private_dependency'),
  );
});

test('allows modules to use only the public Vault protocol', () => {
  const allowed = [
    kernel(),
    ...vaultPackages(),
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-vault-protocol')],
    ),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const forbidden = [
    kernel(),
    ...vaultPackages(),
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-vault-key-provider')],
    ),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('vault_private_dependency'),
  );
});

test('prevents Kernel from linking a private Vault package', () => {
  const packages = [
    kernel([dependency('hermes-vault-runtime')]),
    ...vaultPackages(),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_private_dependency'),
  );
});

test('prevents Vault packages from depending on Kernel or module packages', () => {
  const telegramApi = workspacePackage('hermes-telegram-api', {
    role: 'integration',
    owner: 'telegram',
    surface: 'contract',
  });
  const packages = [
    kernel(),
    telegramApi,
    ...vaultPackages({ runtimeDependencies: [dependency('hermes-telegram-api')] }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_owner_dependency'),
  );
});

test('keeps every Vault package independent of NATS and PostgreSQL clients', () => {
  const packages = [
    kernel(),
    ...vaultPackages({ runtimeDependencies: [dependency('async-nats')] }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('vault_owner_dependency'),
  );
});

test('requires the complete canonical Storage Control package set once storage ownership appears', () => {
  const packages = [kernel(), storageProtocol()];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('missing_storage_package'),
  );
});

test('accepts the exact canonical Storage Control package set', () => {
  const packages = [kernel(), ...storagePackages()];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('accepts the intended isolated Storage Control dependency graph', () => {
  const packages = [
    kernel(),
    ...storagePackages({
      controlDependencies: [dependency('hermes-storage-protocol')],
      runtimeDependencies: [
        dependency('hermes-storage-protocol'),
        dependency('hermes-storage-control'),
        dependency('hermes-storage-postgres'),
        dependency('hermes-storage-pgbouncer'),
        dependency('hermes-storage-migrations'),
      ],
      postgresDependencies: [dependency('hermes-storage-control'), dependency('sqlx')],
      pgbouncerDependencies: [dependency('hermes-storage-control')],
      migrationsDependencies: [dependency('hermes-storage-control'), dependency('pg_query')],
    }),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('rejects incorrect metadata on every canonical Storage Control package', () => {
  const mutations = [
    { protocol: { surface: 'implementation' } },
    { control: { surface: 'runtime' } },
    { runtime: { components: [] } },
    { postgres: { surface: 'implementation' } },
    { pgbouncer: { owner: 'events' } },
    { migrations: { surface: 'persistence' } },
  ];

  for (const overrides of mutations) {
    const packages = [kernel(), ...storagePackages({ overrides })];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_storage_package'),
    );
  }
});

test('rejects undeclared packages claiming the Storage owner', () => {
  const packages = [
    kernel(),
    ...storagePackages(),
    workspacePackage('hermes-storage-compat', {
      role: 'platform',
      owner: 'storage',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_storage_package'),
  );
});

test('keeps storage_control exclusive to the canonical Storage runtime', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-storage-helper', {
      role: 'platform',
      owner: 'events',
      surface: 'runtime',
      components: ['storage_control'],
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('invalid_storage_package'),
  );
});

for (const kind of [null, 'build', 'dev']) {
  for (const forbiddenDependency of [
    'async-nats',
    'nats',
    'sqlx',
    'tokio-postgres',
    'postgres',
    'diesel',
    'sea-orm',
    'rusqlite',
    'serde_json',
  ]) {
    test(`keeps storage protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const [, ...remainingStoragePackages] = storagePackages();
      const packages = [
        kernel(),
        storageProtocol([dependency(forbiddenDependency, kind)]),
        ...remainingStoragePackages,
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('storage_protocol_dependency'),
      );
    });
  }
}

test('allows PostgreSQL and AST clients only in their exact Storage packages', () => {
  const allowed = [
    kernel(),
    ...storagePackages({
      postgresDependencies: [dependency('sqlx')],
      migrationsDependencies: [dependency('pg_query')],
    }),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const sqlInControl = [
    kernel(),
    ...storagePackages({ controlDependencies: [dependency('sqlx')] }),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(sqlInControl)))
      .has('storage_dependency'),
  );

  const astInRuntime = [
    kernel(),
    ...storagePackages({ runtimeDependencies: [dependency('pg_query')] }),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(astInRuntime)))
      .has('storage_ast_dependency'),
  );
});

test('allows production packages outside Storage to use only storage protocol', () => {
  const allowed = [
    kernel([dependency('hermes-storage-protocol')]),
    ...storagePackages(),
  ];
  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const forbidden = [
    kernel([dependency('hermes-storage-postgres')]),
    ...storagePackages(),
  ];
  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden)))
      .has('storage_private_dependency'),
  );
});

test('prevents Storage packages from depending on Kernel, Gateway or modules', () => {
  const contacts = workspacePackage('hermes-contacts-api', {
    role: 'domain',
    owner: 'contacts',
    surface: 'contract',
  });
  const packages = [
    kernel(),
    contacts,
    ...storagePackages({ controlDependencies: [dependency('hermes-contacts-api')] }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('storage_owner_dependency'),
  );
});

test('rejects SQLite clients in owner PostgreSQL persistence packages', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-contacts-persistence',
      { role: 'domain', owner: 'contacts', surface: 'persistence' },
      [dependency('rusqlite')],
    ),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('sqlite_dependency'),
  );
});

test('accepts only the exact canonical events protocol metadata', () => {
  const mutations = [
    { role: 'core' },
    { owner: 'telemetry' },
    { surface: 'implementation' },
    { components: ['event_hub'] },
  ];

  for (const metadataOverrides of mutations) {
    const packages = [kernel(), eventsProtocol([], metadataOverrides)];
    assert.ok(
      codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
        .has('invalid_events_protocol_package'),
    );
  }
});

test('allows only one package to claim the canonical events protocol owner', () => {
  const packages = [
    kernel(),
    eventsProtocol(),
    workspacePackage('hermes-events-protocol-alias', {
      role: 'platform',
      owner: 'events',
      surface: 'contract',
    }),
  ];

  assert.ok(
    codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
      .has('events_protocol_owner'),
  );
});

for (const kind of [null, 'build', 'dev']) {
  for (const forbiddenDependency of [
    'async-nats',
    'nats',
    'sqlx',
    'tokio-postgres',
    'postgres',
    'diesel',
    'sea-orm',
    'rusqlite',
    'serde_json',
  ]) {
    test(`keeps events protocol independent of ${forbiddenDependency} through ${kind ?? 'normal'} dependencies`, () => {
      const packages = [
        kernel(),
        eventsProtocol([dependency(forbiddenDependency, kind)]),
      ];

      assert.ok(
        codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)))
          .has('events_protocol_dependency'),
      );
    });
  }
}

test('allows protobuf-only dependencies in the canonical events protocol', () => {
  const packages = [
    kernel(),
    eventsProtocol([dependency('prost'), dependency('bytes')]),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('allows an integration to publish only through Communications ingress', () => {
  const communicationsContract = workspacePackage('hermes-communications-ingress', {
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
      [dependency('hermes-communications-ingress')],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(allowed)), []);

  const clientContract = workspacePackage('hermes-communications-api', {
    role: 'domain',
    owner: 'communications',
    surface: 'contract',
  });
  const forbidden = [
    kernel(),
    clientContract,
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-communications-api')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(forbidden))).has('integration_domain_contract_dependency'));
});

for (const packageName of [
  'hermes-hub-backend',
  'hermes-api',
  'hermes-worker-runtime',
  'hermes-desktop-runtime',
  'hermes-schema',
  'hermes-common',
  'hermes-provider-api',
]) {
  test(`rejects compile-graph aggregation package ${packageName}`, () => {
    const packages = [
      kernel(),
      workspacePackage(packageName, {
        role: 'platform',
        owner: 'runtime_protocol',
        surface: 'contract',
      }),
    ];

    assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('forbidden_aggregate_package'));
  });
}

test('prevents module packages from depending on Kernel implementation', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-telegram-core',
      { role: 'integration', owner: 'telegram', surface: 'implementation' },
      [dependency('hermes-kernel')],
    ),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('kernel_dependency'));
});

test('prevents Kernel from compiling owner-specific module contracts', () => {
  const packages = [
    kernel([dependency('hermes-contacts-contracts')]),
    workspacePackage('hermes-contacts-contracts', {
      role: 'domain',
      owner: 'contacts',
      surface: 'contract',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('kernel_module_dependency'));
});

test('keeps Gateway protocol independent from owner-specific contracts', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-gateway-protocol',
      { role: 'api', owner: 'gateway', surface: 'contract' },
      [dependency('hermes-contacts-contracts')],
    ),
    workspacePackage('hermes-contacts-contracts', {
      role: 'domain',
      owner: 'contacts',
      surface: 'contract',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('gateway_module_dependency'));
});

test('prevents one runtime package from aggregating another runtime', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-telegram-runtime',
      { role: 'integration', owner: 'telegram', surface: 'runtime' },
      [dependency('hermes-telegram-sync-runtime')],
    ),
    workspacePackage('hermes-telegram-sync-runtime', {
      role: 'integration',
      owner: 'telegram',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('runtime_aggregation_dependency'));
});

test('rejects persistence adapter dependencies across owners', () => {
  const packages = [
    kernel(),
    workspacePackage(
      'hermes-tasks-persistence',
      { role: 'domain', owner: 'tasks', surface: 'persistence' },
      [dependency('hermes-contacts-persistence')],
    ),
    workspacePackage('hermes-contacts-persistence', {
      role: 'domain',
      owner: 'contacts',
      surface: 'persistence',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('cross_owner_persistence_dependency'));
});

for (const { owner, adapters } of [
  { owner: 'mail', adapters: ['imap', 'smtp'] },
  { owner: 'telegram', adapters: ['tdlib'] },
  { owner: 'zulip', adapters: ['http'] },
]) {
  test(`accepts an isolated ${owner} package graph without a Communications implementation dependency`, () => {
    const adapterPackages = adapters.map((adapter) => workspacePackage(
      `hermes-${owner}-${adapter}`,
      { role: 'integration', owner, surface: 'implementation' },
      [dependency(`hermes-${owner}-core`)],
    ));
    const packages = [
      kernel(),
      workspacePackage('hermes-communications-ingress', {
        role: 'domain',
        owner: 'communications',
        surface: 'contract',
      }),
      workspacePackage(`hermes-${owner}-api`, {
        role: 'integration',
        owner,
        surface: 'contract',
      }),
      workspacePackage(
        `hermes-${owner}-core`,
        { role: 'integration', owner, surface: 'implementation' },
        [dependency('hermes-communications-ingress')],
      ),
      ...adapterPackages,
      workspacePackage(
        `hermes-${owner}-persistence`,
        { role: 'integration', owner, surface: 'persistence' },
        [dependency(`hermes-${owner}-core`)],
      ),
      workspacePackage(
        `hermes-${owner}-runtime`,
        { role: 'integration', owner, surface: 'runtime' },
        [
          dependency(`hermes-${owner}-api`),
          dependency(`hermes-${owner}-core`),
          ...adapters.map((adapter) => dependency(`hermes-${owner}-${adapter}`)),
          dependency(`hermes-${owner}-persistence`),
        ],
      ),
    ];

    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
  });
}

for (const owner of ['contacts', 'organizations', 'tasks', 'calendar', 'documents', 'ai']) {
  test(`accepts an isolated package graph for enabled domain ${owner}`, () => {
    const packages = [
      kernel(),
      workspacePackage(`hermes-${owner}-contracts`, {
        role: 'domain',
        owner,
        surface: 'contract',
      }),
      workspacePackage(
        `hermes-${owner}-domain`,
        { role: 'domain', owner, surface: 'implementation' },
        [dependency(`hermes-${owner}-contracts`)],
      ),
      workspacePackage(
        `hermes-${owner}-persistence`,
        { role: 'domain', owner, surface: 'persistence' },
        [dependency(`hermes-${owner}-domain`)],
      ),
      workspacePackage(
        `hermes-${owner}-runtime`,
        { role: 'domain', owner, surface: 'runtime' },
        [
          dependency(`hermes-${owner}-contracts`),
          dependency(`hermes-${owner}-domain`),
          dependency(`hermes-${owner}-persistence`),
        ],
      ),
    ];

    assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
  });
}

test('accepts the split Communications ingress and client API package graph', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-communications-ingress', {
      role: 'domain',
      owner: 'communications',
      surface: 'contract',
    }),
    workspacePackage('hermes-communications-api', {
      role: 'domain',
      owner: 'communications',
      surface: 'contract',
    }),
    workspacePackage(
      'hermes-communications-domain',
      { role: 'domain', owner: 'communications', surface: 'implementation' },
      [
        dependency('hermes-communications-ingress'),
        dependency('hermes-communications-api'),
      ],
    ),
    workspacePackage(
      'hermes-communications-persistence',
      { role: 'domain', owner: 'communications', surface: 'persistence' },
      [dependency('hermes-communications-domain')],
    ),
    workspacePackage(
      'hermes-communications-runtime',
      { role: 'domain', owner: 'communications', surface: 'runtime' },
      [
        dependency('hermes-communications-ingress'),
        dependency('hermes-communications-api'),
        dependency('hermes-communications-domain'),
        dependency('hermes-communications-persistence'),
      ],
    ),
  ];

  assert.deepEqual(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages)), []);
});

test('keeps WhatsApp implementation in the hidden host WebView boundary', () => {
  const packages = [
    kernel(),
    workspacePackage('hermes-whatsapp-runtime', {
      role: 'integration',
      owner: 'whatsapp',
      surface: 'runtime',
    }),
  ];

  assert.ok(codes(validateCargoMetadata(canonicalPolicyForTests(), metadata(packages))).has('host_only_integration'));
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
