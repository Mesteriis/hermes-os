import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('recipient suggestion agreement separates source ownership from workflow decisions', async () => {
  const [adr, inventorySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0365-communication-recipient-suggestion-workflow-and-source-boundary.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const source = inventory.slices.find(({ gate }) => gate === 'communications_recipient_source_v1');
  const workflow = inventory.slices.find(
    ({ gate }) => gate === 'communication_recipient_suggestion_v1',
  );

  assert.deepEqual(source, {
    gate: 'communications_recipient_source_v1',
    role: 'domain',
    owner: 'communications',
    state: 'planned',
    dependsOn: ['communications_canonical_read_v2', 'blob_v1', 'nats_data_plane_v1'],
  });
  assert.deepEqual(workflow, {
    gate: 'communication_recipient_suggestion_v1',
    role: 'workflow',
    owner: 'communication_recipient_suggestion',
    state: 'planned',
    dependsOn: ['communications_recipient_source_v1', 'client_gateway_v1', 'blob_v1'],
  });
  for (const unit of [
    'hermes-communications-recipient-source-api',
    'hermes-communication-recipient-suggestion-api',
    'hermes-communication-recipient-suggestion-core',
    'hermes-communication-recipient-suggestion-persistence',
    'hermes-communication-recipient-suggestion-runtime',
    'hermes-communication-recipient-suggestion-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /какую bounded[\s\S]*организационную роль/);
  assert.match(adr, /accounting_or_bookkeeping/);
  assert.match(adr, /legal_counsel/);
  assert.match(adr, /project_stakeholder/);
  assert.match(adr, /target-bound Blob/);
  assert.match(adr, /общий replayable SSE/);
  assert.match(adr, /Kernel\/Gateway не компилируют/);
  assert.match(adr, /Состояние реализации: planned/);
  assert.doesNotMatch(adr, /Communications (?:owns|владеет) recipient decision|generic `execute\(any\)`/i);
});
