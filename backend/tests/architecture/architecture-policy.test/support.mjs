import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { validatePolicy } from '../../../scripts/lib/policy-schema.mjs';
import { collectSourceEntries } from '../../../scripts/lib/repository-scan.mjs';
import { validateSourceEntries } from '../../../scripts/lib/source-boundaries.mjs';
import { canonicalPolicyForTests as policy } from '../support/canonical-policy.mjs';


export function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}
