import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const compose = ['compose', '-f', 'development/compose.yaml', 'exec', '-T'];

async function run(service, command) {
  const result = await execFileAsync('docker', [...compose, service, ...command], {
    encoding: 'utf8',
  });
  return result.stdout;
}

try {
  const [postgres, pgbouncer, nats] = await Promise.all([
    run('postgres', ['psql', '-U', 'hermes_development', '-d', 'hermes_development', '-tAc', 'SELECT 1']),
    run('postgres', ['psql', 'postgres://hermes_development@pgbouncer:6432/hermes_development', '-tAc', 'SELECT 1']),
    run('nats', ['wget', '-q', '-O', '-', 'http://127.0.0.1:8222/healthz']),
  ]);
  if (postgres.trim() !== '1') throw new Error('PostgreSQL query did not return 1');
  if (pgbouncer.trim() !== '1') throw new Error('PgBouncer query did not return 1');
  if (JSON.parse(nats).status !== 'ok') throw new Error('NATS health response is not ok');
  process.stdout.write('development-platform-smoke: ok\n');
} catch (error) {
  process.stderr.write(`development-platform-smoke: failed: ${error.message}\n`);
  process.exitCode = 1;
}
