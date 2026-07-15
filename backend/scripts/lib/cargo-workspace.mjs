import { execFile } from 'node:child_process';
import { access } from 'node:fs/promises';
import { dirname, join, relative, sep } from 'node:path';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

export async function readCargoMetadata(repositoryRoot) {
  const manifestPath = join(repositoryRoot, 'Cargo.toml');
  try {
    await access(manifestPath);
  } catch (error) {
    if (error?.code === 'ENOENT') return null;
    throw error;
  }

  const { stdout } = await execFileAsync(
    'cargo',
    ['metadata', '--manifest-path', manifestPath, '--format-version', '1', '--no-deps'],
    {
      cwd: repositoryRoot,
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    },
  );
  return JSON.parse(stdout);
}

function normalize(path) {
  return path.split(sep).join('/');
}

export function workspaceManifestPaths(cargoMetadata, repositoryRoot) {
  const workspaceIds = new Set(cargoMetadata.workspace_members);
  return cargoMetadata.packages
    .filter(({ id }) => workspaceIds.has(id))
    .map(({ manifest_path: manifestPath }) => normalize(relative(repositoryRoot, manifestPath)))
    .sort();
}

export function workspacePackageRoots(cargoMetadata, repositoryRoot, metadataKey) {
  const workspaceIds = new Set(cargoMetadata.workspace_members);
  return cargoMetadata.packages
    .filter(({ id }) => workspaceIds.has(id))
    .map((pkg) => ({
      manifest: normalize(relative(repositoryRoot, pkg.manifest_path)),
      role: pkg.metadata?.[metadataKey]?.role ?? null,
    }))
    .sort((left, right) => left.manifest.localeCompare(right.manifest));
}

export function associateSqlWithWorkspace(sourceEntries, cargoMetadata, repositoryRoot, metadataKey) {
  const workspaceIds = new Set(cargoMetadata.workspace_members);
  const packageRoots = cargoMetadata.packages
    .filter(({ id }) => workspaceIds.has(id))
    .map((pkg) => ({
      name: pkg.name,
      root: normalize(relative(repositoryRoot, dirname(pkg.manifest_path))),
      descriptor: pkg.metadata?.[metadataKey] ?? {},
    }))
    .sort((left, right) => right.root.length - left.root.length);

  return sourceEntries
    .filter(({ path }) => path.toLowerCase().endsWith('.sql'))
    .map((entry) => {
      const ownerPackage = packageRoots.find(
        ({ root }) => root === '' || entry.path.startsWith(`${root}/`),
      );
      return {
        ...entry,
        packageName: ownerPackage?.name ?? null,
        owner: ownerPackage?.descriptor.owner ?? null,
        surface: ownerPackage?.descriptor.surface ?? null,
      };
    });
}
