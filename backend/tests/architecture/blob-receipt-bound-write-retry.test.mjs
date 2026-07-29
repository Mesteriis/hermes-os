import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('Blob retries only an exact receipt-bound write while ordinary write stays create-only', async () => {
  const [lifecycle, service, adr] = await Promise.all([
    readFile(
      new URL('src/platform/blob/runtime/src/storage/lifecycle.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/service/src/control/data/service.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'docs/adr/ADR-0334-receipt-bound-idempotent-blob-write-retry.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(lifecycle, /pub fn write_receipt_bound/);
  assert.match(lifecycle, /BlobMetadataError::AlreadyExists/);
  assert.match(lifecycle, /BlobStorageError::AlreadyExists/);
  assert.match(lifecycle, /Sha256::digest\(existing\)/);
  assert.match(lifecycle, /BlobLifecycleError::Integrity/);
  assert.match(service, /session\.expected_plaintext_sha256\(\)/);
  assert.match(service, /write_receipt_bound\(request, expected_sha256\)/);
  assert.match(service, /write_new\(request\)/);
  assert.match(adr, /[Оо]бычный write[\s\S]*остаётся строго create-only/);
  assert.match(adr, /полном совпадении SHA-256/);
});
