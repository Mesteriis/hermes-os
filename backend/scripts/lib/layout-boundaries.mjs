import { lstat } from 'node:fs/promises';
import { resolve } from 'node:path';

import { violation } from './validation-diagnostics.mjs';

async function existingPaths(root, configuredPaths) {
  const existing = [];
  const symlinks = [];
  for (const path of configuredPaths) {
    try {
      const stats = await lstat(resolve(root, path));
      existing.push(path);
      if (stats.isSymbolicLink()) symlinks.push(path);
    } catch (error) {
      if (error?.code !== 'ENOENT') throw error;
    }
  }
  return { existing, symlinks };
}

export async function inspectBackendLayout(policy, backendRoot, projectRoot) {
  const required = await existingPaths(backendRoot, policy.layout.requiredBackendPaths);
  const forbiddenBackend = await existingPaths(backendRoot, policy.layout.forbiddenBackendPaths);
  const forbiddenProject = await existingPaths(projectRoot, policy.layout.forbiddenProjectPaths);

  return {
    requiredBackendPaths: policy.layout.requiredBackendPaths,
    existingBackendPaths: new Set(required.existing),
    forbiddenBackendPaths: forbiddenBackend.existing,
    forbiddenProjectPaths: forbiddenProject.existing,
    symlinkPaths: [
      ...required.symlinks.map((path) => `backend/${path}`),
      ...forbiddenBackend.symlinks.map((path) => `backend/${path}`),
      ...forbiddenProject.symlinks,
    ],
  };
}

export function validateBackendLayout({
  requiredBackendPaths,
  existingBackendPaths,
  forbiddenBackendPaths = [],
  forbiddenProjectPaths,
  symlinkPaths = [],
}) {
  const violations = [];
  for (const path of requiredBackendPaths) {
    if (!existingBackendPaths.has(path)) {
      violations.push(violation(
        'missing_backend_layout_path',
        `backend/${path}`,
        'required backend-owned architecture asset is missing',
      ));
    }
  }
  for (const path of forbiddenProjectPaths) {
    violations.push(violation(
      'dual_backend_layout',
      path,
      'backend-owned path must not exist at project root',
    ));
  }
  for (const path of forbiddenBackendPaths) {
    violations.push(violation(
      'misplaced_backend_path',
      `backend/${path}`,
      'backend packages and migrations must live below backend/src owner boundaries',
    ));
  }
  for (const path of symlinkPaths) {
    violations.push(violation(
      'backend_layout_symlink',
      path,
      'compatibility symlinks are forbidden by ADR-0211',
    ));
  }
  return violations;
}
