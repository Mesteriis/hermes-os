import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import { resolve, relative } from 'node:path';

const backendRoot = resolve(import.meta.dirname, '..');
const projectRoot = resolve(backendRoot, '..');
const evidencePath = resolve(projectRoot, '.architecture/current.json');
const policyPath = resolve(backendRoot, 'architecture/policy.json');
const docsRoot = resolve(projectRoot, 'docs');

const evidence = JSON.parse(await readFile(evidencePath, 'utf8'));
const policySha256 = digest(await readFile(policyPath));
const docsSha256 = await digestDocs(docsRoot);

if (evidence.version !== 1 || evidence.policy_sha256 !== policySha256 || evidence.docs_sha256 !== docsSha256) {
  throw new Error('architecture evidence hashes do not match the canonical policy and docs');
}
console.log(`architecture-evidence-check: ok (policy=${policySha256} docs=${docsSha256})`);

async function digestDocs(root) {
  const files = await collectFiles(root);
  const hash = createHash('sha256');
  for (const file of files) {
    hash.update(`${relative(projectRoot, file)}\0`);
    hash.update(await readFile(file));
    hash.update('\0');
  }
  return hash.digest('hex');
}

async function collectFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map(async (entry) => {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) return collectFiles(path);
    return entry.isFile() ? [path] : [];
  }));
  return nested.flat().sort();
}

function digest(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}
