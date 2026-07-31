import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../../../', import.meta.url);

test('communication translation agreement isolates workflow domain engine and provider', async () => {
  const [adr, inventorySource] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0363-communication-translation-workflow-and-ai-contracts.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT)),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'communication_translation_v1');

  assert.deepEqual(slice, {
    gate: 'communication_translation_v1',
    role: 'workflow',
    owner: 'communication_translation',
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
    'hermes-communication-translation-api',
    'hermes-communication-translation-core',
    'hermes-communication-translation-persistence',
    'hermes-communication-translation-runtime',
    'hermes-communication-translation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /ai\.translation\.request\.v1/);
  assert.match(adr, /ai\.provider\.translate\.v1/);
  assert.match(adr, /перевод одного canonical communication evidence item/);
  assert.match(adr, /Attachment translation[\s\S]*отдельным/);
  assert.match(adr, /Thread translation[\s\S]*не\s+является неявным batch mode/);
  assert.match(adr, /Kernel\/Gateway не компилируют Translation\s+schema/);
  assert.match(adr, /Состояние реализации: planned/);
  assert.doesNotMatch(adr, /generic `execute\(any\)` разрешён|Communications owns translation/i);
});
