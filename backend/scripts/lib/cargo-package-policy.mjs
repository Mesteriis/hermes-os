import {
  duplicates,
  list,
  ownerAliases,
  pathTokens,
  violation,
} from './validation-diagnostics.mjs';

const HERMES_METADATA_KEYS = new Set(['role', 'owner', 'surface', 'components']);

function metadataDescriptor(policy, pkg, violations) {
  const metadataKey = policy.cargo.metadataKey;
  const metadata = pkg?.metadata?.[metadataKey];
  const location = `cargo:${pkg?.name ?? '<unknown>'}`;

  if (!metadata || typeof metadata !== 'object' || Array.isArray(metadata)) {
    violations.push(violation('missing_metadata', location, `missing [package.metadata.${metadataKey}]`));
    return null;
  }

  for (const key of Object.keys(metadata)) {
    if (!HERMES_METADATA_KEYS.has(key)) {
      violations.push(violation('unknown_metadata_key', location, `unknown Hermes metadata key: ${key}`));
    }
  }

  const { role, owner, surface } = metadata;
  const components = metadata.components ?? [];

  if (typeof role !== 'string' || !policy.cargo.roles.includes(role)) {
    const blockedRole = typeof role === 'string' && policy.projections.blockedRoles.includes(role);
    violations.push(violation(
      blockedRole ? 'blocked_projection' : 'invalid_role',
      location,
      `role must be exactly one of: ${policy.cargo.roles.join(', ')}`,
    ));
  }
  if (typeof owner !== 'string' || owner.length === 0) {
    violations.push(violation('invalid_owner', location, 'owner must be a non-empty string'));
  }
  if (typeof surface !== 'string' || !policy.cargo.surfaces.includes(surface)) {
    violations.push(violation(
      'invalid_surface',
      location,
      `surface must be one of: ${policy.cargo.surfaces.join(', ')}`,
    ));
  }
  if (!Array.isArray(components) || components.some((component) => typeof component !== 'string')) {
    violations.push(violation('invalid_components', location, 'components must be an array of strings'));
  } else if (duplicates(components).length > 0) {
    violations.push(violation('duplicate_component', location, 'components must be unique'));
  }

  if (typeof role !== 'string' || typeof owner !== 'string' || typeof surface !== 'string') return null;
  return { role, owner, surface, components: Array.isArray(components) ? components : [] };
}

