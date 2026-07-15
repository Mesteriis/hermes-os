import { readFile } from 'node:fs/promises';

import { duplicates, list, violation } from './validation-diagnostics.mjs';

export async function loadPolicy(path) {
  return JSON.parse(await readFile(path, 'utf8'));
}

export function validatePolicy(policy) {
  const violations = [];
  if (policy?.schemaVersion !== 1) {
    violations.push(violation('policy_schema', 'architecture/policy.json', 'schemaVersion must be 1'));
  }

  const registered = list(policy?.domains?.registered);
  const allowed = list(policy?.domains?.developmentAllowlist);
  const blocked = list(policy?.domains?.blocked);
  for (const [name, values] of [['registered', registered], ['developmentAllowlist', allowed], ['blocked', blocked]]) {
    const repeated = duplicates(values);
    if (repeated.length > 0) {
      violations.push(violation('duplicate_policy_value', `domains.${name}`, `duplicate values: ${repeated.join(', ')}`));
    }
  }

  const expected = new Set(registered);
  const partition = [...allowed, ...blocked];
  const partitionSet = new Set(partition);
  const overlaps = allowed.filter((owner) => blocked.includes(owner));
  const missing = registered.filter((owner) => !partitionSet.has(owner));
  const unknown = partition.filter((owner) => !expected.has(owner));
  if (overlaps.length > 0 || missing.length > 0 || unknown.length > 0 || partition.length !== registered.length) {
    violations.push(violation(
      'domain_partition',
      'domains',
      `allowed/blocked must partition registered domains; overlaps=${overlaps.join(',') || '-'} missing=${missing.join(',') || '-'} unknown=${unknown.join(',') || '-'}`,
    ));
  }

  const requiredCore = new Set(list(policy?.kernel?.requiredComponents));
  if (!requiredCore.has('event_hub') || !requiredCore.has('telemetry_control')) {
    violations.push(violation('kernel_components', 'kernel.requiredComponents', 'Event Hub and telemetry_control are mandatory Kernel components'));
  }
  for (const component of list(policy?.kernel?.exclusiveComponents)) {
    if (!requiredCore.has(component)) {
      violations.push(violation('kernel_components', 'kernel.exclusiveComponents', `${component} must also be required by Kernel`));
    }
  }

  if (!list(policy?.cargo?.roles).length || !list(policy?.cargo?.surfaces).length) {
    violations.push(violation('cargo_policy', 'cargo', 'roles and surfaces must be non-empty'));
  }
  if (!list(policy?.projections?.blockedOwners).length) {
    violations.push(violation('projection_policy', 'projections', 'blocked projection owners must be explicit'));
  }
  if (!list(policy?.telemetry?.forbiddenDependencies).length) {
    violations.push(violation('telemetry_policy', 'telemetry', 'Telemetry Collector forbidden dependencies must be explicit'));
  }
  const integrationDomainContracts = list(policy?.dependencies?.integrationDomainContractOwners);
  if (!integrationDomainContracts.length
    || integrationDomainContracts.some((owner) => !allowed.includes(owner))) {
    violations.push(violation(
      'dependency_policy',
      'dependencies.integrationDomainContractOwners',
      'integration domain contracts must be an explicit subset of the enabled domain allowlist',
    ));
  }
  if (!list(policy?.storage?.clientDependencies).length) {
    violations.push(violation('storage_policy', 'storage.clientDependencies', 'storage client dependencies must be explicit'));
  }
  if (!list(policy?.source?.ownerPathMarkers).length) {
    violations.push(violation('source_policy', 'source', 'ownerPathMarkers must be explicit'));
  }
  if (!list(policy?.source?.roots).length
    || !list(policy?.source?.contentExtensions).length
    || policy?.source?.forbidSymlinks !== true) {
    violations.push(violation(
      'source_policy',
      'source',
      'production roots, readable content extensions and symlink prohibition must be explicit',
    ));
  }

  const testRoots = list(policy?.tests?.workspaceRoots);
  const forbiddenTestDirectories = list(policy?.tests?.forbiddenProductionDirectories);
  const forbiddenTestFiles = list(policy?.tests?.forbiddenProductionFilePatterns);
  if (!testRoots.length
    || !forbiddenTestDirectories.length
    || !forbiddenTestFiles.length
    || policy?.tests?.forbidInlineRustTests !== true) {
    violations.push(violation(
      'test_layout_policy',
      'tests',
      'test-only roots and production test-code prohibitions must be explicit',
    ));
  }

  if (!list(policy?.layout?.requiredBackendPaths).length
    || !list(policy?.layout?.forbiddenProjectPaths).length
    || !list(policy?.layout?.forbiddenBackendPaths).length) {
    violations.push(violation(
      'layout_policy',
      'layout',
      'required backend paths and forbidden legacy paths must be explicit',
    ));
  }

  return violations;
}
