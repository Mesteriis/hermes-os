export function dependency(name, kind = null) {
  return {
    name,
    kind,
    path: `/workspace/${name}`,
    source: null,
    rename: null,
    optional: false,
  };
}

export function workspacePackage(name, hermes, dependencies = []) {
  return {
    id: `path+file:///workspace/${name}#0.1.0`,
    name,
    metadata: { hermes },
    dependencies,
  };
}

export function metadata(packages, externalPackages = []) {
  const workspaceByName = new Map(packages.map((pkg) => [pkg.name, pkg]));
  const resolvedExternalPackages = new Map(
    externalPackages.map((pkg) => [pkg.id, pkg]),
  );
  const externalByDeclaration = new Map();
  for (const pkg of packages) {
    for (const entry of pkg.dependencies.filter(({ source }) => source != null)) {
      const version = entry.req?.startsWith('=') ? entry.req.slice(1) : entry.req;
      const key = `${entry.source}:${entry.name}:${version}`;
      if (!externalByDeclaration.has(key)) {
        const externalPackage = {
          id: `${entry.source}#${entry.name}@${version}`,
          name: entry.name,
          version,
          source: entry.source,
        };
        externalByDeclaration.set(key, externalPackage);
        resolvedExternalPackages.set(externalPackage.id, externalPackage);
      }
    }
  }
  return {
    packages: [...packages, ...resolvedExternalPackages.values()],
    workspace_members: packages.map(({ id }) => id),
    resolve: {
      nodes: packages.map((pkg) => ({
        id: pkg.id,
        deps: pkg.dependencies.flatMap((entry) => {
          const version = entry.req?.startsWith('=') ? entry.req.slice(1) : entry.req;
          const external = externalByDeclaration.get(
            `${entry.source}:${entry.name}:${version}`,
          );
          const target = entry.source == null
            ? workspaceByName.get(entry.name)
            : external;
          if (!target) return [];
          return [{
            name: entry.rename ?? entry.name,
            pkg: target.id,
            dep_kinds: [{ kind: entry.kind, target: null }],
          }];
        }),
      })),
    },
  };
}

export function kernel(dependencies = [], metadataOverrides = {}) {
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
        'settings_registry',
      ],
      ...metadataOverrides,
    },
    dependencies,
  );
}

export function runtimeProtocol(dependencies = [], metadataOverrides = {}) {
  return workspacePackage(
    'hermes-runtime-protocol',
    {
      role: 'platform',
      owner: 'runtime_protocol',
      surface: 'contract',
      ...metadataOverrides,
    },
    dependencies,
  );
}

export function vaultProtocol(dependencies = [], metadataOverrides = {}) {
  return workspacePackage(
    'hermes-vault-protocol',
    {
      role: 'platform',
      owner: 'vault',
      surface: 'contract',
      ...metadataOverrides,
    },
    dependencies,
  );
}

export function vaultPackages({
  protocolDependencies = [],
  keyProviderDependencies = [],
  runtimeDependencies = [],
  overrides = {},
} = {}) {
  return [
    vaultProtocol(protocolDependencies, overrides.protocol),
    workspacePackage(
      'hermes-vault-key-provider',
      {
        role: 'platform',
        owner: 'vault',
        surface: 'contract',
        ...overrides.keyProvider,
      },
      keyProviderDependencies,
    ),
    workspacePackage(
      'hermes-vault-runtime',
      {
        role: 'platform',
        owner: 'vault',
        surface: 'runtime',
        components: ['vault_service'],
        ...overrides.runtime,
      },
      runtimeDependencies,
    ),
    workspacePackage('hermes-vault-store-sqlcipher', {
      role: 'platform',
      owner: 'vault',
      surface: 'persistence',
      ...overrides.store,
    }),
    workspacePackage('hermes-vault-key-provider-file', {
      role: 'platform',
      owner: 'vault',
      surface: 'implementation',
      ...overrides.keyProviderFile,
    }),
  ];
}

export function storageProtocol(dependencies = [], metadataOverrides = {}) {
  return workspacePackage(
    'hermes-storage-protocol',
    {
      role: 'platform',
      owner: 'storage',
      surface: 'contract',
      ...metadataOverrides,
    },
    dependencies,
  );
}

export function storagePackages({
  protocolDependencies = [],
  controlDependencies = [],
  runtimeDependencies = [],
  postgresDependencies = [],
  pgbouncerDependencies = [],
  migrationsDependencies = [],
  overrides = {},
} = {}) {
  return [
    storageProtocol(protocolDependencies, overrides.protocol),
    workspacePackage(
      'hermes-storage-control',
      {
        role: 'platform',
        owner: 'storage',
        surface: 'implementation',
        ...overrides.control,
      },
      controlDependencies,
    ),
    ...storageRuntimePackages({
      runtimeDependencies,
      postgresDependencies,
      pgbouncerDependencies,
      migrationsDependencies,
      overrides,
    }),
  ];
}

function storageRuntimePackages({
  runtimeDependencies,
  postgresDependencies,
  pgbouncerDependencies,
  migrationsDependencies,
  overrides,
}) {
  return [
    workspacePackage(
      'hermes-storage-runtime',
      {
        role: 'platform',
        owner: 'storage',
        surface: 'runtime',
        components: ['storage_control'],
        ...overrides.runtime,
      },
      runtimeDependencies,
    ),
    workspacePackage(
      'hermes-storage-postgres',
      {
        role: 'platform',
        owner: 'storage',
        surface: 'persistence',
        ...overrides.postgres,
      },
      postgresDependencies,
    ),
    workspacePackage(
      'hermes-storage-pgbouncer',
      {
        role: 'platform',
        owner: 'storage',
        surface: 'implementation',
        ...overrides.pgbouncer,
      },
      pgbouncerDependencies,
    ),
    workspacePackage(
      'hermes-storage-migrations',
      {
        role: 'platform',
        owner: 'storage',
        surface: 'implementation',
        ...overrides.migrations,
      },
      migrationsDependencies,
    ),
  ];
}

export function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}
