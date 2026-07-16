import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const compose = ['compose', '-f', 'development/compose.yaml'];

async function containerIp(service) {
  const { stdout } = await execFileAsync('docker', [...compose, 'ps', '--quiet', service], {
    encoding: 'utf8',
  });
  const containerId = stdout.trim();
  if (!containerId) throw new Error(`${service} development container is not running`);
  const inspect = await execFileAsync('docker', ['inspect', '-f', '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}', containerId], {
    encoding: 'utf8',
  });
  const ip = inspect.stdout.trim();
  if (!ip) throw new Error(`${service} development container has no network address`);
  return ip;
}

try {
  const [postgresIp, natsIp] = await Promise.all([containerIp('postgres'), containerIp('nats')]);
  process.stdout.write(`HERMES_DEVELOPMENT_POSTGRES_URL=postgres://hermes_development@${postgresIp}:5432/hermes_development\n`);
  process.stdout.write(`HERMES_DEVELOPMENT_NATS_URL=nats://${natsIp}:4222\n`);
} catch (error) {
  process.stderr.write(`development platform endpoint discovery failed: ${error.message}\n`);
  process.exitCode = 1;
}
