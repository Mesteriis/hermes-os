import assert from 'node:assert/strict';
import { access, readdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PACKAGE_ROOTS = [
  'src',
  'development',
  'tests/support',
];
test('Cargo package source roots exist for every workspace package', async () => {
  const packageManifests = (await Promise.all(
    PACKAGE_ROOTS.map((root) => cargoManifests(root)),
  )).flat();
  for (const manifest of packageManifests) {
    const sourceRoot = join(dirname(manifest), 'src');
    await access(new URL(`${sourceRoot}/`, BACKEND_ROOT));
  }
  assert.ok(packageManifests.length > 0);
});

async function cargoManifests(root) {
  const entries = await readdir(new URL(`${root}/`, BACKEND_ROOT), { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && entry.name === 'Cargo.toml')
    .map((entry) => `${root}/${entry.parentPath.replace(new URL(`${root}/`, BACKEND_ROOT).pathname, '').replaceAll('\\', '/')}/Cargo.toml`)
    .map((path) => path.replaceAll('//', '/'));
}
