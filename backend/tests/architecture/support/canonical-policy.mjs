import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const currentDirectory = dirname(fileURLToPath(import.meta.url));
const policyPath = join(currentDirectory, '..', '..', '..', 'architecture', 'policy.json');
const canonicalPolicy = JSON.parse(readFileSync(policyPath, 'utf8'));

export function canonicalPolicyForTests() {
  return structuredClone(canonicalPolicy);
}
