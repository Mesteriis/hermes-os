import assert from 'node:assert/strict';
import test from 'node:test';

import { validateStorageEntries } from '../../scripts/lib/storage-boundaries.mjs';
import { canonicalPolicyForTests as policy } from './support/canonical-policy.mjs';

function storageEntry(content, overrides = {}) {
  return {
    path: 'modules/tasks/migrations/0001.sql',
    content,
    packageName: 'hermes-tasks-persistence',
    role: 'domain',
    owner: 'tasks',
    surface: 'persistence',
    ...overrides,
  };
}

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('allows schema-qualified owner-local SQL and additive migrations', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      CREATE TYPE hermes_data.tasks_state AS ENUM ('open', 'closed');
      CREATE TABLE hermes_data.tasks_items (
        id UUID PRIMARY KEY,
        state hermes_data.tasks_state NOT NULL,
        parent_id UUID REFERENCES hermes_data.tasks_items(id) ON UPDATE CASCADE
      );
      CREATE INDEX tasks_items_id_idx ON hermes_data.tasks_items (id);
      ALTER TABLE hermes_data.tasks_items ADD COLUMN title TEXT;
      UPDATE hermes_data.tasks_items SET title = 'owned' WHERE id = $1;
      SELECT * FROM hermes_data.tasks_items;
    `),
  ]);

  assert.deepEqual(violations, []);
});

test('allows platform persistence only in hermes_platform', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      CREATE TABLE hermes_platform.events_outbox (id UUID PRIMARY KEY);
      SELECT * FROM hermes_platform.events_outbox;
    `, {
      packageName: 'hermes-events-postgres',
      role: 'platform',
      owner: 'events',
    }),
  ]);

  assert.deepEqual(violations, []);
});

for (const sqlitePackage of [
  {
    packageName: 'hermes-kernel-control-store-sqlite',
    role: 'core',
    owner: 'kernel',
    table: 'kernel_registrations',
  },
  {
    packageName: 'hermes-vault-store-sqlcipher',
    role: 'platform',
    owner: 'vault',
    table: 'vault_secrets',
  },
]) {
  test(`keeps ${sqlitePackage.packageName} outside PostgreSQL schema rules`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry(`CREATE TABLE ${sqlitePackage.table} (id TEXT PRIMARY KEY);`, sqlitePackage),
    ]);

    assert.deepEqual(violations, []);
  });
}

test('retains owner-prefix enforcement for the exact SQLite packages', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE secrets (id TEXT PRIMARY KEY);', {
      packageName: 'hermes-vault-store-sqlcipher',
      role: 'platform',
      owner: 'vault',
    }),
  ]);

  assert.ok(codes(violations).has('unowned_sql_identifier'));
});

test('does not exempt an unregistered SQLite-looking package from PostgreSQL schemas', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE tasks_items (id TEXT PRIMARY KEY);', {
      packageName: 'hermes-tasks-sqlite',
    }),
  ]);

  assert.ok(codes(violations).has('unqualified_sql_identifier'));
});

test('rejects SQL files outside a persistence surface', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM hermes_data.tasks_items;', {
      packageName: 'hermes-tasks-runtime',
      surface: 'runtime',
    }),
  ]);

  assert.ok(codes(violations).has('sql_outside_persistence'));
});

