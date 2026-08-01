import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Review note-candidate is a distinct domain capability without Task or Knowledge implementation coupling', async () => {
  const [
    adr,
    policySource,
    workspace,
    apiManifest,
    api,
    envelope,
    protocol,
    coreManifest,
    core,
    model,
    lifecycle,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0369-communication-note-candidate-extraction-and-reviewed-knowledge-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/review-note-candidate-api/proto/hermes/review/note_candidate/v1/note_candidate.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/review-note-candidate-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-core/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/review-note-candidate-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'review_note_candidate_contract_core_v1');
  for (const unit of [
    'hermes-review-note-candidate-api',
    'hermes-review-note-candidate-core',
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

  assert.match(api, /review\.note-candidate\.submission\.v1/);
  assert.match(api, /review\.note-candidate\.promotion\.v1/);
  assert.match(api, /REVIEWED_NOTE_CANDIDATE_PROMOTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(api, /"reviewed_note_candidate_promotion"/);
  assert.match(envelope, /build_submit_review_note_candidate_outbox_record_v1/);
  assert.match(envelope, /build_review_note_candidate_approved_outbox_record_v1/);
  assert.match(envelope, /ActorKindV1::OwnerDevice/);
  assert.doesNotMatch(
    envelope.split('#[cfg(test)]')[0],
    /title|excerpt|topic_hints|provider_id|account_id/,
  );

  assert.match(protocol, /SubmitNoteCandidateForReviewCommandV1/);
  assert.match(protocol, /NoteCandidateApprovedForPromotionV1/);
  assert.match(protocol, /ReviewNoteCandidateContentV1/);
  assert.match(protocol, /string excerpt/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_FINANCIAL/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_LEGAL/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_DECISION_STATEMENT/);
  assert.match(protocol, /REVIEW_NOTE_TOPIC_HINT_DEADLINE_STATEMENT/);
  assert.doesNotMatch(
    protocol,
    /due_text_hint|assignee_label_hint|task_id|knowledge_note_id|provider_id|account_id|model_id|prompt|map<|ollama/,
  );

  assert.match(core, /decide_review_note_candidate_v1/);
  assert.match(model, /ReviewNoteCandidatePromotionStatusV1/);
  assert.match(model, /ReviewNoteTopicHintV1/);
  assert.match(model, /promoted_note_id/);
  assert.match(lifecycle, /approval_is_terminal_and_starts_separate_promotion/);
  assert.match(lifecycle, /rejection_never_requests_promotion/);
  assert.match(lifecycle, /stale_revision_and_missing_human_actor_are_rejected/);
  assert.doesNotMatch(
    `${core}\n${model}\n${lifecycle}`,
    /review_attention|hermes_communications|hermes_tasks|hermes_knowledge|ollama|sqlx|reqwest/,
  );
});
