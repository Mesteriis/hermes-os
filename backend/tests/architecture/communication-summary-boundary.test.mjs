import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('communication summary agreement keeps workflow domain engine and integration separate', async () => {
  const [adr, inventorySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0362-communication-summary-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_summary_v1');

  assert.deepEqual(slice, {
    gate: 'communication_summary_v1',
    role: 'workflow',
    owner: 'communication_summary',
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
    'hermes-communication-summary-api',
    'hermes-communication-summary-core',
    'hermes-communication-summary-persistence',
    'hermes-communication-summary-runtime',
    'hermes-communication-summary-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /ai\.summary\.request\.v1/);
  assert.match(adr, /ai\.provider\.summarize\.v1/);
  assert.match(adr, /existing managed workflow admission/);
  assert.match(adr, /Kernel\/Gateway не компилируют summary schema/);
  assert.match(adr, /Task\/note\/deadline extraction не смешивается/);
  assert.match(adr, /Gate[\s\S]*`communication_summary_v1`[\s\S]*остаётся `planned`/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns summary/i);
});
