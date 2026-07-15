import assert from 'node:assert/strict';
import test from 'node:test';

import { validateStorageEntries } from '../../scripts/lib/storage-boundaries.mjs';
import { canonicalPolicyForTests as policy } from './support/canonical-policy.mjs';

function storageEntry(content, overrides = {}) {
  return {
    path: 'modules/tasks/migrations/0001.sql',
    content,
    packageName: 'hermes-tasks-persistence',
    owner: 'tasks',
    surface: 'persistence',
    ...overrides,
  };
}

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('allows owner-prefixed SQL in a persistence surface', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      CREATE TABLE tasks_items (id UUID PRIMARY KEY);
      CREATE INDEX tasks_items_id_idx ON tasks_items (id);
      SELECT * FROM tasks_items;
    `),
  ]);

  assert.deepEqual(violations, []);
});

test('rejects SQL files outside a persistence surface', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM tasks_items;', {
      packageName: 'hermes-tasks-runtime',
      surface: 'runtime',
    }),
  ]);

  assert.ok(codes(violations).has('sql_outside_persistence'));
});

test('rejects cross-owner SQL reads and foreign keys', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      SELECT * FROM contacts_people;
      CREATE TABLE tasks_items (
        id UUID PRIMARY KEY,
        contact_id UUID REFERENCES contacts_people(id)
      );
    `),
  ]);

  assert.ok(codes(violations).has('cross_owner_sql'));
});

test('rejects unprefixed SQL identifiers', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE items (id UUID PRIMARY KEY);'),
  ]);

  assert.ok(codes(violations).has('unowned_sql_identifier'));
});

test('rejects SQL not associated with a workspace package', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE tasks_items (id UUID PRIMARY KEY);', {
      packageName: null,
      owner: null,
      surface: null,
    }),
  ]);

  assert.ok(codes(violations).has('orphan_sql'));
});

test('ignores ownership-looking SQL inside comments', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      -- SELECT * FROM contacts_people;
      /* CREATE TABLE projects_items (id UUID); */
      CREATE TABLE tasks_items (id UUID PRIMARY KEY);
    `),
  ]);

  assert.deepEqual(violations, []);
});