test('rejects raw cross-owner SQL reads and foreign keys', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      SELECT * FROM hermes_data.contacts_people;
      CREATE TABLE hermes_data.tasks_items (
        id UUID PRIMARY KEY,
        contact_id UUID REFERENCES hermes_data.contacts_people(id)
      );
    `),
  ]);

  assert.ok(codes(violations).has('cross_owner_sql'));
});

test('prevents AI persistence from reading another owner table for context', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM hermes_data.tasks_items;', {
      path: 'modules/ai/persistence/src/sql/context.sql',
      packageName: 'hermes-ai-persistence',
      owner: 'ai',
    }),
  ]);

  assert.ok(codes(violations).has('cross_owner_sql'));
});

for (const statement of [
  'INSERT INTO hermes_data.contacts_people (id) VALUES ($1);',
  'UPDATE hermes_data.contacts_people SET name = $2 WHERE id = $1;',
  'DELETE FROM hermes_data.contacts_people WHERE id = $1;',
]) {
  test(`rejects raw cross-owner DML: ${statement.split(' ')[0]}`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry(statement, { path: 'modules/tasks/src/sql/write.sql' }),
    ]);

    assert.ok(codes(violations).has('cross_owner_sql'));
  });
}

test('rejects unqualified PostgreSQL identifiers', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE tasks_items (id UUID PRIMARY KEY);'),
  ]);

  assert.ok(codes(violations).has('unqualified_sql_identifier'));
});

test('rejects a cross-owner PostgreSQL index name while keeping its table qualified', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      CREATE INDEX contacts_items_id_idx ON hermes_data.tasks_items (id);
    `),
  ]);

  assert.ok(codes(violations).has('cross_owner_sql'));
});

for (const schema of ['public', 'private', 'hermes_platform']) {
  test(`rejects ${schema} for a domain raw table access`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry(`SELECT * FROM ${schema}.tasks_items;`),
    ]);

    assert.ok(codes(violations).has('forbidden_sql_schema'));
  });
}

test('rejects hermes_data for platform raw table access', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM hermes_data.events_outbox;', {
      packageName: 'hermes-events-postgres',
      role: 'platform',
      owner: 'events',
    }),
  ]);

  assert.ok(codes(violations).has('forbidden_sql_schema'));
});

test('rejects owner-prefixed SQL for an unsupported PostgreSQL role', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('SELECT * FROM hermes_data.gateway_state;', {
      packageName: 'hermes-gateway-persistence',
      role: 'api',
      owner: 'gateway',
    }),
  ]);

  assert.ok(codes(violations).has('unsupported_sql_role'));
});

test('allows a versioned hermes_platform technical function instead of raw platform DML', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      INSERT INTO hermes_data.tasks_items (id) VALUES ($1);
      SELECT hermes_platform.events_append_outbox_v1($1, $2);
      SELECT * FROM hermes_platform.events_accept_inbox_v1($1);
    `, { path: 'modules/tasks/src/sql/create_task.sql' }),
  ]);

  assert.deepEqual(violations, []);
});

test('rejects unversioned, unlisted and non-platform-owned hermes_platform functions', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      SELECT hermes_platform.events_append_outbox($1);
      SELECT hermes_platform.storage_claim_v2($1);
      SELECT hermes_platform.contacts_lookup_v1($1);
    `, { path: 'modules/tasks/src/sql/create_task.sql' }),
  ]);

  assert.ok(codes(violations).has('invalid_platform_function'));
});

