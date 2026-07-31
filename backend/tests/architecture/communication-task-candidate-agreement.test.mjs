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
    persistenceRepository,
    persistenceOutbox,
    persistenceSchema,
    migration,
    runtimeManifest,
    runtime,
    runtimeAdmission,
    runtimeExtraction,
    runtimeReviewSubmission,
    runtimeSourceResults,
    assemblyManifest,
    assembly,
    sourceManifest,
    sourceApi,
    sourceProtocol,
    sourceEnvelope,
    communicationsRuntimeManifest,
    communicationsAdmission,
    communicationsEventRuntime,
    communicationsTaskSource,
    managedSetup,
    managedFlow,
    authenticatedStorage,
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
    readFile(new URL('src/communication-task-candidate-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/outbox.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-persistence/migrations/0001_task_candidate.sql', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/extraction.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/review_submission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-runtime/src/source_results.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communication-task-candidate-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
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
    readFile(new URL('src/communications-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/task_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/task_candidate_managed_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
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
  assert.equal(policy.implementation.currentSlice, 'communication_task_candidate_managed_admission_v1');
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
    'hermes-communication-task-candidate-runtime',
    'hermes-communication-task-candidate-assembly',
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
  assert.match(persistenceRepository, /persist_extraction_transition/);
  assert.match(persistenceRepository, /review_submissions/);
  assert.match(
    persistenceRepository,
    /communication_task_candidate_extraction_outbox[\s\S]*insert_realtime_transition[\s\S]*transaction\.commit/,
  );
  assert.match(persistenceOutbox, /unpublished_events/);
  assert.match(persistenceOutbox, /mark_event_published/);
  assert.doesNotMatch(`${persistence}\n${persistenceModel}\n${persistenceRepository}\n${migration}`, /communication_recipient_suggestion|hermes_communications|hermes_review|hermes_tasks|ollama|prompt|provider_id/);
  assert.match(runtimeManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(runtimeManifest, /surface = "runtime"/);
  assert.match(runtime, /CommunicationTaskCandidateManagedRuntimeV1/);
  assert.match(runtimeAdmission, /communication_task_candidate_extraction\.source\.blob\.v1/);
  assert.match(runtimeAdmission, /communication_task_candidate_extraction\.review_submission\.v1/);
  assert.match(runtimeAdmission, /review_task_candidate_submit_contract_reference_v1/);
  assert.match(runtimeAdmission, /review_task_candidate_submit_publish_request_v1/);
  assert.match(runtimeAdmission, /BlobQuotaOperationV1::Write/);
  assert.match(runtimeAdmission, /ProvidedSurfaceKindV1::ClientRealtime/);
  assert.match(runtimeExtraction, /extract_communication_task_candidates_v1/);
  assert.match(runtimeExtraction, /CommunicationTaskSourceContentV1/);
  assert.match(runtimeExtraction, /prepare_review_submissions_v1/);
  assert.match(runtimeReviewSubmission, /build_submit_review_task_candidate_outbox_record_v1/);
  assert.match(runtimeReviewSubmission, /write_review_candidate_v1/);
  assert.match(runtimeReviewSubmission, /ReviewTaskCandidateEnvelopeContextV1/);
  assert.match(runtimeReviewSubmission, /SubmitTaskCandidateForReviewCommandV1/);
  assert.match(runtimeSourceResults, /source_read_receipt_bytes/);
  assert.match(runtimeSourceResults, /materialize_task_source_v1/);
  assert.match(runtimeManifest, /hermes-review-task-candidate-api/);
  assert.doesNotMatch(runtimeManifest, /hermes-review-task-candidate-(core|persistence|runtime|assembly)/);
  assert.doesNotMatch(
    `${runtime}\n${runtimeAdmission}\n${runtimeExtraction}\n${runtimeReviewSubmission}\n${runtimeSourceResults}`,
    /recipient_suggestion|hermes_review_task_candidate_(core|persistence|runtime|assembly)|hermes_tasks|ollama|reqwest|prompt|provider_id/,
  );
  assert.match(assemblyManifest, /owner = "communication_task_candidate_extraction"/);
  assert.match(assemblyManifest, /surface = "assembly"/);
  assert.match(assembly, /communication_task_candidate_extraction_storage_bundle_v1/);
  assert.match(assembly, /communication_task_candidate_extraction\.runtime\.v1/);
  assert.match(assembly, /communication_task_candidate_extraction\.storage\.v1/);
  assert.doesNotMatch(assembly, /recipient_suggestion|hermes_communications|hermes_review|hermes_tasks|ollama|provider_id/);
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
  assert.match(communicationsRuntimeManifest, /hermes-communications-task-source-api/);
  assert.match(communicationsAdmission, /communications_task_source_capability_v1/);
  assert.match(communicationsAdmission, /communications\.task-source\.blob\.v1/);
  assert.match(communicationsEventRuntime, /consume_next_task_source_prepare_v1/);
  assert.match(communicationsTaskSource, /CommunicationTaskSourceContentV1/);
  assert.match(communicationsTaskSource, /subject_utf8: snapshot\.subject_utf8\.clone\(\)/);
  assert.match(communicationsTaskSource, /write_target_bound_source/);
  assert.match(communicationsTaskSource, /persist_source_result/);
  assert.doesNotMatch(communicationsTaskSource, /provider_id|account_id|model_id|prompt|ollama|reqwest/);
  assert.match(managedSetup, /installed_task_candidate_ensemble_release_v1/);
  assert.match(managedSetup, /communication_task_candidate_extraction\.runtime\.v1/);
  assert.match(managedSetup, /review\.task-candidate\.runtime\.v1/);
  assert.match(managedSetup, /tasks\.runtime\.v1/);
  assert.match(managedSetup, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.match(managedSetup, /ManagedDomainRuntimeConfigurationV1/);
  assert.match(managedFlow, /managed_task_candidate_chain_starts_from_one_signed_release/);
  assert.match(managedFlow, /configure_communications_jetstream/);
  assert.match(managedFlow, /start_communications_domain/);
  assert.match(managedFlow, /start_task_candidate_ensemble_v1/);
  assert.match(
    authenticatedStorage,
    /HERMES_STORAGE_MANAGED_TEST_FILTER[\s\S]*managed_task_candidate_chain_starts_from_one_signed_release/,
  );
});
