import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('communication explanation agreement separates workflow domain engine and provider', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    workspace,
    apiManifest,
    api,
    protocol,
    coreManifest,
    core,
    communicationsSourceProtocol,
    communicationsSourceApi,
    aiProtocol,
    aiContracts,
    aiExplanationValidation,
    ollamaApi,
    persistenceManifest,
    persistenceSchema,
    persistenceModel,
    persistenceRepository,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0364-communication-explanation-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-explanation-api/proto/hermes/communication_explanation/v1/explanation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-explanation-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-explanation-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communications-ai-source-api/proto/hermes/communications/ai_source/v1/ai_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communications-ai-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/proto/hermes/ai/contracts/v1/ai.proto', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/explanation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-explanation-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-explanation-persistence/migrations/0001_explanation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-explanation-persistence/src/model.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-explanation-persistence/src/repository.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_explanation_v1');

  assert.deepEqual(slice, {
    gate: 'communication_explanation_v1',
    role: 'workflow',
    owner: 'communication_explanation',
    state: 'planned',
    dependsOn: [
      'communications_ai_context_source_v1',
      'ai_inference_v1',
      'ollama_ai_provider_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });
  for (const unit of [
    'hermes-communication-explanation-api',
    'hermes-communication-explanation-core',
    'hermes-communication-explanation-persistence',
    'hermes-communication-explanation-runtime',
    'hermes-communication-explanation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /почему один canonical[\s\S]*требовать внимания/);
  assert.match(adr, /ai\.explanation\.request\.v1/);
  assert.match(adr, /ai\.provider\.explain\.v1/);
  assert.match(adr, /Smart CC остаётся отдельным/);
  assert.match(adr, /exact reason kind\/source-basis enums/);
  assert.match(adr, /Kernel\/Gateway не компилируют[\s\S]*Explanation schema/);
  assert.match(adr, /Состояние реализации: planned/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns explanation/i);

  assert.equal(policy.implementation.currentSlice, 'communication_explanation_persistence_v1');
  assert.match(workspace, /"src\/communication-explanation-api"/);
  assert.match(workspace, /"src\/communication-explanation-core"/);
  assert.match(workspace, /"src\/communication-explanation-persistence"/);
  assert.match(apiManifest, /owner = "communication_explanation"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /owner = "communication_explanation"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(persistenceManifest, /owner = "communication_explanation"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(api, /COMMUNICATION_EXPLANATION_CAPABILITY_ID_V1/);
  assert.match(protocol, /CommunicationExplanationCandidateV1/);
  assert.match(protocol, /COMMUNICATION_EXPLANATION_REASON_KIND_DEADLINE/);
  assert.match(protocol, /COMMUNICATION_EXPLANATION_SOURCE_BASIS_COMBINED/);
  assert.doesNotMatch(
    protocol,
    /provider_id|model_id|endpoint|prompt|source_body|recipient|task|note|map</,
  );
  assert.match(core, /transition_communication_explanation_v1/);
  assert.match(core, /DuplicateReasonKind/);
  assert.match(core, /allows_empty_reason_list_without_fabricating_a_reason/);
  assert.doesNotMatch(
    core,
    /communication_summary|communication_translation|hermes_ai|ollama|communications_domain/,
  );
  assert.ok(
    policy.implementation.ownerInventory.businessCapabilities.includes(
      'communication.explanation.v1',
    ),
  );
  assert.match(communicationsSourceProtocol, /PrepareCommunicationExplanationSourceCommandV1/);
  assert.match(communicationsSourceProtocol, /CommunicationExplanationSourcePreparedV1/);
  assert.match(communicationsSourceProtocol, /CommunicationExplanationSourceRejectedV1/);
  assert.match(communicationsSourceApi, /communications\.ai-explanation-source\.v1/);
  assert.match(communicationsSourceApi, /communication_explanation\.source\.blob\.v1/);
  assert.doesNotMatch(
    communicationsSourceProtocol,
    /provider_id|model_id|endpoint|prompt|recipient|task|note|map</,
  );
  assert.match(aiProtocol, /CommunicationExplanationInferenceRequestV1/);
  assert.match(aiProtocol, /AiProviderExplanationRequestV1/);
  assert.match(aiProtocol, /AI_EXPLANATION_REASON_KIND_LEGAL_OR_CONTRACTUAL/);
  assert.match(aiProtocol, /AI_EXPLANATION_SOURCE_BASIS_CANONICAL_METADATA/);
  assert.match(aiContracts, /AI_EXPLANATION_REQUEST_CAPABILITY_ID_V1/);
  assert.match(aiContracts, /AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1/);
  assert.match(aiExplanationValidation, /seal_explanation_inference_request_v1/);
  assert.match(aiExplanationValidation, /provider_result_rejects_duplicate_reason_kinds/);
  assert.doesNotMatch(
    aiExplanationValidation,
    /CommunicationSummary|CommunicationTranslation|\b(?:provider_id|model_id|endpoint|prompt)\b/,
  );
  assert.match(ollamaApi, /OLLAMA_AI_EXPLANATION_CAPABILITY_ID_V1/);
  for (const capability of [
    'ai.explanation.request.v1',
    'ai.provider.explain.v1',
    'communication_explanation.inference.v1',
    'communication_explanation.source.blob.v1',
    'communication_explanation.source_prepare.v1',
    'communication_explanation.source_prepared.v1',
    'communication_explanation.source_rejected.v1',
    'communication_explanation.storage.v1',
    'communications.ai-explanation-source.blob.v1',
    'communications.ai-explanation-source.v1',
  ]) {
    assert.ok(policy.implementation.ownerInventory.businessCapabilities.includes(capability));
  }
  assert.match(persistenceSchema, /communication_explanation_runs/);
  assert.match(persistenceSchema, /UNIQUE \(logical_owner_id, operation_id\)/);
  assert.match(persistenceSchema, /candidate_reasons_bytes/);
  assert.match(persistenceSchema, /communication_explanation_inbox/);
  assert.match(persistenceSchema, /communication_explanation_outbox/);
  assert.match(persistenceSchema, /communication_explanation_realtime/);
  assert.doesNotMatch(
    persistenceSchema,
    /communications_|mail_|telegram_|whatsapp_|zulip_|source_body|prompt|provider_id|model_id|endpoint/,
  );
  assert.match(persistenceModel, /encode_reasons/);
  assert.match(persistenceModel, /decode_reasons/);
  assert.match(persistenceRepository, /load_recoverable_runs/);
  assert.match(persistenceRepository, /InboxConflict/);
  assert.match(persistenceRepository, /request_fingerprint/);
  assert.doesNotMatch(
    `${persistenceManifest}\n${persistenceModel}\n${persistenceRepository}`,
    /hermes-(?:communications-domain|ai-inference|ollama|mail|telegram|whatsapp|zulip)/,
  );
});
