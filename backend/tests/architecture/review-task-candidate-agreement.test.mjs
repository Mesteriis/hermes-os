import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Review task-candidate is an exact domain capability, not an attention facade', async () => {
  const [
    adr,
    policySource,
    workspace,
    apiManifest,
    api,
    protocol,
    coreManifest,
    core,
    model,
    lifecycle,
    persistenceManifest,
    persistence,
    repository,
    schema,
    migration,
  ] =
    await Promise.all([
      readFile(
        new URL(
          'docs/adr/ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md',
          REPOSITORY_ROOT,
        ),
        'utf8',
      ),
      readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
      readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL(
          'src/review-task-candidate-api/proto/hermes/review/task_candidate/v1/task_candidate.proto',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
      readFile(new URL('src/review-task-candidate-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-core/src/model.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/src/repository.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/review-task-candidate-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL(
          'src/review-task-candidate-persistence/migrations/0001_review_task_candidate.sql',
          BACKEND_ROOT,
        ),
        'utf8',
      ),
    ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'review_task_candidate_persistence_v1');
  for (const unit of [
    'hermes-review-task-candidate-api',
    'hermes-review-task-candidate-core',
    'hermes-review-task-candidate-persistence',
  ]) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('hermes-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
    assert.equal(policy.implementation.productionPackages.some(({ name }) => name === unit), true);
  }
  assert.match(apiManifest, /role = "domain"/);
  assert.match(apiManifest, /owner = "review"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /role = "domain"/);
  assert.match(coreManifest, /owner = "review"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(api, /review\.task-candidate\.submission\.v1/);
  assert.match(api, /review\.task-candidate\.promotion\.v1/);
  assert.match(protocol, /SubmitTaskCandidateForReviewCommandV1/);
  assert.match(protocol, /TaskCandidateApprovedForPromotionV1/);
  assert.match(protocol, /ReviewTargetBoundCandidateReceiptV1/);
  assert.match(protocol, /ReviewTaskCandidateStatusChangedV1/);
  assert.doesNotMatch(protocol, /provider_id|account_id|model_id|prompt|map<|google|telegram|ollama/);
  assert.match(core, /decide_review_task_candidate_v1/);
  assert.match(model, /ReviewTaskCandidatePromotionStatusV1/);
  assert.match(lifecycle, /approval_is_terminal_and_starts_separate_promotion/);
  assert.match(lifecycle, /rejection_never_requests_promotion/);
  assert.match(lifecycle, /stale_revision_and_missing_human_actor_are_rejected/);
  assert.doesNotMatch(`${core}\n${model}\n${lifecycle}`, /review_attention|hermes_communications|hermes_tasks|ollama|sqlx|reqwest/);
  assert.match(persistenceManifest, /owner = "review"/);
  assert.match(persistenceManifest, /surface = "persistence"/);
  assert.match(persistence, /ReviewTaskCandidatePersistenceV1/);
  assert.match(repository, /reserve_submission/);
  assert.match(repository, /load_recoverable_submissions/);
  assert.match(repository, /review_task_candidate_operations/);
  assert.match(repository, /review_task_candidate_promotion_inbox/);
  assert.match(repository, /insert_outbox/);
  assert.match(repository, /insert_realtime/);
  assert.match(schema, /review_task_candidate_storage_bundle_v1/);
  assert.match(migration, /request_sha256 BYTEA/);
  assert.match(migration, /decision_fingerprint BYTEA/);
  assert.match(migration, /review_task_candidate_outbox/);
  assert.match(migration, /review_task_candidate_realtime/);
  assert.doesNotMatch(`${persistence}\n${repository}\n${migration}`, /review_attention|communications_|tasks_|provider_id|account_id|ollama|prompt|model_id/);
  assert.match(adr, /без[\s\S]*расширения `review-attention`/);
});
