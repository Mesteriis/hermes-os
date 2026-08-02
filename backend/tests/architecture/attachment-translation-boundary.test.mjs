import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('attachment translation agreement keeps workflow source engine and provider separate', async () => {
  const [inventorySource, adr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('docs/adr/ADR-0378-bounded-attachment-translation-workflow.md', REPOSITORY_ROOT),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'attachment_translation_v1');

  assert.deepEqual(slice, {
    gate: 'attachment_translation_v1',
    role: 'workflow',
    owner: 'attachment_translation',
    state: 'planned',
    dependsOn: [
      'attachment_text_extraction_v1',
      'ai_inference_v1',
      'ollama_ai_provider_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });
  for (const unit of [
    'hermes-attachment-translation-api',
    'hermes-attachment-translation-ingress',
    'hermes-attachment-translation-core',
    'hermes-attachment-translation-persistence',
    'hermes-attachment-translation-runtime',
    'hermes-attachment-translation-assembly',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }
  assert.match(adr, /`attachment_translation` является workflow, не domain и не integration/);
  assert.match(adr, /Workflow не вызывает Attachment Text Extraction RPC/);
  assert.match(adr, /ai\.attachment-translation\.request\.v1/);
  assert.match(adr, /distinct capability/);
  assert.match(adr, /Source text и translated[\s\S]*не попадают в SQL workflow owner/);
  assert.match(adr, /inventory state остаётся `planned`/);
  assert.doesNotMatch(
    adr,
    /Communications owns attachment translation|legacy REST facade открывает gate|caller выбирает provider/,
  );
});
