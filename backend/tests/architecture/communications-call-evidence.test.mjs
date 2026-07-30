import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('call evidence ingress core and persistence are separate Communications domain units', async () => {
  const [ingressManifest, coreManifest, persistenceManifest, policySource] = await Promise.all([
    backendSource('src/communications-call-evidence-ingress/Cargo.toml'),
    backendSource('src/communications-call-evidence-core/Cargo.toml'),
    backendSource('src/communications-call-evidence-persistence/Cargo.toml'),
    backendSource('architecture/policy.json'),
  ]);
  const policy = JSON.parse(policySource);

  for (const manifest of [ingressManifest, coreManifest]) {
    assert.match(manifest, /role = "domain"/);
    assert.match(manifest, /owner = "communications"/);
    assert.doesNotMatch(
      manifest,
      /telegram-(?:runtime|tdlib|calls)|whatsapp-(?:runtime|host)|zoom|sqlx|kernel|gateway/,
    );
  }
  assert.match(persistenceManifest, /role = "domain"/);
  assert.match(persistenceManifest, /owner = "communications"/);
  assert.match(ingressManifest, /surface = "contract"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(coreManifest, /hermes-communications-call-evidence-ingress/);
  assert.match(persistenceManifest, /hermes-communications-call-evidence-core/);
  assert.match(persistenceManifest, /hermes-storage-protocol/);
  assert.doesNotMatch(ingressManifest, /communications-call-evidence-core/);
  assert.doesNotMatch(persistenceManifest, /telegram|whatsapp|zulip|mail-/);

  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ name }) => name.startsWith('hermes-communications-call-evidence-'))
      .map(({ name, role, owner, surface }) => `${name}:${role}:${owner}:${surface}`),
    [
      'hermes-communications-call-evidence-ingress:domain:communications:contract',
      'hermes-communications-call-evidence-core:domain:communications:implementation',
      'hermes-communications-call-evidence-persistence:domain:communications:persistence',
    ],
  );
  assert.ok(
    policy.dependencies.integrationDomainContractPackages.includes(
      'hermes-communications-call-evidence-ingress',
    ),
  );
  assert.ok(
    !policy.dependencies.integrationDomainContractPackages.includes(
      'hermes-communications-call-evidence-core',
    ),
  );
  assert.ok(
    !policy.dependencies.integrationDomainContractPackages.includes(
      'hermes-communications-call-evidence-persistence',
    ),
  );
});

test('call evidence durable observation is exact typed and locator negative', async () => {
  const [proto, ingress, envelope] = await Promise.all([
    backendSource(
      'src/communications-call-evidence-ingress/proto/hermes/communications/call_evidence/v1/call_evidence.proto',
    ),
    backendSource('src/communications-call-evidence-ingress/src/lib.rs'),
    backendSource('src/communications-call-evidence-ingress/src/envelope.rs'),
  ]);

  assert.match(proto, /message CallEvidenceObservedV1/);
  assert.match(proto, /bytes call_evidence_id = 1/);
  assert.match(proto, /bytes source_call_cursor_sha256 = 2/);
  assert.match(proto, /uint64 source_revision = 11/);
  assert.match(proto, /CallTerminalDispositionV1 terminal_disposition/);
  const protoWithoutComments = proto.replaceAll(/\/\/.*$/gm, '');
  assert.doesNotMatch(
    protoWithoutComments,
    /\b(?:account_id|call_id|chat_id|provider_user_id|username|phone_number|encryption_key|signaling|pcm|audio_bytes|transcript|credential|session|raw_json|debug_log)\b/,
  );
  assert.doesNotMatch(protoWithoutComments, /\bgoogle\.protobuf\.Any\b|\bmap\s*</);

  assert.match(ingress, /CALL_EVIDENCE_OBSERVED_CONTRACT_NAME_V1.*call_evidence_observed/s);
  assert.match(ingress, /DurableEnvelopeKindV1::Observation/);
  assert.match(ingress, /EventSubscriptionRequirementV1::Required/);
  assert.match(envelope, /partition_key: call_evidence_id\.to_vec\(\)/);
  assert.match(envelope, /source_sequence: Some\(draft\.source_revision\)/);
  assert.match(envelope, /source_call_cursor_sha256/);
  assert.doesNotMatch(envelope, /payload[\s\S]{0,600}external_(?:account|call|conversation|participant)_id/);
});

test('call evidence core is monotonic terminal and provider behavior free', async () => {
  const core = await backendSource('src/communications-call-evidence-core/src/lib.rs');

  assert.match(core, /CallEvidenceApplyOutcomeV1::Duplicate/);
  assert.match(core, /CallEvidenceApplyOutcomeV1::Stale/);
  assert.match(core, /CallEvidenceCoreErrorV1::RevisionConflict/);
  assert.match(core, /CallEvidenceCoreErrorV1::TerminalConflict/);
  assert.match(core, /CallEvidenceCoreErrorV1::StateRegression/);
  assert.doesNotMatch(
    core,
    /createCall|acceptCall|discardCall|tgcalls|TDLib|WhatsAppHost|ZoomClient|provider command/,
  );
});

test('call evidence persistence is owner local atomic and private-content negative', async () => {
  const [manifest, repository, migration] = await Promise.all([
    backendSource('src/communications-call-evidence-persistence/Cargo.toml'),
    backendSource('src/communications-call-evidence-persistence/src/repository.rs'),
    backendSource(
      'src/communications-call-evidence-persistence/migrations/0001_call_evidence.sql',
    ),
  ]);

  assert.match(manifest, /hermes-communications-call-evidence-core/);
  assert.match(manifest, /hermes-storage-protocol/);
  assert.match(repository, /existing_inbox_outcome/);
  assert.match(repository, /InboxHashConflict/);
  assert.match(repository, /FOR UPDATE/);
  assert.match(repository, /transaction\.commit\(\)/);
  assert.match(repository, /next_realtime_sequence/);
  assert.match(migration, /communications_call_evidence_inbox/);
  assert.match(migration, /communications_call_evidence_projection/);
  assert.match(migration, /communications_call_evidence_history/);
  assert.match(migration, /communications_call_evidence_realtime_frames/);
  assert.doesNotMatch(
    repository,
    /\b(?:phone_number|raw_provider|provider_call_id|provider_account_id|pcm|audio_bytes|transcript|cookie|session_store|debug_log)\b/,
  );
  assert.doesNotMatch(
    migration,
    /\b(?:phone_number|username|raw_provider|provider_call_id|provider_account_id|pcm|audio_bytes|transcript|credential|cookie|session_store|debug_log)\b/,
  );
});

test('call evidence ADR keeps the completion gate closed until live managed evidence', async () => {
  const [adr, inventorySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0349-event-backed-communications-call-evidence.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    backendSource('architecture/communications-settings-reconstruction.json'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'communications_call_evidence_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communications_call_evidence_v1',
    role: 'domain',
    owner: 'communications',
    state: 'planned',
    dependsOn: ['communications_canonical_read_v2'],
  });
  assert.match(adr, /Integration не импортирует Communications implementation или persistence/);
  assert.match(adr, /Communications не импортирует integration API, runtime, SDK или storage/);
  assert.match(adr, /ADR и static package presence сами по себе gate не открывают/);
  assert.match(adr, /live managed proof from integration outbox through NATS/);
});
