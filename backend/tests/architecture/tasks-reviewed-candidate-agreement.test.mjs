import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('Tasks reviewed-candidate command and core are distinct target-owned units', async () => {
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
      new URL(
        'docs/adr/ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-command-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-command-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-command-api/src/envelope.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/tasks-command-api/proto/hermes/tasks/command/v1/tasks_command.proto', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/tasks-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-core/src/model.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/tasks-core/src/creation.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const policy = JSON.parse(policySource);

  assert.equal(policy.implementation.currentSlice, 'tasks_reviewed_candidate_contract_core_v1');
  for (const unit of ['hermes-tasks-command-api', 'hermes-tasks-core']) {
    assert.match(workspace, new RegExp(`"src/${unit.replace('hermes-', '')}"`));
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
    assert.equal(policy.implementation.productionPackages.some(({ name }) => name === unit), true);
  }
  assert.match(apiManifest, /role = "domain"/);
  assert.match(apiManifest, /owner = "tasks"/);
  assert.match(apiManifest, /surface = "contract"/);
  assert.match(coreManifest, /role = "domain"/);
  assert.match(coreManifest, /owner = "tasks"/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(protocol, /CreateTaskFromReviewedCandidateCommandV1/);
  assert.match(protocol, /TaskCreatedFromReviewedCandidateV1/);
  assert.match(protocol, /TaskCreationFromReviewedCandidateRejectedV1/);
  assert.match(protocol, /TasksTargetBoundCandidateReceiptV1/);
  assert.doesNotMatch(protocol, /provider_id|account_id|project_id|calendar_event_id|map<|ollama/);
  assert.match(api, /tasks\.reviewed-candidate\.command\.v1/);
  assert.match(api, /create_task_from_reviewed_candidate_consume_request_v1/);
  assert.match(envelope, /build_create_task_from_reviewed_candidate_outbox_record_v1/);
  assert.match(envelope, /ResultOutcomeV1::Succeeded/);
  assert.match(envelope, /ResultOutcomeV1::Rejected/);
  assert.match(core, /create_task_from_reviewed_candidate_v1/);
  assert.match(model, /TaskProvenanceV1/);
  assert.match(model, /derive_task_id_v1/);
  assert.match(model, /task_creation_fingerprint_v1/);
  assert.match(creation, /reviewed_candidate_creates_exactly_one_deterministic_open_task/);
  assert.match(creation, /hints_do_not_materialize_foreign_domain_identity/);
  assert.doesNotMatch(
    `${core}\n${model}\n${creation}`,
    /hermes_review|hermes_communications|hermes_calendar|hermes_contacts|hermes_projects|sqlx|reqwest/,
  );
});
