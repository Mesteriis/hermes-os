import { execFile, spawn } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { chmod, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const toolchain = process.argv[2] || '1.97.0';
const project = `hermes-storage-authenticated-${process.pid}`;
const compose = ['compose', '--project-name', project, '-f', 'development/authenticated/compose.yaml'];
const focusedTest = process.env.HERMES_STORAGE_AUTHENTICATED_TEST_FILTER?.trim();
const managedTest = process.env.HERMES_STORAGE_MANAGED_TEST_FILTER?.trim();
const keepContour = process.env.HERMES_STORAGE_KEEP_CONTOUR === '1';
const authenticatedTests = [
  'authenticated_revoke_fences_the_real_pool_and_postgres_role',
  'authenticated_runtime_revokes_the_exact_staged_binding_through_vault',
  'authenticated_admin_console_requires_a_file_backed_credential',
  'authenticated_pgbouncer_reloads_the_storage_owned_database_include',
  'authenticated_runtime_readiness_accepts_the_resolved_platform_credential',
  'authenticated_runtime_applies_the_kernel_staged_binding_to_pgbouncer',
  'authenticated_inherited_runtime_bootstraps_real_platform_services',
  'authenticated_runtime_accepts_only_the_vault_delivered_role_credential',
  'authenticated_runtime_applies_the_exact_bound_owner_migration_bundle',
  'authenticated_runtime_bootstraps_the_platform_postgres_schema',
  'authenticated_runtime_reconciles_roles_for_the_kernel_staged_binding',
];

async function run(command, args, options = {}) {
  if (process.env.HERMES_STORAGE_UNBUFFERED === '1') {
    await new Promise((resolve, reject) => {
      const child = spawn(command, args, { stdio: 'inherit', ...options });
      child.once('error', reject);
      child.once('exit', (code) => {
        if (code === 0) resolve();
        else reject(new Error(`${command} exited with ${code ?? 'signal'}`));
      });
    });
    return;
  }
  await execFileAsync(command, args, { encoding: 'utf8', ...options });
}

async function create_secret_files() {
  const directory = await mkdtemp(join(tmpdir(), 'hermes-storage-auth-'));
  await chmod(directory, 0o700);
  const postgresPath = await create_secret_file(directory, 'postgres-admin-password');
  const pgbouncerPath = await create_secret_file(directory, 'pgbouncer-admin-password');
  const runtimeDirectory = join(directory, 'runtime');
  const storageDirectory = join(runtimeDirectory, 'storage');
  const pgbouncerDirectory = join(storageDirectory, 'pgbouncer');
  const pgbouncerAuthDirectory = join(pgbouncerDirectory, 'auth');
  await mkdir(pgbouncerAuthDirectory, { recursive: true, mode: 0o700 });
  await chmod(runtimeDirectory, 0o700);
  await chmod(storageDirectory, 0o700);
  await chmod(pgbouncerDirectory, 0o700);
  await chmod(pgbouncerAuthDirectory, 0o700);
  const databasesPath = join(pgbouncerDirectory, 'databases.ini');
  await writeFile(databasesPath, '[databases]\n', { mode: 0o600 });
  await chmod(databasesPath, 0o600);
  return {
    directory,
    postgresPath,
    pgbouncerPath,
    storageDirectory,
    pgbouncerDirectory,
    pgbouncerAuthDirectory,
    databasesPath,
    authPath: join(pgbouncerAuthDirectory, 'users.txt'),
  };
}

async function create_secret_file(directory, name) {
  const path = join(directory, name);
  await writeFile(path, `${randomBytes(32).toString('hex')}\n`, { mode: 0o600 });
  await chmod(path, 0o600);
  return path;
}

function compose_environment(secrets) {
  return {
    ...process.env,
    HERMES_STORAGE_POSTGRES_SECRET_FILE: secrets.postgresPath,
    HERMES_STORAGE_PGBOUNCER_SECRET_FILE: secrets.pgbouncerPath,
    HERMES_STORAGE_PGBOUNCER_DATABASES_DIRECTORY: secrets.pgbouncerDirectory,
    HERMES_STORAGE_PGBOUNCER_AUTH_DIRECTORY: secrets.pgbouncerAuthDirectory,
    HERMES_STORAGE_PGBOUNCER_RUNTIME_UID: String(process.getuid()),
  };
}

async function start_contour(secrets) {
  allocate_runtime_files(secrets);
  await mkdir(secrets.pgbouncerDirectory, { recursive: true, mode: 0o700 });
  await mkdir(secrets.pgbouncerAuthDirectory, { recursive: true, mode: 0o700 });
  await chmod(secrets.pgbouncerDirectory, 0o700);
  await chmod(secrets.pgbouncerAuthDirectory, 0o700);
  await writeFile(secrets.databasesPath, '[databases]\n', { mode: 0o600 });
  await chmod(secrets.databasesPath, 0o600);
  await rm(secrets.authPath, { force: true });
  await run('docker', [...compose, 'up', '--detach', '--wait'], {
    env: compose_environment(secrets),
  });
  const { stdout } = await execFileAsync('docker', [...compose, 'ps', '--quiet', 'postgres'], {
    encoding: 'utf8',
    env: compose_environment(secrets),
  });
  const container = stdout.trim();
  if (!/^[a-f0-9]{12,64}$/i.test(container)) throw new Error('authenticated PostgreSQL container is unavailable');
  secrets.postgresContainer = container;
}

function allocate_runtime_files(secrets) {
  const contour = randomBytes(8).toString('hex');
  secrets.runtimeDirectory = join(secrets.directory, `runtime-${contour}`);
  secrets.storageDirectory = join(secrets.runtimeDirectory, 'storage');
  secrets.pgbouncerDirectory = join(secrets.storageDirectory, 'pgbouncer');
  secrets.pgbouncerAuthDirectory = join(secrets.pgbouncerDirectory, 'auth');
  secrets.databasesPath = join(secrets.pgbouncerDirectory, 'databases.ini');
  secrets.authPath = join(secrets.pgbouncerAuthDirectory, 'users.txt');
}

async function stop_contour(secrets) {
  if (keepContour) return;
  await run('docker', [...compose, 'down', '--volumes', '--remove-orphans'], {
    env: compose_environment(secrets),
  });
}

async function run_conformance(secrets) {
  try {
    for (const test of focusedTest ? [focusedTest] : managedTest ? [] : authenticatedTests) {
      await start_contour(secrets);
      try {
        await run('cargo', [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'test',
    '--locked',
    '-p',
    'hermes-storage-testkit',
    '--',
    '--ignored',
    test,
    '--test-threads=1',
    ], {
    env: {
      ...process.env,
      HERMES_STORAGE_AUTHENTICATED_TEST: '1',
      HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE: secrets.pgbouncerPath,
      HERMES_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE: secrets.postgresPath,
      HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_HOST: '127.0.0.1',
      HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_PORT: '36532',
      HERMES_STORAGE_AUTHENTICATED_POSTGRES_HOST: '127.0.0.1',
      HERMES_STORAGE_AUTHENTICATED_POSTGRES_PORT: '35532',
      HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE: secrets.databasesPath,
      HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_AUTH_FILE: secrets.authPath,
      HERMES_STORAGE_AUTHENTICATED_POSTGRES_CONTAINER: secrets.postgresContainer,
    },
        });
      } finally {
        await stop_contour(secrets);
      }
    }
    if (!focusedTest) {
      await run_managed_process_conformance(secrets);
    }
  } catch (error) {
    print_test_diagnostics(error);
    throw error;
  }
}

async function run_managed_process_conformance(secrets) {
  await run('cargo', [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'build',
    '--locked',
    '-p',
    'hermes-vault-runtime',
    '-p',
    'hermes-storage-runtime',
    '-p',
    'hermes-scheduler-runtime',
    '-p',
    'hermes-communications-runtime',
    '-p',
    'hermes-blob-service',
  ]);
  for (const test of managedTest ? [managedTest] : [
    'managed_storage_binary_bootstraps_through_live_vault',
    'managed_scheduler_crash_uses_storage_control_successor_provisioning',
    'managed_communications_domain_starts_with_owner_local_storage_and_events',
  ]) {
    await start_contour(secrets);
    try {
      await run('cargo', [
    `+${toolchain}`,
    '--config',
    'build.rustc-wrapper=""',
    'test',
    '--locked',
    '-p',
    'hermes-kernel-recovery-testkit',
    '--',
    '--ignored',
    test,
    '--test-threads=1',
  ], {
    env: {
      ...authenticated_environment(secrets),
      HERMES_VAULT_RUNTIME_BIN: `${process.cwd()}/target/debug/hermes-vault-runtime`,
      HERMES_STORAGE_RUNTIME_BIN: `${process.cwd()}/target/debug/hermes-storage-runtime`,
      HERMES_SCHEDULER_RUNTIME_BIN: `${process.cwd()}/target/debug/hermes-scheduler-runtime`,
      HERMES_SCHEDULER_LIVE_NATS_ENDPOINT: 'nats://127.0.0.1:43225',
      HERMES_COMMUNICATIONS_RUNTIME_BIN: `${process.cwd()}/target/debug/hermes-communications-runtime`,
      HERMES_BLOB_SERVICE_BIN: `${process.cwd()}/target/debug/hermes-blob-service`,
      HERMES_COMMUNICATIONS_LIVE_NATS_ENDPOINT: 'nats://127.0.0.1:43225',
    },
      });
    } finally {
      await stop_contour(secrets);
    }
  }
}

function authenticated_environment(secrets) {
  return {
    ...process.env,
    HERMES_STORAGE_AUTHENTICATED_TEST: '1',
    HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE: secrets.pgbouncerPath,
    HERMES_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE: secrets.postgresPath,
    HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_HOST: '127.0.0.1',
    HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_PORT: '36532',
    HERMES_STORAGE_AUTHENTICATED_POSTGRES_HOST: '127.0.0.1',
    HERMES_STORAGE_AUTHENTICATED_POSTGRES_PORT: '35532',
    HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_DATABASES_FILE: secrets.databasesPath,
      HERMES_STORAGE_AUTHENTICATED_PGBOUNCER_AUTH_FILE: secrets.authPath,
      HERMES_STORAGE_AUTHENTICATED_POSTGRES_CONTAINER: secrets.postgresContainer,
  };
}

function print_test_diagnostics(error) {
  if (!(error && typeof error === 'object' && 'stdout' in error)) return;
  const output = String(error.stdout)
    .split('\n')
    .filter((line) => !/(password|secret)/i.test(line));
  process.stderr.write(`${output.join('\n')}\n`);
}

async function cleanup(secret) {
  if (keepContour) {
    process.stderr.write(`authenticated-storage-contour: ${project}\n`);
    process.stderr.write(`authenticated-storage-runtime: ${secret.pgbouncerDirectory}\n`);
    return;
  }
  await stop_contour(secret).catch(() => undefined);
  await rm(secret.directory, { force: true, recursive: true });
}

async function print_startup_diagnostics(secrets) {
  const result = await execFileAsync('docker', [...compose, 'logs', '--tail', '30', 'pgbouncer'], {
    encoding: 'utf8',
    env: compose_environment(secrets),
  }).catch(() => null);
  if (!result) return;
  const safeLines = `${result.stdout}${result.stderr}`
    .split('\n')
    .filter((line) => !/(password|secret)/i.test(line));
  process.stderr.write(`${safeLines.join('\n')}\n`);
}

const secret = await create_secret_files();
let stage = 'start';
try {
  stage = 'conformance';
  await run_conformance(secret);
  process.stdout.write('authenticated-storage-conformance: ok\n');
} catch (error) {
  process.stderr.write(`authenticated-storage-conformance: failed during ${stage}\n`);
  await print_startup_diagnostics(secret);
  if (error instanceof Error) process.stderr.write(`${error.message}\n`);
  process.exitCode = 1;
} finally {
  await cleanup(secret);
}
