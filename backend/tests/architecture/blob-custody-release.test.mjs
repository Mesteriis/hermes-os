import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('Blob custody release starts as a typed control-plane protocol, not data-plane delete', async () => {
  const [adr, blobProto, managedProto, validation, service] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0343-capability-routed-blob-custody-release.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/hermes/runtime/v1/blob_runtime.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/hermes/runtime/v1/managed_runtime_control.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/platform/runtime_protocol/src/validation/blob.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/service/src/control/runtime.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(adr, /Состояние реализации: не реализовано/);
  assert.match(blobProto, /message BlobCustodyReleaseGrantV1/);
  assert.match(blobProto, /message BlobCustodyReleaseRequestV1/);
  assert.match(blobProto, /message BlobCustodyReleaseResponseV1/);
  assert.match(managedProto, /ManagedRuntimeBlobCustodyReleaseRequestV1/);
  assert.match(managedProto, /release_blob_custody = 14/);
  assert.match(validation, /fn valid_release_grant/);
  assert.match(validation, /custody_source_proof_sha256/);
  const dataOperation = blobProto.match(
    /enum BlobDataOperationV1 \{[\s\S]*?\n\}/,
  )?.[0];
  assert.ok(dataOperation);
  assert.doesNotMatch(dataOperation, /RELEASE|DELETE/);
  assert.match(
    service,
    /Some\(Operation::ReleaseCustody\(_\)\) => error_response\("operation_not_available"\)/,
  );
  assert.doesNotMatch(managedProto, /filesystem|data_socket_path.*release_blob_custody/);
});
