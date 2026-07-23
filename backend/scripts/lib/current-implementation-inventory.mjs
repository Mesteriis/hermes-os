import { list, violation } from './validation-diagnostics.mjs';

const CRATES_IO_REGISTRY_SOURCE =
  'registry+https://github.com/rust-lang/crates.io-index';

function workspacePackages(cargoMetadata) {
  const workspaceIds = new Set(list(cargoMetadata?.workspace_members));
  return list(cargoMetadata?.packages).filter(
    (pkg) => workspaceIds.size === 0 || workspaceIds.has(pkg.id),
  );
}

function exactMetadataKeys(metadata, expectedKeys) {
  if (!metadata || typeof metadata !== 'object' || Array.isArray(metadata)) return false;
  const keys = Object.keys(metadata);
  return keys.length === expectedKeys.length
    && keys.every((key) => expectedKeys.includes(key));
}

function exactOrderedList(actual, expected) {
  return Array.isArray(actual)
    && actual.length === expected.length
    && actual.every((entry, index) => entry === expected[index]);
}

function dependencyKind(dependency) {
  return dependency?.kind ?? 'normal';
}

function dependencyKey(dependency) {
  return `${dependencyKind(dependency)}:${dependency?.name}`;
}

function resolvedDependencyKey(kind, packageId) {
  return `${kind ?? 'normal'}:${packageId}`;
}

function sameStringSet(actual, expected) {
  const actualValues = [...list(actual)].sort();
  const expectedValues = [...list(expected)].sort();
  return exactOrderedList(actualValues, expectedValues);
}

function sourceMatches(dependency, expectedSource) {
  if (expectedSource !== 'crates_io') return false;
  return dependency?.source === CRATES_IO_REGISTRY_SOURCE;
}

function resolvesToReviewedPackage(resolveNode, packagesById, specification) {
  const exactVersion = specification.version.startsWith('=')
    ? specification.version.slice(1)
    : null;
  if (exactVersion === null) return false;

  return list(resolveNode?.deps).some((edge) => {
    const resolvedPackage = packagesById.get(edge?.pkg);
    const kindMatches = list(edge?.dep_kinds).some(
      ({ kind }) => (kind ?? 'normal') === specification.kind,
    );
    return kindMatches
      && resolvedPackage?.name === specification.name
      && resolvedPackage?.version === exactVersion
      && sourceMatches(resolvedPackage, specification.source);
  });
}

