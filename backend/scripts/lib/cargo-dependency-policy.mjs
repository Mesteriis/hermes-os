import { list, violation } from './validation-diagnostics.mjs';

function isAllowedSameOwnerDependency(source, target) {
  if (source.role !== target.role || source.owner !== target.owner) return false;
  if (target.surface === 'contract') return true;

  switch (source.surface) {
    case 'implementation':
      return target.surface === 'implementation';
    case 'persistence':
      return ['implementation', 'persistence'].includes(target.surface);
    case 'runtime':
      return ['implementation', 'persistence', 'runtime'].includes(target.surface);
    default:
      return false;
  }
}

function isAllowedDependency(policy, source, target) {
  if (source.role === target.role && source.owner === target.owner) {
    return isAllowedSameOwnerDependency(source, target);
  }
  if (target.surface !== 'contract') return false;

  switch (source.role) {
    case 'domain':
      return ['platform', 'engine'].includes(target.role);
    case 'integration':
      return ['platform', 'engine'].includes(target.role)
        || (target.role === 'domain'
          && policy.dependencies.integrationDomainContractOwners.includes(target.owner));
    case 'workflow':
      return ['domain', 'integration', 'platform', 'engine', 'api'].includes(target.role);
    case 'engine':
      return target.role === 'platform';
    case 'platform':
      return target.role === 'platform';
    case 'api':
      return ['domain', 'integration', 'workflow', 'platform', 'engine'].includes(target.role);
    case 'core':
      return ['platform', 'api'].includes(target.role);
    case 'test':
      return target.role !== 'core';
    default:
      return false;
  }
}

export function validateDependencyEdges(policy, packages, descriptors) {
  const violations = [];

  for (const pkg of packages) {
    const source = descriptors.get(pkg.name);
    if (!source) continue;
    const isTelemetryImplementation = source.role === 'platform'
      && source.owner === policy.telemetry.owner
      && source.surface !== 'contract';

    for (const dependency of list(pkg.dependencies)) {
      const kind = dependency.kind ?? 'normal';
      const target = descriptors.get(dependency.name);

      if (policy.storage.clientDependencies.includes(dependency.name) && source.surface !== 'persistence') {
        violations.push(violation(
          'storage_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `${dependency.name} is allowed only in a persistence surface`,
        ));
      }

      if (!target) {
        if (isTelemetryImplementation && policy.telemetry.forbiddenDependencies.includes(dependency.name)) {
          violations.push(violation(
            'telemetry_dependency',
            `cargo:${pkg.name}:${kind}:${dependency.name}`,
            `telemetry implementation cannot depend on ${dependency.name}`,
          ));
        }
        continue;
      }

      if (source.role !== 'test' && target.role === 'test') {
        if (kind === 'dev') continue;
        violations.push(violation(
          'production_test_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'production packages may use test support only as a dev dependency',
        ));
        continue;
      }

      if (source.owner !== target.owner && target.surface !== 'contract') {
        violations.push(violation(
          'implementation_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          'cross-owner dependencies must target contracts, not implementations',
        ));
        continue;
      }

      if (!isAllowedDependency(policy, source, target)) {
        violations.push(violation(
          'forbidden_dependency',
          `cargo:${pkg.name}:${kind}:${dependency.name}`,
          `${source.role}:${source.owner} cannot depend on ${target.role}:${target.owner}`,
        ));
      }
    }
  }

  return violations;
}
