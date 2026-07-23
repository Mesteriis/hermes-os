import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
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
  const policy = JSON.parse(
    readFileSync(join(backendRoot, 'architecture', 'policy.json'), 'utf8'),
  );
  const communicationsPackages = new Set([
    'hermes-communications-api',
    'hermes-communications-domain',
    'hermes-communications-ingress',
    'hermes-communications-persistence',
    'hermes-communications-runtime',
  ]);

  for (const owner of ['mail', 'telegram', 'zulip', 'whatsapp']) {
    const integrationPackages = policy.implementation.productionPackages.filter(
      (entry) => entry.role === 'integration' && entry.owner === owner,
    );
    assert.ok(integrationPackages.length > 0, `missing ${owner} integration inventory`);

    for (const integrationPackage of integrationPackages) {
      const communicationsDependencies = (policy.implementation.workspaceDependencyAllowlist[integrationPackage.name] ?? [])
        .map((dependency) => dependency.name)
        .filter((name) => communicationsPackages.has(name));
      assert.deepEqual(
        communicationsDependencies,
        communicationsDependencies.length === 0 ? [] : ['hermes-communications-ingress'],
        `${integrationPackage.name} has a direct Communications implementation edge`,
      );
    }
  }
});
