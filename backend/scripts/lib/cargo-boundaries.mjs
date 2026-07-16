import { validateDependencyEdges } from './cargo-dependency-policy.mjs';
import { inspectWorkspacePackages } from './cargo-package-policy.mjs';
export {
  validateCurrentImplementationInventory,
  validateCurrentImplementationSourceCoverage,
} from './current-implementation-inventory.mjs';
import { violation } from './validation-diagnostics.mjs';

export function validateCargoMetadata(policy, cargoMetadata) {
  const { violations, packages, descriptors } = inspectWorkspacePackages(policy, cargoMetadata);
  return [
    ...violations,
    ...validateDependencyEdges(policy, packages, descriptors),
  ];
}

export function validateWorkspaceManifestCoverage(
  productionManifests,
  testManifests,
  developmentManifests,
  workspaceManifests,
) {
  const registered = new Set(workspaceManifests);
  const scoped = new Set([...productionManifests, ...testManifests, ...developmentManifests]);
  const orphanProductionManifests = productionManifests
    .filter((manifest) => !registered.has(manifest))
    .map((manifest) => violation(
      'orphan_cargo_manifest',
      manifest,
      'production Cargo package must be a member of the clean-room workspace',
    ));
  const orphanTestManifests = testManifests
    .filter((manifest) => !registered.has(manifest))
    .map((manifest) => violation(
      'orphan_cargo_manifest',
      manifest,
      'test-support Cargo package must be a member of the clean-room workspace',
    ));
  const orphanDevelopmentManifests = developmentManifests
    .filter((manifest) => !registered.has(manifest))
    .map((manifest) => violation(
      'orphan_cargo_manifest',
      manifest,
      'development Cargo package must be a member of the clean-room workspace',
    ));
  const unscopedPackages = workspaceManifests
    .filter((manifest) => !scoped.has(manifest))
    .map((manifest) => violation(
      'unscoped_workspace_package',
      manifest,
      'clean-room workspace package must live under a configured production source root',
    ));
  return [
    ...orphanProductionManifests,
    ...orphanTestManifests,
    ...orphanDevelopmentManifests,
    ...unscopedPackages,
  ];
}

export function validateWorkspacePackageRoots(
  policy,
  workspacePackages,
  productionManifests,
  testManifests,
  developmentManifests,
) {
  const violations = [];
  for (const pkg of workspacePackages) {
    if (productionManifests.has(pkg.manifest) && pkg.role === 'test') {
      violations.push(violation(
        'test_package_in_production_root',
        pkg.manifest,
        `role=${policy.owners.test} packages must live under ${policy.tests.workspaceRoots.join(', ')}`,
      ));
    }
    if (testManifests.has(pkg.manifest) && pkg.role !== 'test') {
      violations.push(violation(
        'production_package_in_test_root',
        pkg.manifest,
        'only role=test packages may live under the test-only workspace root',
      ));
    }
    if (developmentManifests.has(pkg.manifest) && pkg.role !== 'development') {
      violations.push(violation(
        'production_package_in_development_root',
        pkg.manifest,
        'only role=development packages may live in the explicit development runtime root',
      ));
    }
  }
  return violations;
}
