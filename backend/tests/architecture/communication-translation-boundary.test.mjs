import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('communication translation agreement isolates workflow domain engine and provider', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    workspace,
    apiManifest,
    api,
    protocol,
    core,
    persistenceManifest,
    persistenceSchema,
    persistenceRepository,
    communicationsSourceProtocol,
    aiProtocol,
    aiContracts,
    aiTranslationValidation,
    ollamaApi,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0363-communication-translation-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-translation-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-translation-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-translation-api/proto/hermes/communication_translation/v1/translation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-translation-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-translation-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-translation-persistence/migrations/0001_translation.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/communication-translation-persistence/src/repository.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communications-ai-source-api/proto/hermes/communications/ai_source/v1/ai_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/ai-contracts/proto/hermes/ai/contracts/v1/ai.proto', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ai-contracts/src/translation.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/ollama-ai-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_translation_v1');

  assert.deepEqual(slice, {
    gate: 'communication_translation_v1',
    role: 'workflow',
    owner: 'communication_translation',
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
    'hermes-communication-translation-api',
    'hermes-communication-translation-core',
    'hermes-communication-translation-persistence',
    'hermes-communication-translation-runtime',
    'hermes-communication-translation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /ai\.translation\.request\.v1/);
  assert.match(adr, /ai\.provider\.translate\.v1/);
  assert.match(adr, /перевод одного canonical communication evidence item/);
  assert.match(adr, /Attachment translation[\s\S]*отдельным/);
  assert.match(adr, /Thread translation[\s\S]*не\s+является неявным batch mode/);
  assert.match(adr, /Kernel\/Gateway не компилируют Translation\s+schema/);
  assert.match(adr, /Состояние реализации: planned/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns translation/i);

  assert.equal(
    policy.implementation.currentSlice,
    'communication_translation_persistence_v1',
  );
  assert.match(workspace, /"src\/communication-translation-api"/);
  assert.match(workspace, /"src\/communication-translation-core"/);
  assert.match(apiManifest, /owner = "communication_translation"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(api, /COMMUNICATION_TRANSLATION_CAPABILITY_ID_V1/);
  assert.match(protocol, /CommunicationTranslationCandidateV1/);
  assert.match(protocol, /COMMUNICATION_TRANSLATION_LANGUAGE_ENGLISH/);
  assert.match(protocol, /COMMUNICATION_TRANSLATION_LANGUAGE_RUSSIAN/);
  assert.match(protocol, /COMMUNICATION_TRANSLATION_LANGUAGE_SPANISH/);
  assert.doesNotMatch(protocol, /provider_id|model_id|endpoint|prompt|source_body|thread_id|attachment_id|map</);
  assert.match(core, /transition_communication_translation_v1/);
  assert.match(core, /DigestMismatch/);
  assert.doesNotMatch(core, /communication_summary|hermes_ai|ollama|communications_domain/);
  for (const capability of [
    'ai.provider.translate.v1',
    'ai.translation.request.v1',
    'communication.translation.v1',
    'communication_translation.source.blob.v1',
    'communication_translation.storage.v1',
    'communications.ai-translation-source.v1',
  ]) {
    assert.ok(policy.implementation.ownerInventory.businessCapabilities.includes(capability));
  }
  assert.match(communicationsSourceProtocol, /PrepareCommunicationTranslationSourceCommandV1/);
  assert.match(communicationsSourceProtocol, /CommunicationTranslationSourcePreparedV1/);
  assert.match(communicationsSourceProtocol, /CommunicationTranslationSourceRejectedV1/);
  assert.match(aiProtocol, /CommunicationTranslationInferenceRequestV1/);
  assert.match(aiProtocol, /AiProviderTranslationRequestV1/);
  assert.match(aiContracts, /AI_TRANSLATION_REQUEST_CAPABILITY_ID_V1/);
  assert.match(aiContracts, /AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1/);
  assert.match(aiTranslationValidation, /seal_translation_inference_request_v1/);
  assert.doesNotMatch(aiTranslationValidation, /CommunicationSummary|CommunicationReply/);
  assert.match(ollamaApi, /OLLAMA_AI_TRANSLATION_CAPABILITY_ID_V1/);
  assert.doesNotMatch(aiProtocol, /provider_id|model_id|map</);
  assert.match(persistenceManifest, /owner = "communication_translation"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistenceSchema, /communication_translation_runs/);
  assert.match(persistenceSchema, /request_fingerprint/);
  assert.match(persistenceSchema, /communication_translation_inbox/);
  assert.match(persistenceSchema, /communication_translation_outbox/);
  assert.match(persistenceSchema, /communication_translation_realtime/);
  assert.match(persistenceRepository, /ON CONFLICT \(logical_owner_id, operation_id\)/);
  assert.doesNotMatch(
    `${persistenceSchema}\n${persistenceRepository}`,
    /communication_summary|communications_|mail_|telegram_|whatsapp_|zulip_|source_body|provider_id|model_id|endpoint/,
  );
});
