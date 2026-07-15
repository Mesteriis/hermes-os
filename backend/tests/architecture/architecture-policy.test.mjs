import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { validatePolicy } from '../../scripts/lib/policy-schema.mjs';
import { collectSourceEntries } from '../../scripts/lib/repository-scan.mjs';
import { validateSourceEntries } from '../../scripts/lib/source-boundaries.mjs';
import { canonicalPolicyForTests as policy } from './support/canonical-policy.mjs';

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('accepts the canonical registry and current development allowlist', () => {
  assert.deepEqual(validatePolicy(policy()), []);
});

test('requires registered domains to be partitioned between allowed and blocked', () => {
  const invalid = policy();
  invalid.domains.developmentAllowlist = invalid.domains.developmentAllowlist.filter(
    (owner) => owner !== 'ai',
  );

  assert.ok(codes(validatePolicy(invalid)).has('domain_partition'));
});

test('requires Event Hub and Telemetry control to remain Kernel components', () => {
  const invalid = policy();
  invalid.kernel.requiredComponents = invalid.kernel.requiredComponents.filter(
    (component) => component !== 'event_hub',
  );

  assert.ok(codes(validatePolicy(invalid)).has('kernel_components'));
});

test('requires integration domain contracts to stay inside the enabled allowlist', () => {
  const invalid = policy();
  invalid.dependencies.integrationDomainContractOwners = ['communications', 'projects'];

  assert.ok(codes(validatePolicy(invalid)).has('dependency_policy'));
});

test('requires an explicit storage client dependency registry', () => {
  const invalid = policy();
  invalid.storage.clientDependencies = [];

  assert.ok(codes(validatePolicy(invalid)).has('storage_policy'));
});

test('requires explicit production and test-only workspace roots', () => {
  const invalid = policy();
  delete invalid.tests;

  assert.ok(codes(validatePolicy(invalid)).has('test_layout_policy'));
});

test('requires an explicit single-layout policy', () => {
  const invalid = policy();
  delete invalid.layout;

  assert.ok(codes(validatePolicy(invalid)).has('layout_policy'));
});

for (const owner of ['relationships', 'projects', 'obligations', 'decisions', 'knowledge', 'review']) {
  test(`rejects a production path for blocked domain ${owner}`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `modules/${owner}/src/lib.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('blocked_source_owner'));
  });
}

test('rejects singular aliases of blocked owners in production paths', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/project/src/lib.rs', content: '' },
  ]);

  assert.ok(codes(violations).has('blocked_source_owner'));
});

for (const owner of ['graph', 'timeline', 'search', 'context']) {
  test(`rejects a production path for blocked projection ${owner}`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `crates/hermes-${owner}-runtime/src/lib.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('blocked_projection'));
  });
}

test('rejects SQL ownership for a blocked domain', () => {
  const violations = validateSourceEntries(policy(), [
    {
      path: 'modules/tasks/migrations/0001.sql',
      content: 'CREATE TABLE projects (id UUID PRIMARY KEY);',
    },
  ]);

  assert.ok(codes(violations).has('blocked_sql_owner'));
});

test('does not treat SQL comments as ownership declarations', () => {
  const violations = validateSourceEntries(policy(), [
    {
      path: 'modules/tasks/migrations/0001.sql',
      content: '-- CREATE TABLE projects (id UUID);\nCREATE TABLE tasks (id UUID);',
    },
  ]);

  assert.deepEqual(violations, []);
});

test('allows source paths for enabled domains including AI', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/communications/src/lib.rs', content: '' },
    { path: 'modules/contacts/src/lib.rs', content: '' },
    { path: 'modules/organizations/src/lib.rs', content: '' },
    { path: 'modules/tasks/src/lib.rs', content: '' },
    { path: 'modules/calendar/src/lib.rs', content: '' },
    { path: 'modules/documents/src/lib.rs', content: '' },
    { path: 'modules/ai/src/lib.rs', content: '' },
  ]);

  assert.deepEqual(violations, []);
});

test('does not infer ownership from ordinary filenames below an allowed owner', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/tasks/src/request_context.rs', content: '' },
    { path: 'crates/hermes-documents-runtime/src/search_adapter.rs', content: '' },
  ]);

  assert.deepEqual(violations, []);
});

for (const directory of ['tests', 'fixtures', 'snapshots']) {
  test(`rejects package-local ${directory} directories in production source`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `src/domains/tasks/runtime/${directory}/api.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('test_in_production_source'));
  });
}

for (const path of [
  'src/domains/tasks/implementation/src/task_test.rs',
  'src/domains/tasks/implementation/src/test_task.rs',
]) {
  test(`rejects production test filename ${path}`, () => {
    const violations = validateSourceEntries(policy(), [{ path, content: '' }]);

    assert.ok(codes(violations).has('test_in_production_source'));
  });
}

test('rejects snapshot files in production source', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/state.snap',
    content: '',
  }]);

  assert.ok(codes(violations).has('test_in_production_source'));
});

test('rejects inline cfg(test) modules in production Rust', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '#[cfg(test)]\nmod tests;',
  }]);

  assert.ok(codes(violations).has('test_in_production_source'));
});

test('rejects compound Rust cfg attributes that include test code', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '#[cfg(all(test, feature = "slow"))]\nmod slow_tests;',
  }]);

  assert.ok(codes(violations).has('test_in_production_source'));
});

test('does not treat cfg(test) text inside a Rust comment as an inline test', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '// #[cfg(test)] is forbidden by ADR-0211',
  }]);

  assert.deepEqual(violations, []);
});

test('filesystem scan reads Rust content and exposes nested test paths to policy', async (context) => {
  const root = await mkdtemp(join(tmpdir(), 'hermes-architecture-'));
  context.after(() => rm(root, { recursive: true, force: true }));
  const packageSource = join(root, 'src', 'domains', 'tasks', 'implementation', 'src');
  await mkdir(join(packageSource, 'tests'), { recursive: true });
  await writeFile(join(packageSource, 'lib.rs'), '#[cfg(test)]\nmod tests;\n');
  await writeFile(join(packageSource, 'tests', 'api.rs'), 'fn helper() {}\n');
  await symlink(join(root, 'outside-source'), join(packageSource, 'linked-source'));

  const entries = await collectSourceEntries(root, policy());
  const violations = validateSourceEntries(policy(), entries);

  assert.match(
    entries.find(({ path }) => path.endsWith('/lib.rs'))?.content ?? '',
    /cfg\(test\)/u,
  );
  assert.ok(entries.some(({ path }) => path.includes('/tests/')));
  assert.ok(entries.some(({ isSymbolicLink }) => isSymbolicLink === true));
  assert.ok(codes(violations).has('test_in_production_source'));
  assert.ok(codes(violations).has('source_symlink'));
});
