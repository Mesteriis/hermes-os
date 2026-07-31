import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('recipient suggestion agreement separates source ownership from workflow decisions', async () => {
  const [adr, inventorySource, policySource, workspace, apiManifest, api, protocol, coreManifest, core] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0365-communication-recipient-suggestion-workflow-and-source-boundary.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-recipient-suggestion-api/proto/hermes/communication_recipient_suggestion/v1/recipient_suggestion.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-recipient-suggestion-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-recipient-suggestion-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const source = inventory.slices.find(({ gate }) => gate === 'communications_recipient_source_v1');
  const workflow = inventory.slices.find(
    ({ gate }) => gate === 'communication_recipient_suggestion_v1',
  );

  assert.deepEqual(source, {
    gate: 'communications_recipient_source_v1',
    role: 'domain',
    owner: 'communications',
    state: 'planned',
    dependsOn: ['communications_canonical_read_v2', 'blob_v1', 'nats_data_plane_v1'],
  });
  assert.deepEqual(workflow, {
    gate: 'communication_recipient_suggestion_v1',
    role: 'workflow',
    owner: 'communication_recipient_suggestion',
    state: 'planned',
    dependsOn: ['communications_recipient_source_v1', 'client_gateway_v1', 'blob_v1'],
  });
  for (const unit of [
    'hermes-communications-recipient-source-api',
    'hermes-communication-recipient-suggestion-api',
    'hermes-communication-recipient-suggestion-core',
    'hermes-communication-recipient-suggestion-persistence',
    'hermes-communication-recipient-suggestion-runtime',
    'hermes-communication-recipient-suggestion-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /какую bounded[\s\S]*организационную роль/);
  assert.match(adr, /accounting_or_bookkeeping/);
  assert.match(adr, /legal_counsel/);
  assert.match(adr, /project_stakeholder/);
  assert.match(adr, /target-bound Blob/);
  assert.match(adr, /общий replayable SSE/);
  assert.match(adr, /Kernel\/Gateway не компилируют/);
  assert.match(adr, /Состояние реализации: planned/);
  assert.doesNotMatch(adr, /Communications (?:owns|владеет) recipient decision|generic `execute\(any\)`/i);

  assert.equal(
    policy.implementation.currentSlice,
    'communication_recipient_suggestion_contract_core_v1',
  );
  assert.match(workspace, /"src\/communication-recipient-suggestion-api"/);
  assert.match(workspace, /"src\/communication-recipient-suggestion-core"/);
  assert.match(apiManifest, /owner = "communication_recipient_suggestion"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /owner = "communication_recipient_suggestion"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(api, /COMMUNICATION_RECIPIENT_SUGGESTION_CAPABILITY_ID_V1/);
  assert.match(protocol, /COMMUNICATION_RECIPIENT_ROLE_ACCOUNTING_OR_BOOKKEEPING/);
  assert.match(protocol, /COMMUNICATION_RECIPIENT_ROLE_LEGAL_COUNSEL/);
  assert.match(protocol, /COMMUNICATION_RECIPIENT_ROLE_PROJECT_STAKEHOLDER/);
  assert.doesNotMatch(
    protocol,
    /email_address|contact_id|person_id|organization_id|provider_id|account_id|model_id|prompt|source_body|map</,
  );
  assert.match(core, /evaluate_communication_recipient_candidates_v1/);
  assert.match(core, /allows_empty_candidate_list_without_fabricating_a_recipient/);
  assert.match(core, /SourceDigestMismatch/);
  assert.doesNotMatch(
    core,
    /hermes_ai|ollama|communications_domain|communication_explanation|communication_reply_suggestion/,
  );
  assert.ok(
    policy.implementation.ownerInventory.workflows.includes(
      'communication_recipient_suggestion',
    ),
  );
  assert.ok(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'communication.recipient-suggestion.v1',
    ),
  );
});
