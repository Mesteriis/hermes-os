import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('provider-neutral Blob delegation reuses the exact Kernel request-provider resolver', async () => {
  const [protocol, blobSession, requestRouter, blobClient, adr] = await Promise.all([
    read('backend/src/platform/runtime_protocol/proto/hermes/runtime/v1/managed_runtime_control.proto'),
    read('backend/src/kernel/src/platform/blob/session.rs'),
    read('backend/src/kernel/src/modules/capability/module_request.rs'),
    read('backend/src/platform/blob/client/src/lib.rs'),
    read('docs/adr/ADR-0390-call-recording-custody-and-speech-to-text-boundary.md'),
  ]);

  assert.match(protocol, /ContractReferenceV1 target_request_contract = 10/);
  assert.match(protocol, /resolved_target_owner_id = 3/);
  assert.match(blobSession, /resolve_provider_for_caller/);
  assert.match(requestRouter, /pub\(crate\) fn resolve_provider_for_caller/);
  assert.match(blobClient, /ManagedBlobResolvedProviderCustodyDelegationRequestV1/);
  const resolvedRequest = blobClient.slice(
    blobClient.indexOf('pub struct ManagedBlobResolvedProviderCustodyDelegationRequestV1'),
    blobClient.indexOf('pub struct ManagedBlobCustodyDelegationV1'),
  );
  for (const forbidden of ['target_owner_id', 'target_module_id', 'target_capability_id']) {
    assert.ok(!resolvedRequest.includes(forbidden), `caller-selected target ${forbidden}`);
  }
  assert.match(adr, /Caller не передаёт эти\s+координаты в provider-neutral режиме/);
});
