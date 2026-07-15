import assert from 'node:assert/strict';
import test from 'node:test';

import { validateBackendLayout } from '../../scripts/lib/layout-boundaries.mjs';

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('accepts the canonical backend-owned layout', () => {
  const violations = validateBackendLayout({
    requiredBackendPaths: [
      'Cargo.toml',
      'Makefile',
      'architecture/policy.json',
      'scripts/check-architecture-policy.mjs',
      'tests/architecture',
    ],
    existingBackendPaths: new Set([
      'Cargo.toml',
      'Makefile',
      'architecture/policy.json',
      'scripts/check-architecture-policy.mjs',
      'tests/architecture',
    ]),
    forbiddenProjectPaths: [],
  });

  assert.deepEqual(violations, []);
});

test('rejects missing canonical backend paths', () => {
  const violations = validateBackendLayout({
    requiredBackendPaths: ['Cargo.toml', 'architecture/policy.json'],
    existingBackendPaths: new Set(['Cargo.toml']),
    forbiddenProjectPaths: [],
  });

  assert.ok(codes(violations).has('missing_backend_layout_path'));
});

test('rejects the former root-level backend layout', () => {
  const violations = validateBackendLayout({
    requiredBackendPaths: [],
    existingBackendPaths: new Set(),
    forbiddenProjectPaths: ['Makefile', 'architecture', 'scripts', 'tests/architecture'],
  });

  assert.ok(codes(violations).has('dual_backend_layout'));
});

test('rejects global backend migrations outside an owner persistence package', () => {
  const violations = validateBackendLayout({
    requiredBackendPaths: [],
    existingBackendPaths: new Set(),
    forbiddenBackendPaths: ['migrations'],
    forbiddenProjectPaths: [],
  });

  assert.ok(codes(violations).has('misplaced_backend_path'));
});

test('rejects compatibility symlinks between layouts', () => {
  const violations = validateBackendLayout({
    requiredBackendPaths: [],
    existingBackendPaths: new Set(),
    forbiddenProjectPaths: [],
    symlinkPaths: ['scripts'],
  });

  assert.ok(codes(violations).has('backend_layout_symlink'));
});
