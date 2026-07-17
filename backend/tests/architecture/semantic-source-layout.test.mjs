import assert from 'node:assert/strict';
import { readdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PACKAGE_ROOTS = [
  'src',
  'development',
  'tests/support',
];
const SOURCE_ROOT_FILES = new Set(['lib.rs', 'main.rs']);

test('Cargo package source roots contain only composition files', async () => {
  const packageManifests = (await Promise.all(
    PACKAGE_ROOTS.map((root) => cargoManifests(root)),
  )).flat();
  const violations = [];

  for (const manifest of packageManifests) {
    const sourceRoot = join(dirname(manifest), 'src');
    const entries = await readdir(new URL(`${sourceRoot}/`, BACKEND_ROOT), { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isFile() && entry.name.endsWith('.rs') && !SOURCE_ROOT_FILES.has(entry.name)) {
        violations.push(`${sourceRoot}/${entry.name}`);
      }
    }
  }

  assert.deepEqual(
    violations.sort(),
    [],
    'package source roots must delegate behavior to semantic namespaces',
  );
});

async function cargoManifests(root) {
  const entries = await readdir(new URL(`${root}/`, BACKEND_ROOT), { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && entry.name === 'Cargo.toml')
    .map((entry) => `${root}/${entry.parentPath.replace(new URL(`${root}/`, BACKEND_ROOT).pathname, '').replaceAll('\\', '/')}/Cargo.toml`)
    .map((path) => path.replaceAll('//', '/'));
}
