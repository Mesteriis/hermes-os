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

async function backendSource(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

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

test('Communications AI source is one provider-neutral event contract unit', async () => {
  const [manifest, api, envelope, proto] = await Promise.all([
    backendSource('src/communications-ai-source-api/Cargo.toml'),
    backendSource('src/communications-ai-source-api/src/lib.rs'),
    backendSource('src/communications-ai-source-api/src/envelope.rs'),
    backendSource(
      'src/communications-ai-source-api/proto/hermes/communications/ai_source/v1/ai_source.proto',
    ),
  ]);

  assert.match(manifest, /role = "domain"/);
  assert.match(manifest, /owner = "communications"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /communication-reply-suggestion|ollama|ai-inference|sqlx|kernel|gateway/,
  );
  assert.match(api, /communication_reply_source_prepare/);
  assert.match(api, /communication_reply_source_prepared/);
  assert.match(api, /communication_reply_source_rejected/);
  assert.match(api, /communication_reply_suggestion\.source\.blob\.v1/);
  assert.match(envelope, /DurableEnvelopeV1/);
  assert.match(envelope, /target_capability: COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID_V1/);
  assert.match(envelope, /validate_envelope_v1/);
  assert.match(proto, /uint64 expected_source_revision = 3/);
  assert.match(proto, /bytes custody_transfer_source_proof = 4/);
  assert.doesNotMatch(
    `${api}\n${envelope}\n${proto}`,
    /provider_id|provider_account|provider_locator|model_id|model_key|prompt|string target_owner|string target_module|string target_capability|message_body|body_text/,
  );
});

test('Communications AI source runtime commits an owner-bound event handoff before ack', async () => {
  const [manifest, persistence, runtime, admission, eventRuntime] = await Promise.all([
    backendSource('src/communications-runtime/Cargo.toml'),
    backendSource('src/communications-persistence/src/ai_source.rs'),
    backendSource('src/communications-runtime/src/ai_source.rs'),
    backendSource('src/communications-runtime/src/admission.rs'),
    backendSource('src/communications-runtime/src/event_runtime.rs'),
  ]);

  assert.match(manifest, /hermes-communications-ai-source-api/);
  assert.match(persistence, /communications_event_inbox/);
  assert.match(persistence, /communications_domain_outbox/);
  assert.match(persistence, /canonical_revision/);
  assert.match(persistence, /last_evidence_id/);
  assert.match(persistence, /body_blob_reference_id/);
  assert.match(persistence, /body_blob_declared_bytes/);
  assert.match(persistence, /body_blob_sha256/);
  assert.match(persistence, /transaction\s*\.commit\(\)/);
  assert.match(runtime, /payload\.logical_owner_id != logical_human_owner_id/);
  assert.match(runtime, /COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(runtime, /COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_MODULE_ID_V1/);
  assert.match(runtime, /COMMUNICATION_REPLY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1/);
  assert.match(runtime, /COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID/);
  assert.match(admission, /communications_ai_source_blob_capability_v1/);
  assert.match(admission, /communications_ai_source_capability_v1/);
  assert.match(eventRuntime, /communication_reply_source_prepare_contract_reference_v1/);
  assert.match(eventRuntime, /CommunicationsConsumerV1::AiSourcePrepare/);

  const persisted = runtime.indexOf('.persist_ai_source_result(');
  const acknowledged = runtime.indexOf('delivery.acknowledge()', persisted);
  assert.ok(persisted >= 0);
  assert.ok(acknowledged > persisted);
  assert.doesNotMatch(
    `${persistence}\n${runtime}`,
    /provider_id|provider_account|provider_locator|model_id|model_key|prompt|ollama/,
  );
});

test('AI public contracts are one concrete provider-neutral engine unit', async () => {
  const [manifest, api, validation, proto] = await Promise.all([
    backendSource('src/ai-contracts/Cargo.toml'),
    backendSource('src/ai-contracts/src/lib.rs'),
    backendSource('src/ai-contracts/src/validation.rs'),
    backendSource('src/ai-contracts/proto/hermes/ai/contracts/v1/ai.proto'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "ai"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(api, /communication_reply_suggestion_inference/);
  assert.match(api, /ai_provider_reply_generation/);
  assert.match(api, /AI_INFERENCE_REQUEST_CAPABILITY_ID_V1/);
  assert.match(api, /AI_PROVIDER_GENERATION_CAPABILITY_ID_V1/);
  assert.match(validation, /compute_reply_inference_request_digest_v1/);
  assert.match(validation, /AI_CONTRACTS_SCHEMA_SHA256/);
  assert.match(validation, /AiEgressPolicyLocalOnly/);
  assert.match(proto, /message AiContextReceiptV1/);
  assert.match(proto, /message CommunicationReplySuggestionInferenceRequestV1/);
  assert.match(proto, /message CommunicationReplySuggestionInferenceResultV1/);
  assert.match(proto, /message AiProviderReplyGenerationRequestV1/);
  assert.match(proto, /message AiProviderReplyGenerationResultV1/);
  assert.match(proto, /uint32 maximum_output_bytes/);
  assert.match(proto, /uint32 maximum_output_tokens/);
  assert.match(proto, /AiInferenceCompletenessV1 completeness = 10/);
  assert.match(proto, /uint32 confidence_basis_points = 11/);
  assert.doesNotMatch(
    `${api}\n${validation}\n${proto}`,
    /(?:string|bytes)\s+(?:provider_id|provider_name|model_id|model_name|endpoint|prompt_text)\b|google\.protobuf\.Any|map<|string target_owner|string target_module|string target_capability/,
  );
  assert.doesNotMatch(manifest, /communications|reply-suggestion|ollama|sqlx|gateway|kernel/);
});

test('AI inference core owns lifecycle and fixed policy without provider implementation', async () => {
  const [manifest, core] = await Promise.all([
    backendSource('src/ai-inference-core/Cargo.toml'),
    backendSource('src/ai-inference-core/src/lib.rs'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "ai"/);
  assert.match(manifest, /surface = "implementation"/);
  assert.match(manifest, /hermes-ai-contracts/);
  assert.match(core, /AiInferenceRunStateV1/);
  assert.match(core, /accept_reply_inference_v1/);
  assert.match(core, /begin_reply_inference_v1/);
  assert.match(core, /complete_reply_inference_v1/);
  assert.match(core, /reject_reply_inference_v1/);
  assert.match(core, /AI_INFERENCE_PROVIDER_POLICY_REVISION_V1/);
  assert.match(core, /prompt_policy_sha256_v1/);
  assert.doesNotMatch(
    `${manifest}\n${core}`,
    /communications|reply-suggestion|ollama|reqwest|hyper|sqlx|gateway|kernel|settings_registry|provider_id|model_id|endpoint/,
  );
});
