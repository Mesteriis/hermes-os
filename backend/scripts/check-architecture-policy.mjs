#!/usr/bin/env node

import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  inspectBackendLayout,
  validateBackendLayout,
} from './lib/layout-boundaries.mjs';
import { loadPolicy, validatePolicy } from './lib/policy-schema.mjs';
import { collectSourceEntries } from './lib/repository-scan.mjs';
import { validateSourceEntries } from './lib/source-boundaries.mjs';
import { formatViolations } from './lib/validation-diagnostics.mjs';

const backendRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const projectRoot = dirname(backendRoot);
const policyPath = join(backendRoot, 'architecture', 'policy.json');

const policy = await loadPolicy(policyPath);
const policyViolations = validatePolicy(policy);

if (policyViolations.length > 0) {
  console.error(`architecture-policy-check: invalid policy (${policyViolations.length} violations)`);
  console.error(formatViolations(policyViolations));
  process.exitCode = 1;
} else {
  const layout = await inspectBackendLayout(policy, backendRoot, projectRoot);
  const sourceEntries = await collectSourceEntries(backendRoot, policy);
  const violations = [
    ...validateBackendLayout(layout),
    ...validateSourceEntries(policy, sourceEntries),
  ];
  if (violations.length > 0) {
    console.error(`architecture-policy-check: failed (${violations.length} violations)`);
    console.error(formatViolations(violations));
    process.exitCode = 1;
  } else {
    console.log(`architecture-policy-check: ok (${sourceEntries.length} production paths checked)`);
  }
}
