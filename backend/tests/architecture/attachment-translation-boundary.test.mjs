import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('attachment translation agreement keeps workflow source engine and provider separate', async () => {
  const [inventorySource, policySource, workspace, apiManifest, api, apiProto, ingressManifest, ingress, coreManifest, core, adr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(new URL('Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-translation-api/proto/hermes/attachment_translation/v1/translation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/attachment-translation-ingress/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-ingress/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-translation-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('docs/adr/ADR-0378-bounded-attachment-translation-workflow.md', REPOSITORY_ROOT),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
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
  assert.equal(policy.implementation.currentSlice, 'attachment_translation_contracts_v1');
  assert(policy.implementation.ownerInventory.workflows.includes('attachment_translation'));
  for (const packageName of [
    'hermes-attachment-translation-api',
    'hermes-attachment-translation-ingress',
    'hermes-attachment-translation-core',
  ]) {
    assert(policy.implementation.productionPackages.some(({ name }) => name === packageName));
  }
  assert.match(workspace, /"src\/attachment-translation-api"/);
  assert.match(workspace, /"src\/attachment-translation-ingress"/);
  assert.match(workspace, /"src\/attachment-translation-core"/);
  assert.match(apiManifest, /owner = "attachment_translation"/);
  assert.match(api, /ATTACHMENT_TRANSLATION_TICKET_CONNECT_PATH_V1/);
  assert.doesNotMatch(apiProto, /translated_text_utf8|provider_id|model_id|prompt/);
  assert.match(ingressManifest, /surface = "contract"/);
  assert.match(ingress, /attachment_translation_source_requested_contract_reference_v1/);
  assert.match(ingress, /ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1/);
  assert.match(coreManifest, /surface = "implementation"/);
  assert.match(core, /AttachmentTranslationStateV1/);
  assert.match(core, /MaterializingResult/);
  assert.doesNotMatch(core, /translated_text_utf8|communications|ollama|ai_inference/);
  assert.doesNotMatch(
    adr,
    /Communications owns attachment translation|legacy REST facade открывает gate|caller выбирает provider/,
  );
});
