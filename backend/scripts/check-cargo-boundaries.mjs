#!/usr/bin/env node

import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  validateCargoMetadata,
  validateWorkspaceManifestCoverage,
  validateWorkspacePackageRoots,
} from './lib/cargo-boundaries.mjs';
import {
  associateSqlWithWorkspace,
  readCargoMetadata,
  workspaceManifestPaths,
  workspacePackageRoots,
} from './lib/cargo-workspace.mjs';
import { loadPolicy, validatePolicy } from './lib/policy-schema.mjs';
import {
  collectSourceEntries,
  collectTestSupportEntries,
} from './lib/repository-scan.mjs';
import { validateStorageEntries } from './lib/storage-boundaries.mjs';
import { formatViolations } from './lib/validation-diagnostics.mjs';

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const policyPath = join(repositoryRoot, 'architecture', 'policy.json');

const policy = await loadPolicy(policyPath);
const policyViolations = validatePolicy(policy);
if (policyViolations.length > 0) {
  console.error(`cargo-boundaries-check: invalid policy (${policyViolations.length} violations)`);
  console.error(formatViolations(policyViolations));
  process.exitCode = 1;
} else {
  const cargoMetadata = await readCargoMetadata(repositoryRoot);
  const sourceEntries = await collectSourceEntries(repositoryRoot, policy);
  const testSupportEntries = await collectTestSupportEntries(repositoryRoot, policy);
  const productionManifests = sourceEntries
    .map(({ path }) => path)
    .filter((path) => path.endsWith('/Cargo.toml'));
  const testManifests = testSupportEntries
    .map(({ path }) => path)
    .filter((path) => path.endsWith('/Cargo.toml'));
  const workspaceManifests = cargoMetadata === null
    ? []
    : workspaceManifestPaths(cargoMetadata, repositoryRoot);
  const coverageViolations = validateWorkspaceManifestCoverage(
    productionManifests,
    testManifests,
    workspaceManifests,
  );
  const rootViolations = cargoMetadata === null
    ? []
    : validateWorkspacePackageRoots(
      policy,
      workspacePackageRoots(cargoMetadata, repositoryRoot, policy.cargo.metadataKey),
      new Set(productionManifests),
      new Set(testManifests),
    );
  const workspaceViolations = [...coverageViolations, ...rootViolations];

  if (workspaceViolations.length > 0) {
    console.error(`cargo-boundaries-check: failed (${workspaceViolations.length} violations)`);
    console.error(formatViolations(workspaceViolations));
    process.exitCode = 1;
  } else if (cargoMetadata === null) {
    console.log('cargo-boundaries-check: ok (no active clean-room Cargo workspace yet)');
  } else {
    const storageEntries = associateSqlWithWorkspace(
      sourceEntries,
      cargoMetadata,
      repositoryRoot,
      policy.cargo.metadataKey,
    );
    const violations = [
      ...validateCargoMetadata(policy, cargoMetadata),
      ...validateStorageEntries(policy, storageEntries),
    ];
    if (violations.length > 0) {
      console.error(`cargo-boundaries-check: failed (${violations.length} violations)`);
      console.error(formatViolations(violations));
      process.exitCode = 1;
    } else {
      console.log(`cargo-boundaries-check: ok (${cargoMetadata.workspace_members.length} workspace packages checked)`);
    }
  }
}