export function validateCurrentImplementationInventory(policy, cargoMetadata) {
  const metadataKey = policy?.cargo?.metadataKey;
  const packages = workspacePackages(cargoMetadata);
  const productionPackages = packages.filter(
    (pkg) => ![
      policy?.owners?.test,
      policy?.owners?.development,
    ].includes(pkg?.metadata?.[metadataKey]?.role),
  );

  if (productionPackages.length === 0) return [];

  const expectedPackages = list(policy?.implementation?.productionPackages);
  const expectedByName = new Map(expectedPackages.map((entry) => [entry.name, entry]));
  const workspaceByName = new Map(packages.map((pkg) => [pkg.name, pkg]));
  const workspaceById = new Map(packages.map((pkg) => [pkg.id, pkg]));
  const packagesById = new Map(
    list(cargoMetadata?.packages).map((pkg) => [pkg.id, pkg]),
  );
  const resolveById = new Map(
    list(cargoMetadata?.resolve?.nodes).map((node) => [node.id, node]),
  );
  const inactiveIntegrationPackages = productionPackages.filter((pkg) => (
    pkg?.metadata?.[metadataKey]?.role === 'integration'
    && !expectedByName.has(pkg.name)
  ));
  const activePackages = productionPackages.filter(
    (pkg) => !inactiveIntegrationPackages.includes(pkg),
  );
  const actualByName = new Map(activePackages.map((pkg) => [pkg.name, pkg]));
  const violations = [];

  const developmentPackages = packages.filter(
    (pkg) => pkg?.metadata?.[metadataKey]?.role === policy?.owners?.development,
  );
  const developmentProfile = policy?.implementation?.developmentProfile;
  if (developmentPackages.length > 1) {
    violations.push(violation(
      'development_runtime_inventory',
      'cargo:workspace',
      'only one explicit development platform runtime package is allowed',
    ));
  }
  for (const pkg of developmentPackages) {
    const descriptor = pkg?.metadata?.[metadataKey];
    const metadataMatches = exactMetadataKeys(descriptor, ['role', 'owner', 'surface', 'components'])
      && pkg.name === developmentProfile.package
      && descriptor.role === 'development'
      && descriptor.owner === policy.owners.development
      && descriptor.surface === 'runtime'
      && exactOrderedList(descriptor.components, []);
    if (!metadataMatches) {
      violations.push(violation(
        'development_runtime_inventory',
        `cargo:${pkg.name}`,
        'development operator must exactly match the explicit simulated-platform package descriptor',
      ));
    }
  }

  const unexpected = [...actualByName.keys()].filter((name) => !expectedByName.has(name));
  const missing = [...expectedByName.keys()].filter((name) => !actualByName.has(name));
  if (unexpected.length > 0 || missing.length > 0
    || actualByName.size !== activePackages.length) {
    violations.push(violation(
      'implementation_inventory',
      'cargo:workspace',
      `active production packages must exactly match ${policy.implementation.currentSlice}; missing=${missing.join(',') || '-'} unexpected=${unexpected.join(',') || '-'}`,
    ));
  }

  for (const [name, expected] of expectedByName) {
    const pkg = actualByName.get(name);
    if (!pkg) continue;
    const descriptor = pkg?.metadata?.[metadataKey];
    const isKernelRuntime = name === policy.kernel.package;
    const isVaultRuntime = name === policy.vault.runtimePackage;
    const isTelemetryRuntime = descriptor?.components?.includes(policy.telemetry.collectorComponent);
    const isStorageRuntime = name === policy.storage.runtimePackage;
    const expectedKeys = isKernelRuntime || isVaultRuntime || isTelemetryRuntime || isStorageRuntime
      ? ['role', 'owner', 'surface', 'components']
      : ['role', 'owner', 'surface'];
    const metadataMatches = exactMetadataKeys(descriptor, expectedKeys)
      && descriptor.role === expected.role
      && descriptor.owner === expected.owner
      && descriptor.surface === expected.surface
      && (isKernelRuntime
        ? exactOrderedList(
          descriptor.components,
          policy.implementation.kernelProfile.activeComponents,
        )
        : isVaultRuntime
          ? exactOrderedList(descriptor.components, [policy.vault.runtimeComponent])
          : isTelemetryRuntime
            ? exactOrderedList(descriptor.components, [policy.telemetry.collectorComponent])
            : isStorageRuntime
              ? exactOrderedList(descriptor.components, [policy.storage.runtimeComponent])
            : list(descriptor.components).length === 0);

    if (!metadataMatches) {
      violations.push(violation(
        'implementation_inventory',
        `cargo:${name}`,
        `${name} metadata must exactly match the authorized ${policy.implementation.currentSlice} package descriptor`,
      ));
    }

    if (policy.implementation.cargoFeaturesEnabled === false
      && Object.keys(pkg.features ?? {}).length > 0) {
      violations.push(violation(
        'implementation_features',
        `cargo:${name}`,
        'Cargo features cannot hide or activate capabilities in the recovery-only slice',
      ));
    }

    const targetPolicy = policy.implementation.targetPolicy[name];
    const targetKinds = list(pkg.targets).flatMap((target) => list(target?.kind));
    const primaryTargets = targetKinds.filter((kind) => kind !== 'custom-build');
    const customBuildTargets = targetKinds.filter((kind) => kind === 'custom-build');
    const runtimeTargetsMatch = targetPolicy.primaryKind === 'bin'
      && primaryTargets.includes('bin')
      && primaryTargets.every((kind) => kind === 'bin' || kind === 'lib')
      && primaryTargets.filter((kind) => kind === 'bin').length === 1
      && primaryTargets.filter((kind) => kind === 'lib').length <= 1;
    const targetsMatch = (runtimeTargetsMatch || (
      primaryTargets.length === 1
      && primaryTargets[0] === targetPolicy.primaryKind
    ))
      && customBuildTargets.length <= (targetPolicy.customBuildAllowed ? 1 : 0)
      && targetKinds.length === primaryTargets.length + customBuildTargets.length;
    if (!targetsMatch) {
      violations.push(violation(
        'implementation_target',
        `cargo:${name}`,
        `${name} must expose its declared target surface${targetPolicy.customBuildAllowed ? ' and at most one codegen build target' : ' without a build target'}`,
      ));
    }

    const requiredWorkspaceDependencies = policy.implementation
      .workspaceDependencyAllowlist[name] ?? [];
    const requiredWorkspaceKeys = new Set(requiredWorkspaceDependencies.map((dependency) => {
      const target = workspaceByName.get(dependency.name);
      return resolvedDependencyKey(dependency.kind, target?.id);
    }));
    const resolveNode = resolveById.get(pkg.id);
    if (!resolveNode) {
      violations.push(violation(
        'implementation_dependency_resolution',
        `cargo:${name}`,
        'full Cargo resolve metadata is required to verify current-slice package-ID edges',
      ));
    }
    const observedWorkspaceKeys = new Set(
      list(resolveNode?.deps).flatMap((edge) => {
        if (!workspaceById.has(edge?.pkg)) return [];
        return list(edge?.dep_kinds).map(({ kind }) => resolvedDependencyKey(kind, edge.pkg));
      }),
    );
    const allowedThirdPartyDependencies = policy.implementation
      .thirdPartyDependencyAllowlist[name] ?? [];
    const observedThirdPartyKeys = new Set();
    const invalidResolvedThirdPartyKeys = new Set();
    for (const dependency of list(pkg.dependencies)) {
      const kind = dependencyKind(dependency);
      const workspaceTarget = dependency?.source == null
        ? workspaceByName.get(dependency.name)
        : null;
      const targetRole = workspaceTarget?.metadata?.[metadataKey]?.role;
      const allowedTestDependency = targetRole === policy.owners.test && kind === 'dev';
      if (allowedTestDependency) continue;

      if (workspaceTarget) {
        const resolvedKey = resolvedDependencyKey(kind, workspaceTarget.id);
        const declarationIsLocal = dependency.source == null
          && typeof dependency.path === 'string'
          && dependency.rename == null
          && dependency.optional !== true;
        if (declarationIsLocal && requiredWorkspaceKeys.has(resolvedKey)) continue;
        violations.push(violation(
          'implementation_dependency',
          `cargo:${name}:${kind}:${dependency.name}`,
          `${dependency.name} is outside the recovery-only workspace dependency allowlist for ${name}`,
        ));
        continue;
      }

      const reviewedThirdParty = allowedThirdPartyDependencies.find((allowed) => (
        allowed.name === dependency.name
        && allowed.kind === kind
        && sourceMatches(dependency, allowed.source)
        && dependency.req === allowed.version
        && dependency.uses_default_features === allowed.defaultFeatures
        && sameStringSet(dependency.features, allowed.features)
        && dependency.rename == null
        && dependency.optional !== true
      ));
      if (!reviewedThirdParty) {
        violations.push(violation(
          'implementation_dependency',
          `cargo:${name}:${kind}:${dependency.name}`,
          `${dependency.name} is outside the exact reviewed third-party dependency allowlist for ${name}`,
        ));
      } else if (!resolvesToReviewedPackage(
        resolveNode,
        packagesById,
        reviewedThirdParty,
      )) {
        invalidResolvedThirdPartyKeys.add(dependencyKey(reviewedThirdParty));
        violations.push(violation(
          'implementation_dependency',
          `cargo:${name}:${kind}:${dependency.name}`,
          `${dependency.name} must resolve to its exact reviewed crates.io package and version`,
        ));
      } else {
        observedThirdPartyKeys.add(dependencyKey(dependency));
      }
    }

    for (const requiredDependency of requiredWorkspaceDependencies) {
      const target = workspaceByName.get(requiredDependency.name);
      const key = resolvedDependencyKey(requiredDependency.kind, target?.id);
      if (!observedWorkspaceKeys.has(key)) {
        violations.push(violation(
          'implementation_dependency',
          `cargo:${name}:${requiredDependency.kind}:${requiredDependency.name}`,
          `${name} must depend on ${requiredDependency.name} through an exact ${requiredDependency.kind} edge`,
        ));
      }
    }
    for (const requiredDependency of allowedThirdPartyDependencies) {
      const key = dependencyKey(requiredDependency);
      if (!observedThirdPartyKeys.has(key)
        && !invalidResolvedThirdPartyKeys.has(key)) {
        violations.push(violation(
          'implementation_dependency',
          `cargo:${name}:${requiredDependency.kind}:${requiredDependency.name}`,
          `${name} must use the exact reviewed ${requiredDependency.name} dependency profile`,
        ));
      }
    }
  }

  return violations;
}

function pathBelongsToRoot(path, root) {
  return root !== '' && (path === root || path.startsWith(`${root}/`));
}

export function validateCurrentImplementationSourceCoverage(
  policy,
  sourceEntries,
  packageRoots,
) {
  const expectedNames = new Set(
    list(policy?.implementation?.productionPackages).map(({ name }) => name),
  );
  const productionRoots = list(packageRoots).filter(({ name, role, root }) => (
    role !== policy?.owners?.test
    && (expectedNames.has(name) || role === 'integration')
    && typeof root === 'string'
    && root !== ''
  ));

  return list(sourceEntries)
    .filter((entry) => entry?.isDirectory !== true && entry?.isSymbolicLink !== true)
    .filter(({ path }) => !productionRoots.some(({ root }) => pathBelongsToRoot(path, root)))
    .map(({ path }) => violation(
      'implementation_source_coverage',
      path,
      'every production source file must belong to an authorized current-slice or registered integration Cargo package root',
    ));
}
