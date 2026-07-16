import assert from 'node:assert/strict';
import { chmod, copyFile, mkdir, mkdtemp, readFile, rm, stat, symlink, writeFile } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { createConnection } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

function waitForChildExit(child) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return Promise.resolve();
  }
  return new Promise((resolve) => child.once('exit', resolve));
}

test('Kernel reaches recovery_only with an explicit private data directory', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-recovery-'));
  try {
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^state=recovery_only\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Kernel migrates a v8 pending settings revision without declaring it current', async () => {
  const root = await mkdtemp(join(tmpdir(), 'hermes-kernel-settings-migration-'));
  const dataDir = join(root, 'data');
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  const instanceId = 'a'.repeat(32);
  try {
    await mkdir(dataDir, { mode: 0o700 });
    await writeFile(
      join(dataDir, '.hermes-installation-anchor-v1'),
      `hermes-installation-anchor-v1:${instanceId}\n`,
      { mode: 0o600 },
    );
    const fixture = spawnSync(
      'sqlite3',
      [storePath, `
        CREATE TABLE hermes_kernel_control_store_metadata (
          singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
          schema_version INTEGER NOT NULL,
          instance_id TEXT NOT NULL,
          generation INTEGER NOT NULL CHECK (generation >= 1),
          identity_epoch INTEGER NOT NULL CHECK (identity_epoch >= 1),
          grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1)
        ) STRICT;
        CREATE TABLE hermes_kernel_settings_schema_binding (
          registration_id TEXT PRIMARY KEY,
          schema_major INTEGER NOT NULL CHECK (schema_major >= 1),
          schema_revision INTEGER NOT NULL CHECK (schema_revision >= 1),
          schema_sha256 BLOB NOT NULL CHECK (length(schema_sha256) = 32),
          desired_revision INTEGER NOT NULL CHECK (desired_revision >= 0),
          effective_revision INTEGER NOT NULL CHECK (effective_revision >= 0)
        ) STRICT;
        INSERT INTO hermes_kernel_control_store_metadata
          (singleton, schema_version, instance_id, generation, identity_epoch, grant_epoch)
          VALUES (1, 8, '${instanceId}', 1, 1, 1);
        INSERT INTO hermes_kernel_settings_schema_binding
          (registration_id, schema_major, schema_revision, schema_sha256, desired_revision, effective_revision)
          VALUES ('registration-1', 1, 1, zeroblob(32), 2, 1);
      `],
      { encoding: 'utf8' },
    );
    assert.equal(fixture.status, 0, fixture.stderr);

    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^state=recovery_only\ncontrol_store=trustworthy\n$/);

    const migrated = spawnSync(
      'sqlite3',
      [storePath, 'SELECT schema_version FROM hermes_kernel_control_store_metadata; SELECT apply_state FROM hermes_kernel_settings_schema_binding;'],
      { encoding: 'utf8' },
    );
    assert.equal(migrated.status, 0, migrated.stderr);
    assert.equal(migrated.stdout, '9\npending_validation\n');
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('development profile enrolls exactly one local owner with a software ES256 signer', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-development-enrollment-'));
  try {
    const missingProfile = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'dev-owner', '--device-id', 'dev-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(missingProfile.status, 0);
    assert.match(missingProfile.stderr, /requires a platform adapter/);

    const enrolled = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'dev-owner', '--device-id', 'dev-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    assert.match(enrolled.stderr, /software development signer/);
    assert.match(enrolled.stdout, /development_initial_owner_enrolled=true/);
    const key = await stat(join(dataDir, 'development-device-es256.key'));
    assert.equal(key.mode & 0o777, 0o600);

    const second = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'other-owner', '--device-id', 'other-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(second.status, 0);
    assert.match(second.stderr, /initial owner is already enrolled/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('development enrollment rejects malformed IDs before creating a development key', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-development-invalid-owner-'));
  try {
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'invalid owner', '--device-id', 'dev-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /must be ASCII identifiers/);
    await assert.rejects(stat(join(dataDir, 'development-device-es256.key')));
    await assert.rejects(stat(join(dataDir, 'kernel-control-store.sqlite')));
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('development module registration validates an exact bounded descriptor and remains pending', async () => {
  const root = await mkdtemp(join(tmpdir(), 'hermes-kernel-development-registration-'));
  const dataDir = join(root, 'data');
  const descriptor = join(root, 'module-descriptor.bin');
  const text = (value) => [value.length, ...Buffer.from(value, 'ascii')];
  const capability = [0x0a, ...text('capability.read'), 0x10, 0x01, 0x18, 0x01];
  const bytes = Buffer.from([
    0x08, 0x01, 0x10, 0x01,
    0x1a, ...text('module-1'),
    0x22, ...text('owner-1'),
    0x28, 0x02,
    0x32, ...text('1'),
    0x3a, ...text('build-1'),
    0x4a, capability.length, ...capability,
  ]);
  try {
    await writeFile(descriptor, bytes, { mode: 0o600 });
    const enrolled = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'initial-owner-enroll', '--owner-id', 'dev-owner', '--device-id', 'dev-device'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const missingProfile = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'module-register', '--descriptor', descriptor],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(missingProfile.status, 0);
    assert.match(missingProfile.stderr, /requires --development-profile/);
    const registered = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'module-register', '--descriptor', descriptor],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(registered.status, 0, registered.stderr);
    assert.match(registered.stdout, /^module_registration_id=[0-9a-f]{32}\nmodule_registration_state=pending\n$/);
    const registrationId = registered.stdout.match(/^module_registration_id=([0-9a-f]{32})$/m)?.[1];
    assert.ok(registrationId);
    const approved = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'module-approve', '--registration-id', registrationId, '--capability', 'capability.read'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(approved.status, 0, approved.stderr);
    assert.match(approved.stdout, new RegExp(`^module_registration_id=${registrationId}\\nmodule_grant_epoch=2\\neffective_capability_count=1\\n$`));
    const attested = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'module-external-attest', '--registration-id', registrationId, '--runtime-id', 'compose-postgres', '--runtime-generation', '1', '--distribution-sha256', 'a'.repeat(64)],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(attested.status, 0, attested.stderr);
    assert.match(attested.stdout, new RegExp(`^module_registration_id=${registrationId}\\nexternal_runtime_id=compose-postgres\\nexternal_runtime_generation=1\\nexternal_runtime_grant_epoch=2\\n$`));
    const attestationReplay = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'module-external-attest', '--registration-id', registrationId, '--runtime-id', 'compose-postgres', '--runtime-generation', '1', '--distribution-sha256', 'a'.repeat(64)],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(attestationReplay.status, 0);
    assert.match(attestationReplay.stderr, /StaleExternalRuntimeAttestation/);
    const suspended = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'module-transition', '--registration-id', registrationId, '--state', 'suspended'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(suspended.status, 0, suspended.stderr);
    assert.match(suspended.stdout, new RegExp(`^module_registration_id=${registrationId}\\nmodule_registration_state=suspended\\nmodule_grant_epoch=3\\n$`));
    const status = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--development-profile', '--data-dir', dataDir, 'module-status', '--registration-id', registrationId],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(status.status, 0, status.stderr);
    assert.match(status.stdout, new RegExp(`^module_registration_id=${registrationId}\\nmodule_registration_state=suspended\\nmodule_grant_epoch=3\\neffective_capability_count=0\\nexternal_runtime_attested=false\\n$`));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('Kernel uses the OS-standard local data directory when no override is supplied', async () => {
  const home = await mkdtemp(join(tmpdir(), 'hermes-kernel-home-'));
  try {
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', 'status'],
      {
        cwd: new URL('../../', import.meta.url),
        encoding: 'utf8',
        env: {
          ...process.env,
          HOME: home,
          CARGO_HOME: process.env.CARGO_HOME ?? '/Users/avm/.cargo',
          RUSTUP_HOME: process.env.RUSTUP_HOME ?? '/Users/avm/.rustup',
        },
      },
    );

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^state=recovery_only\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(home, { recursive: true, force: true });
  }
});

test('Kernel rejects a relative data directory without fallback', () => {
  const result = spawnSync(
    'cargo',
    ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', 'relative-store', 'status'],
    { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
  );

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /data directory must be an absolute path/);
});

test('Kernel rejects a second process for the same data directory', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-lock-'));
  const first = spawn(
    'cargo',
    ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'hold-lock'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('first Kernel did not become ready')), 15_000);
      first.stdout.once('data', () => {
        clearTimeout(timeout);
        resolve();
      });
      first.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`first Kernel exited before readiness: ${code}`));
      });
    });
    const second = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(second.status, 0);
    assert.match(second.stderr, /data directory is already in use/);
  } finally {
    first.kill('SIGTERM');
    await waitForChildExit(first);
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Kernel rejects a data directory that is readable by other users', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-permissions-'));
  try {
    await chmod(dataDir, 0o755);
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /data directory must be owner-private/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Kernel rejects symlinked data and control-store paths', async () => {
  const root = await mkdtemp(join(tmpdir(), 'hermes-kernel-symlink-'));
  const target = join(root, 'target');
  const linkedDataDir = join(root, 'linked-data');
  const dataDir = join(root, 'data');
  try {
    await writeFile(target, 'not a store');
    await symlink(target, linkedDataDir);
    const linkedDirectory = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', linkedDataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.notEqual(linkedDirectory.status, 0);
    assert.match(linkedDirectory.stderr, /data directory must not be a symlink/);

    await writeFile(join(root, 'store-target'), 'not a sqlite database');
    await mkdir(dataDir, { mode: 0o700 });
    await symlink(join(root, 'store-target'), join(dataDir, 'kernel-control-store.sqlite'));
    const linkedStore = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(linkedStore.status, 0, linkedStore.stderr);
    assert.match(linkedStore.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('Kernel keeps a corrupt control store in restricted recovery_only', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-corrupt-'));
  try {
    await writeFile(join(dataDir, 'kernel-control-store.sqlite'), 'not a sqlite database', { mode: 0o600 });
    const result = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Installation anchor prevents a missing Store from becoming a new instance', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-anchor-'));
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  try {
    const first = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(first.status, 0, first.stderr);
    await stat(join(dataDir, '.hermes-installation-anchor-v1'));
    await rm(storePath);

    const second = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(second.status, 0, second.stderr);
    assert.match(second.stdout, /^state=recovery_only\ncontrol_store=unavailable\n$/);
    await assert.rejects(stat(storePath));
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Offline restore requires confirmation, verifies the anchor and advances fences', async () => {
  const sourceDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-restore-source-'));
  const targetDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-restore-target-'));
  try {
    const sourceBootstrap = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', sourceDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(sourceBootstrap.status, 0, sourceBootstrap.stderr);
    await copyFile(
      join(sourceDir, '.hermes-installation-anchor-v1'),
      join(targetDir, '.hermes-installation-anchor-v1'),
    );
    await writeFile(join(targetDir, 'kernel-control-store.sqlite'), 'corrupt store', { mode: 0o600 });

    const unconfirmed = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', targetDir, 'control-store', 'restore', '--source', join(sourceDir, 'kernel-control-store.sqlite')],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'no\n' },
    );
    assert.notEqual(unconfirmed.status, 0);
    assert.match(unconfirmed.stderr, /operation was not confirmed/);

    const restored = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', targetDir, 'control-store', 'restore', '--source', join(sourceDir, 'kernel-control-store.sqlite')],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESTORE\n' },
    );
    assert.equal(restored.status, 0, restored.stderr);
    assert.match(restored.stdout, /offline_control_store_operation=restore\n/);
    assert.match(restored.stdout, /control_store_generation=2\n$/);

    const status = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', targetDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(status.status, 0, status.stderr);
    assert.match(status.stdout, /^state=recovery_only\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(sourceDir, { recursive: true, force: true });
    await rm(targetDir, { recursive: true, force: true });
  }
});