function validateDescriptor(policy, pkg, descriptor, violations) {
  if (!descriptor) return;
  const location = `cargo:${pkg.name}`;
  const { role, owner, surface, components } = descriptor;
  const blockedDomains = new Set(policy.domains.blocked);
  const blockedProjections = new Set(policy.projections.blockedOwners);
  const ownerTokens = new Set(pathTokens(owner));

  for (const blocked of blockedDomains) {
    if ([...ownerAliases(blocked)].some((alias) => ownerTokens.has(alias))) {
      violations.push(violation('blocked_domain', location, `owner contains blocked domain ${blocked}`));
    }
  }
  for (const blocked of blockedProjections) {
    if ([...ownerAliases(blocked)].some((alias) => ownerTokens.has(alias))) {
      violations.push(violation('blocked_projection', location, `owner contains blocked projection ${blocked}`));
    }
  }

  const packageTokens = new Set(pathTokens(pkg.name));
  for (const blocked of blockedDomains) {
    if ([...ownerAliases(blocked)].some((alias) => packageTokens.has(alias))) {
      violations.push(violation('blocked_domain', location, `package name contains blocked domain ${blocked}`));
    }
  }
  for (const blocked of blockedProjections) {
    if ([...ownerAliases(blocked)].some((alias) => packageTokens.has(alias))) {
      violations.push(violation('blocked_projection', location, `package name contains blocked projection ${blocked}`));
    }
  }

  if (!pkg.name.startsWith(policy.cargo.packagePrefix)) {
    violations.push(violation('package_prefix', location, `workspace package must start with ${policy.cargo.packagePrefix}`));
  }
  if (role === 'domain' && !policy.domains.developmentAllowlist.includes(owner)) {
    violations.push(violation('blocked_domain', location, `domain owner ${owner} is not in the development allowlist`));
  }
  if (role === 'integration' && policy.domains.registered.some(
    (domain) => [...ownerAliases(domain)].some((alias) => ownerTokens.has(alias)),
  )) {
    violations.push(violation('invalid_owner', location, 'integration owner cannot use a business domain identity'));
  }
  if (role === 'platform' && !policy.owners.platform.includes(owner)) {
    violations.push(violation('invalid_owner', location, `unknown platform owner ${owner}`));
  }
  if (role === 'api' && !policy.owners.api.includes(owner)) {
    violations.push(violation('invalid_owner', location, `unknown API owner ${owner}`));
  }
  if (role === 'core' && (owner !== policy.owners.core || pkg.name !== policy.kernel.package || surface !== 'runtime')) {
    violations.push(violation('invalid_core_package', location, 'core role is reserved for the configured Kernel runtime package'));
  }
  if (role === 'test' && (owner !== policy.owners.test || surface !== 'test_support')) {
    violations.push(violation('invalid_test_package', location, 'test role requires owner=test and surface=test_support'));
  }

  for (const component of components) {
    if (policy.kernel.exclusiveComponents.includes(component) && role !== 'core') {
      violations.push(violation('exclusive_kernel_component', location, `${component} is exclusive to ${policy.kernel.package}`));
    }
  }
  for (const component of policy.kernel.exclusiveComponents) {
    const packageToken = component.replaceAll('_', '-');
    if (pkg.name.includes(packageToken) && role !== 'core') {
      violations.push(violation('exclusive_kernel_component', location, `${packageToken} package is forbidden outside Kernel`));
    }
  }

  const collectorPackageToken = policy.telemetry.collectorComponent.replaceAll('_', '-');
  const declaresCollector = components.includes(policy.telemetry.collectorComponent);
  if (pkg.name.includes(collectorPackageToken) && !declaresCollector) {
    violations.push(violation('invalid_telemetry_collector', location, 'Telemetry Collector package must declare its component'));
  }
  if (declaresCollector) {
    const validCollector = role === 'platform'
      && owner === policy.telemetry.owner
      && pkg.name !== policy.kernel.package
      && ['implementation', 'runtime'].includes(surface);
    if (!validCollector) {
      violations.push(violation(
        'invalid_telemetry_collector',
        location,
        'Telemetry Collector must be a separate telemetry platform implementation/runtime',
      ));
    }
  }
}

export function inspectWorkspacePackages(policy, cargoMetadata) {
  const violations = [];
  const workspaceIds = new Set(list(cargoMetadata?.workspace_members));
  const packages = list(cargoMetadata?.packages).filter(
    (pkg) => workspaceIds.size === 0 || workspaceIds.has(pkg.id),
  );
  const byName = new Map();
  const descriptors = new Map();

  for (const pkg of packages) {
    if (byName.has(pkg.name)) {
      violations.push(violation('duplicate_package', `cargo:${pkg.name}`, 'workspace package names must be unique'));
    }
    byName.set(pkg.name, pkg);
    const descriptor = metadataDescriptor(policy, pkg, violations);
    descriptors.set(pkg.name, descriptor);
    validateDescriptor(policy, pkg, descriptor, violations);
  }

  if (packages.length > 0 && !byName.has(policy.kernel.package)) {
    violations.push(violation('missing_kernel_package', 'cargo:workspace', `${policy.kernel.package} is required when a workspace exists`));
  }

  const kernelDescriptor = descriptors.get(policy.kernel.package);
  if (kernelDescriptor) {
    for (const component of policy.kernel.requiredComponents) {
      if (!kernelDescriptor.components.includes(component)) {
        violations.push(violation('missing_kernel_component', `cargo:${policy.kernel.package}`, `missing Kernel component ${component}`));
      }
    }
  }

  return { violations, packages, descriptors };
}