test('rejects SQL not associated with a workspace package', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry('CREATE TABLE hermes_data.tasks_items (id UUID PRIMARY KEY);', {
      packageName: null,
      role: null,
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
      CREATE TABLE hermes_data.tasks_items (
        id UUID PRIMARY KEY,
        note TEXT DEFAULT 'DROP TABLE hermes_data.contacts_people'
      );
    `),
  ]);

  assert.deepEqual(violations, []);
});

const forbiddenMigrationCases = [
  ['DROP', 'DROP TABLE hermes_data.tasks_items;'],
  ['TRUNCATE', 'TRUNCATE TABLE hermes_data.tasks_items;'],
  ['rename', 'ALTER TABLE hermes_data.tasks_items RENAME TO tasks_old;'],
  ['destructive ALTER TYPE', 'ALTER TABLE hermes_data.tasks_items ALTER COLUMN id TYPE TEXT;'],
  ['ALTER COLUMN DEFAULT', 'ALTER TABLE hermes_data.tasks_items ALTER COLUMN id SET DEFAULT gen_random_uuid();'],
  ['ROLE', 'CREATE ROLE tasks_runtime;'],
  ['DATABASE', 'CREATE DATABASE tasks;'],
  ['SCHEMA', 'CREATE SCHEMA tasks;'],
  ['EXTENSION', 'CREATE EXTENSION pg_trgm;'],
  ['GRANT', 'GRANT SELECT ON hermes_data.tasks_items TO tasks_runtime;'],
  ['REVOKE', 'REVOKE SELECT ON hermes_data.tasks_items FROM tasks_runtime;'],
  ['DO block', 'DO $$ BEGIN NULL; END $$;'],
  ['dynamic EXECUTE', "EXECUTE 'DELETE FROM hermes_data.tasks_items';"],
  ['prepared SQL', 'PREPARE remove_task AS DELETE FROM hermes_data.tasks_items;'],
  ['function', 'CREATE FUNCTION hermes_data.tasks_touch_v1() RETURNS void AS $$ BEGIN END $$ LANGUAGE plpgsql;'],
  ['trigger', 'CREATE TRIGGER tasks_touch BEFORE UPDATE ON hermes_data.tasks_items EXECUTE FUNCTION hermes_data.tasks_touch_v1();'],
  ['FDW', 'CREATE FOREIGN TABLE hermes_data.tasks_remote (id UUID) SERVER remote;'],
  ['COPY PROGRAM', "COPY hermes_data.tasks_items FROM PROGRAM 'cat /tmp/tasks';"],
  ['ALTER SYSTEM', "ALTER SYSTEM SET shared_buffers = '1GB';"],
  ['CONCURRENTLY', 'CREATE INDEX CONCURRENTLY tasks_id_idx ON hermes_data.tasks_items (id);'],
  ['TABLESPACE', 'ALTER TABLE hermes_data.tasks_items SET TABLESPACE fast_space;'],
  ['LOAD', "LOAD 'unsafe_extension';"],
  ['VACUUM', 'VACUUM hermes_data.tasks_items;'],
  ['REINDEX', 'REINDEX TABLE hermes_data.tasks_items;'],
  ['CLUSTER', 'CLUSTER hermes_data.tasks_items;'],
  ['BEGIN', 'BEGIN; SELECT * FROM hermes_data.tasks_items; COMMIT;'],
  ['START TRANSACTION', 'START TRANSACTION;'],
  ['COMMIT', 'COMMIT;'],
  ['ROLLBACK', 'ROLLBACK;'],
  ['END', 'END TRANSACTION;'],
  ['ABORT', 'ABORT;'],
  ['SAVEPOINT', 'SAVEPOINT before_tasks;'],
  ['RELEASE SAVEPOINT', 'RELEASE SAVEPOINT before_tasks;'],
];

for (const [name, sql] of forbiddenMigrationCases) {
  test(`rejects ${name} in a V1 migration`, () => {
    const violations = validateStorageEntries(policy(), [storageEntry(sql)]);

    assert.ok(codes(violations).has('forbidden_migration_construct'));
  });
}

for (const path of [
  'modules/tasks/migrations/0002.down.sql',
  'modules/tasks/migrations/0002_down.sql',
  'modules/tasks/migrations/down/0002.sql',
]) {
  test(`rejects down migration file ${path}`, () => {
    const violations = validateStorageEntries(policy(), [
      storageEntry('SELECT * FROM hermes_data.tasks_items;', { path }),
    ]);

    assert.ok(codes(violations).has('down_migration'));
  });
}

test('migration heuristics ignore forbidden words in comments and literals', () => {
  const violations = validateStorageEntries(policy(), [
    storageEntry(`
      -- DROP TABLE hermes_data.tasks_items;
      -- BEGIN; LOAD 'unsafe'; SAVEPOINT ignored;
      ALTER TABLE hermes_data.tasks_items
        ADD COLUMN note TEXT DEFAULT
          'TRUNCATE, ALTER SYSTEM, TABLESPACE, COMMIT and ROLLBACK are documentation';
    `),
  ]);

  assert.deepEqual(violations, []);
});
