import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const COMMUNICATIONS_DOMAIN_ROOT = new URL('src/communications-domain/src/', BACKEND_ROOT);
const COMMUNICATIONS_PERSISTENCE_ROOT = new URL('src/communications-persistence/src/', BACKEND_ROOT);
const COMMUNICATIONS_RUNTIME_ROOT = new URL('src/communications-runtime/src/', BACKEND_ROOT);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);
const FORBIDDEN_OWNER_IMPLEMENTATIONS = [
  'hermes_mail_',
  'hermes_telegram_',
  'hermes_whatsapp_',
  'hermes_zulip_',
  'hermes_blob_',
];

test('Communications domain does not import integration or Blob implementations', async () => {
  const sources = await rustSources(COMMUNICATIONS_DOMAIN_ROOT);

  assert.ok(sources.length > 0);
  for (const source of sources) {
    for (const implementation of FORBIDDEN_OWNER_IMPLEMENTATIONS) {
      assert.ok(
        !source.content.includes(implementation),
        `${source.path} imports forbidden owner implementation ${implementation}`,
      );
    }
  }
});

test('Communications first owner inventory is exact and owner-local implementations stay provider-free', async () => {
  const [policySource, domainSources, persistenceSources, runtimeSources] = await Promise.all([
    readFile(POLICY_PATH, 'utf8'),
    rustSources(COMMUNICATIONS_DOMAIN_ROOT),
    rustSources(COMMUNICATIONS_PERSISTENCE_ROOT),
    rustSources(COMMUNICATIONS_RUNTIME_ROOT),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'first_owner_v1');
  assert.deepEqual(policy.implementation.ownerInventory, {
    domains: ['communications'],
    integrations: [],
    workflows: [],
    engines: [],
    businessCapabilities: [
      'communications.blob.v1',
      'communications.events.v1',
      'communications.observe.v1',
      'communications.query.v1',
      'communications.storage.v1',
    ],
  });

  for (const source of [...domainSources, ...persistenceSources, ...runtimeSources]) {
    for (const implementation of FORBIDDEN_OWNER_IMPLEMENTATIONS) {
      assert.ok(
        !source.content.includes(implementation),
        `${source.path} imports forbidden provider implementation ${implementation}`,
      );
    }
    assert.ok(!source.content.includes('references/backend-legacy'), `${source.path} uses legacy source`);
    assert.ok(!source.content.includes('references/'), `${source.path} uses reference fallback`);
  }

  const runtime = runtimeSources.map((source) => source.content).join('\n');
  assert.match(runtime, /consume_next_observation_v1/);
  assert.match(runtime, /relay_domain_outbox_once/);
});

async function rustSources(directory) {
  const entries = await readdir(directory, { recursive: true, withFileTypes: true });
  return Promise.all(entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs'))
    .map(async (entry) => {
      const parent = entry.parentPath;
      const path = parent.startsWith(directory.pathname)
        ? join(parent, entry.name)
        : join(directory.pathname, parent, entry.name);
      return { path, content: await readFile(path, 'utf8') };
    }));
}
