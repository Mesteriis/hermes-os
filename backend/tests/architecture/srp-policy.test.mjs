import assert from 'node:assert/strict';
import test from 'node:test';

import { validateSrpEntries } from '../../scripts/lib/srp-policy.mjs';
import { canonicalPolicyForTests } from './support/canonical-policy.mjs';

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('does not infer responsibility from source line count', () => {
  const content = Array.from({ length: 501 }, () => '// line').join('\n');
  const violations = validateSrpEntries(canonicalPolicyForTests(), [
    { path: 'tests/example.test.mjs', content },
  ]);
  assert.deepEqual(violations, []);
});

test('does not infer responsibility from function line count', () => {
  const body = Array.from({ length: 60 }, () => '  let value = 1;').join('\n');
  const violations = validateSrpEntries(canonicalPolicyForTests(), [
    { path: 'src/example.rs', content: `fn oversized() {\n${body}\n}` },
  ]);
  assert.deepEqual(violations, []);
});

test('excludes generated source paths only', () => {
  const content = Array.from({ length: 700 }, () => '// generated').join('\n');
  assert.deepEqual(validateSrpEntries(canonicalPolicyForTests(), [
    { path: 'src/gen/generated.rs', content },
  ]), []);
});