test('Offline reset requires explicit data directory and confirmation', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-reset-'));
  try {
    const bootstrap = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(bootstrap.status, 0, bootstrap.stderr);

    const reset = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'control-store', 'reset'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESET\n' },
    );
    assert.equal(reset.status, 0, reset.stderr);
    assert.match(reset.stdout, /offline_control_store_operation=reset\n/);
    assert.match(reset.stdout, /control_store_generation=2\n$/);

    const missingDataDir = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', 'control-store', 'reset'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESET\n' },
    );
    assert.notEqual(missingDataDir.status, 0);
    assert.match(missingDataDir.stderr, /require an explicit absolute --data-dir/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Offline reset replaces a corrupt control store only through a new-instance ceremony', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-reset-corrupt-'));
  const anchorPath = join(dataDir, '.hermes-installation-anchor-v1');
  const storePath = join(dataDir, 'kernel-control-store.sqlite');
  try {
    const bootstrap = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(bootstrap.status, 0, bootstrap.stderr);
    const oldAnchor = await readFile(anchorPath, 'utf8');
    await writeFile(storePath, 'corrupt store', { mode: 0o600 });

    const reset = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'control-store', 'reset'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8', input: 'RESET\n' },
    );
    assert.equal(reset.status, 0, reset.stderr);
    assert.match(reset.stdout, /reset_mode=new_instance\n/);
    assert.match(reset.stdout, /control_store_generation=1\n$/);
    assert.notEqual(await readFile(anchorPath, 'utf8'), oldAnchor);

    const status = spawnSync(
      'cargo',
      ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'status'],
      { cwd: new URL('../../', import.meta.url), encoding: 'utf8' },
    );
    assert.equal(status.status, 0, status.stderr);
    assert.match(status.stdout, /^state=recovery_only\ncontrol_store=trustworthy\n$/);
  } finally {
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Recovery IPC uses a bounded protobuf length-delimited frame without waiting for EOF', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-ipc-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });

    const response = await new Promise((resolve, reject) => {
      const socket = createConnection(socketPath);
      const chunks = [];
      socket.once('connect', () => socket.write(Buffer.from([0x02, 0x0a, 0x00])));
      socket.on('data', (chunk) => chunks.push(chunk));
      socket.once('end', () => resolve(Buffer.concat(chunks)));
      socket.once('error', reject);
    });

    assert.ok(response.length > 1);
    assert.equal(response[0], response.length - 1);
    assert.deepEqual([...response.subarray(1, 7)], [0x0a, 0x08, 0x0a, 0x06, 0x08, 0x03]);
  } finally {
    kernel.kill('SIGTERM');
    await waitForChildExit(kernel);
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Kernel handles SIGTERM by closing the recovery socket within a bounded interval', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-sigterm-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });
    kernel.kill('SIGTERM');
    await Promise.race([
      waitForChildExit(kernel),
      new Promise((_, reject) => setTimeout(() => reject(new Error('Kernel did not stop after SIGTERM')), 2_000)),
    ]);
    await assert.rejects(stat(socketPath));
  } finally {
    if (kernel.exitCode === null && kernel.signalCode === null) {
      kernel.kill('SIGKILL');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Recovery IPC protocol errors remove the private socket before Kernel exits', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-ipc-error-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });
    const socket = createConnection(socketPath);
    await new Promise((resolve, reject) => {
      socket.once('connect', () => {
        socket.end(Buffer.from([0x80, 0x80, 0x80, 0x80, 0x80]));
        resolve();
      });
      socket.once('error', reject);
    });
    await waitForChildExit(kernel);
    assert.notEqual(kernel.exitCode, 0);
    await assert.rejects(stat(socketPath));
  } finally {
    if (kernel.exitCode === null && kernel.signalCode === null) {
      kernel.kill('SIGKILL');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('Recovery IPC exports through SQLite backup and honours bounded shutdown', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'hermes-kernel-export-'));
  const kernel = spawn(
    'cargo',
    ['run', '-q', '-p', 'hermes-kernel', '--', '--data-dir', dataDir, 'serve'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  try {
    const socketPath = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Kernel did not publish its recovery socket')), 15_000);
      kernel.stdout.on('data', (chunk) => {
        const match = chunk.toString().match(/^recovery_socket=(.+)$/m);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });
      kernel.once('exit', (code) => {
        clearTimeout(timeout);
        reject(new Error(`Kernel exited before IPC readiness: ${code}`));
      });
    });
    const request = (frame) => new Promise((resolve, reject) => {
      const socket = createConnection(socketPath);
      const chunks = [];
      socket.once('connect', () => socket.write(frame));
      socket.on('data', (chunk) => chunks.push(chunk));
      socket.once('end', () => resolve(Buffer.concat(chunks)));
      socket.once('error', reject);
    });

    const exportResponse = await request(Buffer.from([0x02, 0x1a, 0x00]));
    assert.equal(exportResponse[1], 0x1a);
    const exportedStore = join(dataDir, 'recovery', 'control-store.sqlite');
    assert.ok((await stat(exportedStore)).size > 0);

    const shutdownResponse = await request(Buffer.from([0x02, 0x22, 0x00]));
    assert.deepEqual([...shutdownResponse], [0x02, 0x22, 0x00]);
    await waitForChildExit(kernel);
  } finally {
    if (kernel.exitCode === null) {
      kernel.kill('SIGTERM');
      await waitForChildExit(kernel);
    }
    await rm(dataDir, { recursive: true, force: true });
  }
});
