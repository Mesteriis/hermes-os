import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('task candidate agreement keeps extraction review and Tasks in separate owner units', async () => {
  const [adr, inventorySource, policySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT)),
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
});
