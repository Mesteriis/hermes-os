import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('communication explanation agreement separates workflow domain engine and provider', async () => {
  const [adr, inventorySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0364-communication-explanation-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_explanation_v1');

  assert.deepEqual(slice, {
    gate: 'communication_explanation_v1',
    role: 'workflow',
    owner: 'communication_explanation',
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
    'hermes-communication-explanation-api',
    'hermes-communication-explanation-core',
    'hermes-communication-explanation-persistence',
    'hermes-communication-explanation-runtime',
    'hermes-communication-explanation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /почему один canonical[\s\S]*требовать внимания/);
  assert.match(adr, /ai\.explanation\.request\.v1/);
  assert.match(adr, /ai\.provider\.explain\.v1/);
  assert.match(adr, /Smart CC остаётся отдельным/);
  assert.match(adr, /exact reason kind\/source-basis enums/);
  assert.match(adr, /Kernel\/Gateway не компилируют[\s\S]*Explanation schema/);
  assert.match(adr, /Состояние реализации: planned/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns explanation/i);
});
