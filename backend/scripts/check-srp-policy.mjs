#!/usr/bin/env node

import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { loadPolicy } from './lib/policy-schema.mjs';
import { collectEntriesFromRoots } from './lib/repository-scan.mjs';
import { validateSrpEntries } from './lib/srp-policy.mjs';
import { formatViolations } from './lib/validation-diagnostics.mjs';

const backendRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const policy = await loadPolicy(join(backendRoot, 'architecture', 'policy.json'));
const entries = await collectEntriesFromRoots(
  backendRoot,
  policy.source.srpRoots,
  policy.source.ignoredDirectories,
  policy.source.srpContentExtensions,
);
const violations = validateSrpEntries(policy, entries);
if (violations.length > 0) {
  console.error(`srp-policy-check: failed (${violations.length} violations)`);
  console.error(formatViolations(violations));
  process.exitCode = 1;
} else {
  console.log(`srp-policy-check: ok (${entries.length} paths checked)`);
}
