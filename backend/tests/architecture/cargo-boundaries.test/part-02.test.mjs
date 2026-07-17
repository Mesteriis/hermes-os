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
