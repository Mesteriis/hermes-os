import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

const ADR_PATH = new URL(
  'docs/adr/ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md',
  REPOSITORY_ROOT,
);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);
const INVENTORY_PATH = new URL(
  'architecture/communications-settings-reconstruction.json',
  BACKEND_ROOT,
);

test('reply suggestion agreement keeps domain workflow engine and integration separate', async () => {
  const [adr, policySource, inventorySource] = await Promise.all([
    readFile(ADR_PATH, 'utf8'),
    readFile(POLICY_PATH, 'utf8'),
    readFile(INVENTORY_PATH, 'utf8'),
  ]);
  const policy = JSON.parse(policySource);
  const inventory = JSON.parse(inventorySource);
  const slices = new Map(inventory.slices.map((slice) => [slice.gate, slice]));

  assert.equal(policy.aiContext.firstConcreteUseCase, 'communication_reply_suggestion_v1');
  assert.equal(policy.aiContext.firstConcreteUseCaseAdr, 'ADR-0353');
  assert.equal(
    policy.aiContext.communicationsPrivateContentHandoff,
    'event_backed_target_bound_blob_custody_v1',
  );
  assert.equal(policy.aiContext.clientContentTicketReuseForWorkflowEnabled, false);
  assert.equal(policy.aiContext.inferenceOwnerRole, 'engine');
  assert.equal(policy.aiContext.firstProviderIntegration, 'ollama_ai_provider_v1');
  assert.equal(policy.aiContext.firstProviderEgressPolicy, 'local_loopback_only');
  assert.equal(policy.aiContext.callerSelectedProviderOrModelEnabled, false);
  assert.equal(policy.aiContext.providerImplementationInsideInferenceOwnerEnabled, false);

  assert.deepEqual(slices.get('communications_ai_context_source_v1'), {
    gate: 'communications_ai_context_source_v1',
    role: 'domain',
    owner: 'communications',
    state: 'planned',
    dependsOn: ['communications_content_read_v1', 'nats_data_plane_v1', 'blob_v1'],
  });
  assert.deepEqual(slices.get('ai_inference_v1'), {
    gate: 'ai_inference_v1',
    role: 'engine',
    owner: 'ai',
    state: 'planned',
    dependsOn: [
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
      'ollama_ai_provider_v1',
    ],
  });
  assert.deepEqual(slices.get('ollama_ai_provider_v1'), {
    gate: 'ollama_ai_provider_v1',
    role: 'integration',
    owner: 'ollama',
    state: 'planned',
    dependsOn: [
      'capability_routed_module_request_rpc_v1',
      'managed_integration_settings_apply_v1',
    ],
  });
  assert.deepEqual(slices.get('communication_reply_suggestion_v1').dependsOn, [
    'communications_ai_context_source_v1',
    'ai_inference_v1',
    'capability_routed_module_request_rpc_v1',
    'blob_v1',
  ]);

  assert.match(adr, /hermes-communications-ai-source-api/);
  assert.match(adr, /hermes-ai-contracts/);
  assert.match(adr, /hermes-communication-reply-suggestion-api/);
  assert.match(adr, /hermes-ollama-ai-api/);
  assert.match(adr, /Client content ticket из ADR-0315 не используется/);
  assert.match(adr, /Mock or canned response не\s+является production evidence/);
  assert.doesNotMatch(
    adr,
    /Gateway (?:fetches|reads) (?:the )?message body|generic ai context workflow/i,
  );
});
