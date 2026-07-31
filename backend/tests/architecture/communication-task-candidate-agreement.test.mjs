import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('task candidate agreement keeps extraction review and Tasks in separate owner units', async () => {
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
    extraction,
    lifecycle,
    persistenceManifest,
    persistence,
    persistenceModel,
    persistenceSchema,
    migration,
    sourceManifest,
    sourceApi,
    sourceProtocol,
    sourceEnvelope,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT)),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communication-task-candidate-api/proto/hermes/communication_task_candidate/v1/task_candidate.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communication-task-candidate-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-core/src/extraction.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/migrations/0001_task_candidate.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-task-source-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-task-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/communications-task-source-api/proto/hermes/communications/task_source/v1/task_source.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/communications-task-source-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'communication_task_candidate_extraction_v1',
  );

  assert.deepEqual(slice, {
    gate: 'communication_task_candidate_extraction_v1',
    role: 'workflow',
    owner: 'communication_task_candidate_extraction',
    state: 'planned',
    dependsOn: ['communications_content_read_v1'],
  });
  assert.equal(policy.domains.registered.includes('tasks'), true);
  assert.equal(policy.domains.developmentAllowlist.includes('tasks'), true);
  assert.equal(policy.domains.blocked.includes('tasks'), false);
  assert.equal(policy.implementation.currentSlice, 'communication_task_candidate_persistence_v1');
  assert.match(adr, /Состояние реализации: planned/);
  assert.match(adr, /Communications остаётся canonical evidence\/source owner/);
  assert.match(adr, /Extraction остаётся workflow/);
  assert.match(adr, /Review владеет human decision/);
  assert.match(adr, /Tasks — durable Task truth/);
  assert.match(adr, /typed durable commands\/results\/events/);
  assert.match(adr, /target-bound Blob custody/);
  assert.match(adr, /общий[\s\S]*replayable SSE/);
  assert.match(adr, /Periodic polling не вводится/);
  assert.match(adr, /AI Engine и Ollama не используются/);
  assert.match(adr, /Kernel, Gateway и Event Hub остаются owner-neutral/);
  assert.match(adr, /CreateTaskFromReviewedCandidateCommandV1/);
  assert.match(adr, /не создаёт Task до approve/);
  assert.match(adr, /reject[\s\S]*никогда не создаёт Task/);
  assert.match(adr, /approve[\s\S]*ровно один source-backed Task/);
  assert.doesNotMatch(adr, /generic `create\(entity_kind, payload\)` разрешён/);
  assert.doesNotMatch(adr, /Communications владеет Task|Tasks читает Communications storage/);

  for (const unit of [
    'hermes-communication-task-candidate-api',
    'hermes-communication-task-candidate-core',
    'hermes-communication-task-candidate-persistence',
    'hermes-communications-task-source-api',
  ]) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('hermes-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(apiManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(api, /communication\.task-candidate-extraction\.v1/);
  assert.match(protocol, /candidate_digest/);
  assert.match(protocol, /COMMUNICATION_TASK_SIGNAL_KIND_EXPLICIT_ACTION/);
  assert.match(protocol, /COMMUNICATION_TASK_SIGNAL_KIND_DIRECT_REQUEST/);
  assert.match(protocol, /COMMUNICATION_TASK_SIGNAL_KIND_FOLLOW_UP/);
  assert.doesNotMatch(protocol, /project_id|contact_id|persona_id|provider_id|account_id|model_id|prompt|map</);
  assert.match(coreManifest, /role = "workflow"/);
  assert.match(coreManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(core, /extract_communication_task_candidates_v1/);
  assert.match(extraction, /empty_source_does_not_fabricate_a_task_candidate/);
  assert.match(extraction, /duplicate_title_across_subject_and_body_becomes_one_combined_candidate/);
  assert.match(lifecycle, /SourceIdentityMismatch/);
  assert.doesNotMatch(`${core}\n${extraction}\n${lifecycle}`, /hermes_communications|hermes_review|hermes_tasks|ollama|reqwest|sqlx/);
  assert.match(persistenceManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistence, /CommunicationTaskCandidatePersistenceV1/);
  assert.match(persistenceModel, /candidate_codec_preserves_all_typed_fields_and_empty_result/);
  assert.match(persistenceSchema, /communication_task_candidate_extraction_storage_bundle_v1/);
  assert.match(migration, /communication_task_candidate_extraction_runs/);
  assert.match(migration, /communication_task_candidate_extraction_inbox/);
  assert.match(migration, /communication_task_candidate_extraction_outbox/);
  assert.match(migration, /communication_task_candidate_extraction_realtime/);
  assert.doesNotMatch(`${persistence}\n${persistenceModel}\n${migration}`, /communication_recipient_suggestion|hermes_communications|hermes_review|hermes_tasks|ollama|prompt|provider_id/);
  assert.match(sourceManifest, /owner = "communications"/);
  assert.match(sourceManifest, /surface = "contract"/);
  assert.match(sourceApi, /communications\.task-source\.v1/);
  assert.match(sourceApi, /communication_task_candidate_extraction\.source\.blob\.v1/);
  assert.match(sourceProtocol, /PrepareCommunicationTaskSourceCommandV1/);
  assert.match(sourceProtocol, /CommunicationTaskSourceContentV1/);
  assert.match(sourceProtocol, /CommunicationTaskSourcePreparedV1/);
  assert.match(sourceProtocol, /CommunicationTaskSourceRejectedV1/);
  assert.doesNotMatch(sourceProtocol, /provider_id|account_id|model_id|prompt|map</);
  assert.match(sourceEnvelope, /target_capability: COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID_V1/);
});
