import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const COMMUNICATIONS_INGRESS_ROOT = new URL('src/communications-ingress/src/', BACKEND_ROOT);
const COMMUNICATIONS_ATTACHMENT_CONTRACT_ROOT = new URL('src/communications-attachment-contract/src/', BACKEND_ROOT);
const COMMUNICATIONS_API_ROOT = new URL('src/communications-api/src/', BACKEND_ROOT);
const COMMUNICATIONS_DOMAIN_ROOT = new URL('src/communications-domain/src/', BACKEND_ROOT);
const COMMUNICATIONS_PERSISTENCE_ROOT = new URL('src/communications-persistence/src/', BACKEND_ROOT);
const COMMUNICATIONS_RUNTIME_ROOT = new URL('src/communications-runtime/src/', BACKEND_ROOT);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);
const FORBIDDEN_INTEGRATION_IMPLEMENTATIONS = [
  'hermes_mail_',
  'hermes_telegram_',
  'hermes_whatsapp_',
  'hermes_zulip_',
];
const FORBIDDEN_DOMAIN_IMPLEMENTATIONS = [
  ...FORBIDDEN_INTEGRATION_IMPLEMENTATIONS,
  'hermes_blob_',
];

test('Communications domain does not import integration or Blob implementations', async () => {
  const sources = await rustSources(COMMUNICATIONS_DOMAIN_ROOT);

  assert.ok(sources.length > 0);
  for (const source of sources) {
    for (const implementation of FORBIDDEN_DOMAIN_IMPLEMENTATIONS) {
      assert.ok(
        !source.content.includes(implementation),
        `${source.path} imports forbidden owner implementation ${implementation}`,
      );
    }
  }
});

test('Communications remains the exact domain owner after Engine admission', async () => {
  const [
    policySource,
    ingressSources,
    attachmentContractSources,
    apiSources,
    domainSources,
    persistenceSources,
    runtimeSources,
  ] = await Promise.all([
    readFile(POLICY_PATH, 'utf8'),
    rustSources(COMMUNICATIONS_INGRESS_ROOT),
    rustSources(COMMUNICATIONS_ATTACHMENT_CONTRACT_ROOT),
    rustSources(COMMUNICATIONS_API_ROOT),
    rustSources(COMMUNICATIONS_DOMAIN_ROOT),
    rustSources(COMMUNICATIONS_PERSISTENCE_ROOT),
    rustSources(COMMUNICATIONS_RUNTIME_ROOT),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'attachment_security_engine_v1');
  assert.deepEqual(policy.implementation.ownerInventory, {
    domains: ['communications'],
    integrations: [],
    workflows: [],
    engines: ['attachment_security'],
    businessCapabilities: [
      'attachment_security.blob.v1',
      'attachment_security.candidate.observe.v1',
      'attachment_security.communications-state.observe.v1',
      'attachment_security.storage.v1',
      'attachment_security.verdict.publish.v1',
      'communications.attachment.blob-admission.observe.v1',
      'communications.attachment.safety-verdict.observe.v1',
      'communications.blob.v1',
      'communications.events.v1',
      'communications.observe.v1',
      'communications.query.v1',
      'communications.search.index.v1',
      'communications.storage.v1',
    ],
  });
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter((entry) => entry.role === 'integration')
      .map((entry) => entry.name),
    [],
    'Engine admission must not carry integration build units in production inventory',
  );

  for (const source of [
    ...ingressSources,
    ...attachmentContractSources,
    ...apiSources,
    ...domainSources,
    ...persistenceSources,
    ...runtimeSources,
  ]) {
    for (const implementation of FORBIDDEN_INTEGRATION_IMPLEMENTATIONS) {
      assert.ok(
        !source.content.includes(implementation),
        `${source.path} imports forbidden provider implementation ${implementation}`,
      );
    }
    assert.ok(!source.content.includes('references/backend-legacy'), `${source.path} uses legacy source`);
    assert.ok(!source.content.includes('references/'), `${source.path} uses reference fallback`);
    assert.doesNotMatch(source.content, /\b(?:HashMap|BTreeMap|serde_json)\b/, `${source.path} uses a generic owner payload shape`);
  }

  const runtime = runtimeSources.map((source) => source.content).join('\n');
  assert.match(runtime, /consume_next_observation_v1/);
  assert.match(runtime, /relay_domain_outbox_once/);
});

test('Communications attachment schemas have one contract owner and no compatibility facade', async () => {
  const [ingress, api, attachment] = await Promise.all([
    readFile(new URL('src/communications-ingress/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-attachment-contract/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.doesNotMatch(ingress, /attachment_(?:blob|safety|anchor)_v1/);
  assert.doesNotMatch(api, /attachment_wire/);
  assert.match(attachment, /pub mod blob_admission_v1/);
  assert.match(attachment, /pub mod safety_verdict_v1/);
  assert.match(attachment, /pub mod anchor_recorded_v1/);
  assert.match(attachment, /pub mod lifecycle_v1/);
});

test('Communications custody transfer keeps source receipts private and uses only the Blob client port', async () => {
  const [persistenceSources, runtimeSources] = await Promise.all([
    rustSources(COMMUNICATIONS_PERSISTENCE_ROOT),
    rustSources(COMMUNICATIONS_RUNTIME_ROOT),
  ]);
  const custody = persistenceSources.find((source) => source.path.endsWith('/custody_transfer.rs'));
  assert.ok(custody, 'Communications custody persistence is required');
  assert.match(custody.content, /communications_body_custody_transfers/);
  assert.match(custody.content, /source_custody_proof/);

  const runtime = runtimeSources.map((source) => source.content).join('\n');
  assert.match(runtime, /request_managed_blob_custody_transfer/);
  assert.doesNotMatch(runtime, /hermes_blob_service|BlobContentLifecycleStore/);

  for (const source of runtimeSources.filter((source) => source.path.includes('/query'))) {
    assert.doesNotMatch(source.content, /source_blob_ref|source_custody_proof/);
  }
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
