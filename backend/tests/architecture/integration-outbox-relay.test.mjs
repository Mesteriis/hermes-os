import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

test('every Communications integration relay publishes exact durable envelopes', () => {
  for (const owner of ['mail', 'telegram', 'zulip', 'whatsapp']) {
    const source = readFileSync(
      join(backendRoot, 'src', `${owner}-runtime`, 'src', 'communications_outbox.rs'),
      'utf8',
    );

    assert.match(source, /publish_exact\(permit, record\.exact_bytes\(\)\)/);
    assert.match(source, /mark_communications_outbox_published\(record\.message_id\(\), published_at_unix_seconds\)/);
  }
});

test('integration packages reach Communications only through its ingress contract', () => {
  const communicationsPackages = new Set([
    'hermes-communications-api',
    'hermes-communications-domain',
    'hermes-communications-ingress',
    'hermes-communications-persistence',
    'hermes-communications-runtime',
  ]);

  const integrationManifests = readdirSync(join(backendRoot, 'src'), { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => join(backendRoot, 'src', entry.name, 'Cargo.toml'))
    .filter((path) => {
      try {
        return readFileSync(path, 'utf8').includes('role = "integration"');
      } catch {
        return false;
      }
    });

  assert.ok(integrationManifests.length > 0, 'missing integration package manifests');
  for (const manifestPath of integrationManifests) {
    const manifest = readFileSync(manifestPath, 'utf8');
    const communicationsDependencies = [...manifest.matchAll(/^([\w-]+)\s*=.*$/gm)]
      .map((match) => match[1])
      .filter((name) => communicationsPackages.has(name));
    assert.deepEqual(
      communicationsDependencies,
      communicationsDependencies.length === 0 ? [] : ['hermes-communications-ingress'],
      `${manifestPath} has a direct Communications implementation edge`,
    );
  }
});
