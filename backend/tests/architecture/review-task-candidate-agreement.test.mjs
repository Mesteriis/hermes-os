import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Review task-candidate is an exact domain capability, not an attention facade', async () => {
  const [adr, policySource, workspace, apiManifest, api, protocol, coreManifest, core, model, lifecycle] =
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
    ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'review_task_candidate_core_v1');
  for (const unit of ['hermes-review-task-candidate-api', 'hermes-review-task-candidate-core']) {
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
  assert.match(adr, /без[\s\S]*расширения `review-attention`/);
});
