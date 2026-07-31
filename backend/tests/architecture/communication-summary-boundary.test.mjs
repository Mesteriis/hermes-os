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

test('Communications summary source is a distinct event and target-bound Blob handoff', async () => {
  const [runtime, admission, eventRuntime, sourceApi, replyRuntime] = await Promise.all([
    readFile(new URL('src/communications-runtime/src/summary_source.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/event_runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-ai-source-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/communications-runtime/src/ai_source.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(runtime, /PrepareCommunicationSummarySourceCommandV1/);
  assert.match(runtime, /build_communication_summary_source_prepared_outbox_record_v1/);
  assert.match(runtime, /build_communication_summary_source_rejected_outbox_record_v1/);
  assert.match(runtime, /COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(runtime, /COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_MODULE_ID_V1/);
  assert.match(runtime, /COMMUNICATION_SUMMARY_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1/);
  assert.match(runtime, /communications-ai-summary-source-copy-v1/);
  assert.match(admission, /communications\.ai-summary-source\.v1/);
  assert.match(admission, /communications\.ai-summary-source\.blob\.v1/);
  assert.match(eventRuntime, /communication_summary_source_prepare_contract_reference_v1/);
  assert.match(eventRuntime, /CommunicationsConsumerV1::SummarySourcePrepare/);
  assert.match(sourceApi, /"communication_summary"/);
  assert.match(sourceApi, /"hermes-communication-summary-runtime"/);
  assert.doesNotMatch(runtime, /hermes_ollama|ollama|provider_sdk|provider identity/i);
  assert.doesNotMatch(replyRuntime, /PrepareCommunicationSummarySourceCommandV1/);
});
