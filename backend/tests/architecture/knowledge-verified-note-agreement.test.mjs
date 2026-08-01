import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Knowledge admission is exact verified-note ownership without projection or foreign-domain coupling', async () => {
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
    creation,
  ] = await Promise.all([
    readFile(
      new URL('docs/adr/ADR-0370-verified-knowledge-note-owner-admission.md', REPOSITORY_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-command-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-command-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-command-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/knowledge-command-api/proto/hermes/knowledge/command/v1/knowledge_command.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/knowledge-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-core/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/knowledge-core/src/creation.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'knowledge_verified_note_contract_core_v1');
  assert.equal(policy.domains.developmentAllowlist.includes('knowledge'), true);
  assert.equal(policy.domains.blocked.includes('knowledge'), false);
  assert.match(adr, /Состояние реализации: staged/);
  assert.match(adr, /Generic note CRUD, Knowledge Graph, Search, Timeline, Context, Memory/);
  assert.match(adr, /Kernel, Gateway и Event Hub остаются owner-neutral/);
  assert.match(adr, /Cross-owner path остаётся[\s\S]*event-only/);

  for (const unit of ['hermes-knowledge-command-api', 'hermes-knowledge-core']) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('hermes-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
    assert.equal(policy.implementation.productionPackages.some(({ name }) => name === unit), true);
  }

  assert.match(apiManifest, /role = "domain"/);
  assert.match(apiManifest, /owner = "knowledge"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /role = "domain"/);
  assert.match(coreManifest, /owner = "knowledge"/);
  assert.match(coreManifest, /surface = "implementation"/);

  assert.match(api, /knowledge\.reviewed-candidate\.command\.v1/);
  assert.match(api, /knowledge\.reviewed-candidate\.blob\.v1/);
  assert.match(envelope, /build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1/);
  assert.match(envelope, /ResultOutcomeV1::Succeeded/);
  assert.match(envelope, /ResultOutcomeV1::Rejected/);
  assert.doesNotMatch(
    envelope.split('#[cfg(test)]')[0],
    /title|excerpt|topic_hints|provider_id|account_id/,
  );

  assert.match(protocol, /CreateKnowledgeNoteFromReviewedCandidateCommandV1/);
  assert.match(protocol, /KnowledgeNoteCreatedFromReviewedCandidateV1/);
  assert.match(protocol, /KnowledgeNoteCreationFromReviewedCandidateRejectedV1/);
  assert.match(protocol, /ReviewedKnowledgeNoteContentV1/);
  assert.match(protocol, /KNOWLEDGE_NOTE_TOPIC_HINT_DECISION_STATEMENT/);
  assert.match(protocol, /KNOWLEDGE_NOTE_TOPIC_HINT_DEADLINE_STATEMENT/);
  assert.doesNotMatch(
    protocol,
    /provider_id|account_id|project_id|task_id|decision_id|document_id|model_id|prompt|map<|ollama/,
  );

  assert.match(core, /create_verified_knowledge_note_from_reviewed_candidate_v1/);
  assert.match(model, /VerifiedKnowledgeNoteV1/);
  assert.match(model, /VerifiedKnowledgeNoteStatusV1::Verified/);
  assert.match(model, /KnowledgeNoteProvenanceV1/);
  assert.match(model, /note\.note_revision != 1/);
  assert.match(creation, /reviewed_candidate_creates_exactly_one_deterministic_verified_note/);
  assert.match(creation, /missing_human_decision_evidence_is_rejected/);
  assert.match(creation, /unordered_hints_and_invalid_confidence_are_rejected/);
  assert.doesNotMatch(
    `${core}\n${model}\n${creation}`,
    /hermes_communications|hermes_review|hermes_tasks|hermes_documents|graph|search|context|ollama|sqlx|reqwest/,
  );
});
