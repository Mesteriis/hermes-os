import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('archive inspection is admitted as a separate planned engine slice', async () => {
  const [inventorySource, policySource, adr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'docs/adr/ADR-0359-bounded-attachment-archive-inspection-engine.md',
        REPOSITORY_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'attachment_archive_inspection_v1',
  );

  assert.deepEqual(slice, {
    gate: 'attachment_archive_inspection_v1',
    role: 'engine',
    owner: 'attachment_archive_inspection',
    state: 'planned',
    dependsOn: ['blob_v1', 'attachment_security_engine_v1'],
  });
  assert.equal(
    policy.implementation.currentSlice,
    'attachment_archive_inspection_contract_core_v1',
  );
  assert(policy.implementation.ownerInventory.engines.includes(
    'attachment_archive_inspection',
  ));
  assert.match(adr, /До выполнения всех пунктов inventory state остаётся `planned`/);
  assert.match(adr, /не распаковывает entry bytes/);
  assert.match(adr, /не изменяет safety lifecycle/);
});

test('archive inspection API is bounded and carries no Blob or provider authority', async () => {
  const [manifest, proto, source] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-api/proto/hermes/attachment_archive_inspection/v1/archive_inspection.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(manifest, /hermes-(?:communications|attachment-security|blob|kernel)/);
  assert.match(proto, /bytes operation_id = 2/);
  assert.match(proto, /bytes attachment_anchor_id = 3/);
  assert.match(proto, /repeated ArchiveEntryV1 entries = 4/);
  assert.doesNotMatch(
    proto,
    /\b(?:blob_reference|provider|account_id|filesystem|source_bytes|map)\b/,
  );
  assert.match(source, /MAX_REPORT_ENTRIES_V1: usize = 1_000/);
  assert.match(source, /MAX_PATH_BYTES_V1: usize = 1_024/);
});

test('pure archive core owns policy without transport, storage or parser dependency', async () => {
  const [manifest, source] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /hermes-attachment-archive-inspection-api/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\bzip\s*=/,
  );
  assert.match(source, /ArchiveInspectionLimitsV1/);
  assert.match(source, /DuplicateEntryPath/);
  assert.match(source, /EncryptedEntry/);
  assert.match(source, /NestedArchive/);
  assert.match(source, /UnsupportedEntryType/);
  assert.doesNotMatch(
    source,
    /TcpStream|File::|sqlx|postgres|nats|jetstream|hermes_communications|hermes_attachment_security/,
  );
});

test('ZIP adapter is exact, metadata-only and cannot extract to disk', async () => {
  const [manifest, source] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-zip/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-zip/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(
    manifest,
    /zip = \{ version = "=6\.0\.0", default-features = false, features = \["deflate-flate2-zlib-rs"\] \}/,
  );
  assert.match(manifest, /hermes-attachment-archive-inspection-core/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications|attachment-security|blob|events|runtime|storage|kernel)/,
  );
  assert.match(source, /ZipArchive::new/);
  assert.match(source, /file\.compressed_size\(\)/);
  assert.match(source, /file\.size\(\)/);
  assert.match(source, /file\.encrypted\(\)/);
  assert.match(source, /file\.unix_mode\(\)/);
  assert.doesNotMatch(
    source,
    /std::fs|File::create|create_dir|tempdir|\.extract\s*\(|enclosed_name|read_to_end|TcpStream|sqlx/,
  );
});
