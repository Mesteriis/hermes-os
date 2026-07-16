import { lstat, readdir, readFile } from 'node:fs/promises';
import { extname, join, relative, resolve, sep } from 'node:path';

function normalize(path) {
  return path.split(sep).join('/');
}

export async function collectSourceEntries(repositoryRoot, policy) {
  return collectEntriesFromRoots(
    repositoryRoot,
    policy.source.roots,
    policy.source.ignoredDirectories,
    policy.source.contentExtensions,
  );
}

export async function collectTestSupportEntries(repositoryRoot, policy) {
  return collectEntriesFromRoots(
    repositoryRoot,
    policy.tests.workspaceRoots,
    policy.source.ignoredDirectories,
    policy.source.contentExtensions,
  );
}

export async function collectEntriesFromRoots(
  repositoryRoot,
  configuredRoots,
  ignoredDirectories,
  contentExtensions,
) {
  const root = resolve(repositoryRoot);
  const ignored = new Set(ignoredDirectories);
  const readableExtensions = new Set(contentExtensions);
  const entries = [];

  async function walk(absolutePath, includeDirectory) {
    let stats;
    try {
      stats = await lstat(absolutePath);
    } catch (error) {
      if (error?.code === 'ENOENT') return;
      throw error;
    }

    const relativePath = normalize(relative(root, absolutePath));
    if (relativePath.startsWith('../') || relativePath === '..') {
      throw new Error(`source root escapes repository: ${absolutePath}`);
    }
    if (stats.isSymbolicLink()) {
      entries.push({
        path: relativePath,
        content: '',
        isDirectory: false,
        isSymbolicLink: true,
      });
      return;
    }

    if (stats.isDirectory()) {
      const name = absolutePath.split(sep).at(-1);
      if (includeDirectory && ignored.has(name)) return;
      if (includeDirectory) entries.push({
        path: relativePath,
        content: '',
        isDirectory: true,
        isSymbolicLink: false,
      });
      const children = await readdir(absolutePath);
      children.sort();
      for (const child of children) await walk(join(absolutePath, child), true);
      return;
    }

    const content = readableExtensions.has(extname(absolutePath).toLowerCase())
      ? await readFile(absolutePath, 'utf8')
      : '';
    entries.push({
      path: relativePath,
      content,
      isDirectory: false,
      isSymbolicLink: false,
    });
  }

  for (const sourceRoot of configuredRoots) {
    await walk(resolve(root, sourceRoot), false);
  }
  return entries;
}
