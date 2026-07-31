import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('archive inspection persistence is admitted without opening the planned gate', async () => {
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
    'attachment_security_archive_delegation_runtime_v1',
  );
  assert(policy.implementation.ownerInventory.engines.includes(
    'attachment_archive_inspection',
  ));
  assert(policy.implementation.productionPackages.some(
    ({ name, role, owner, surface }) =>
      name === 'hermes-attachment-archive-inspection-persistence'
      && role === 'engine'
      && owner === 'attachment_archive_inspection'
      && surface === 'persistence',
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

test('archive ingress is a target-owned event contract without engine implementation coupling', async () => {
  const [manifest, proto, source] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-ingress/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-ingress/proto/hermes/attachment_archive_inspection/ingress/v1/custody_delegation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-ingress/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(manifest, /hermes-events-protocol/);
  assert.match(manifest, /hermes-runtime-protocol/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:attachment-security|attachment-archive-inspection-(?:api|core|persistence|runtime|assembly)|communications|blob|kernel)/,
  );
  assert.match(proto, /message RequestArchiveInspectionCustodyDelegationV1/);
  assert.match(proto, /bytes candidate_envelope_sha256 = 5/);
  assert.match(proto, /message ArchiveInspectionCustodyDelegatedV1/);
  assert.match(proto, /bytes custody_transfer_source_proof = 9/);
  const request = proto.slice(
    proto.indexOf('message RequestArchiveInspectionCustodyDelegationV1'),
    proto.indexOf('message ArchiveInspectionCustodyDelegatedV1'),
  );
  assert.doesNotMatch(
    request,
    /\b(?:blob_reference|custody_transfer_source_proof|provider_id|account_id|filesystem_path|target_owner_id|target_module_id|target_capability_id)\b/,
  );
  assert.match(source, /ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_CAPABILITY_ID_V1/);
  assert.match(source, /ATTACHMENT_ARCHIVE_INSPECTION_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(source, /DurableEnvelopeKindV1::Command/);
  assert.match(source, /DurableEnvelopeKindV1::Result/);
});

test('pure archive core owns policy without transport, storage or parser dependency', async () => {
  const [manifest, source, join] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-core/src/join.rs', BACKEND_ROOT),
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
  assert.match(join, /ArchiveInspectionCustodyDelegationIntentV1/);
  const intent = join.slice(
    join.indexOf('struct ArchiveInspectionCustodyDelegationIntentV1'),
    join.indexOf('enum ArchiveInspectionRejectionV1'),
  );
  assert.doesNotMatch(
    intent,
    /\b(?:blob_reference_id|declared_size|receipt_sha256|custody_transfer_source_proof)\b/,
  );
  assert.doesNotMatch(
    source,
    /TcpStream|File::|sqlx|postgres|nats|jetstream|hermes_communications|hermes_attachment_security/,
  );
});

test('archive persistence owns replay, event join and fenced jobs without foreign implementations', async () => {
  const [manifest, schema, library, observations, custody, jobs] = await Promise.all([
    readFile(
      new URL('src/attachment-archive-inspection-persistence/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-persistence/migrations/0001_archive_inspection.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-persistence/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-archive-inspection-persistence/src/observations.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-persistence/src/custody.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-archive-inspection-persistence/src/jobs.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_archive_inspection"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /hermes-attachment-archive-inspection-core/);
  assert.match(manifest, /hermes-attachment-archive-inspection-ingress/);
  assert.match(manifest, /hermes-events-protocol/);
  assert.match(manifest, /hermes-storage-protocol/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications|attachment-security|blob|kernel|mail|telegram|whatsapp|zulip)/,
  );
  for (const table of [
    'attachment_archive_inspection_runs',
    'attachment_archive_inspection_event_inbox',
    'attachment_archive_inspection_scan_candidates',
    'attachment_archive_inspection_safety_facts',
    'attachment_archive_inspection_custody_delegation_requests',
    'attachment_archive_inspection_custody_result_inbox',
    'attachment_archive_inspection_jobs',
    'attachment_archive_inspection_reports',
    'attachment_archive_inspection_realtime',
  ]) {
    assert.match(schema, new RegExp(table));
  }
  assert.match(schema, /runtime_generation BIGINT/);
  assert.match(schema, /grant_epoch BIGINT/);
  assert.match(schema, /lease_fence BIGINT/);
  assert.doesNotMatch(
    schema,
    /\b(?:provider_id|provider_path|message_body|archive_bytes|extracted_content)\b/,
  );
  assert.match(library, /verify_storage_ready/);
  assert.match(observations, /persist_scan_candidate/);
  assert.match(observations, /persist_canonical_safety_fact/);
  assert.match(observations, /settle_anchor_runs/);
  assert.match(library, /PendingArchiveInspectionCustodyDelegationV1/);
  assert.match(custody, /pending_custody_delegation_requests/);
  assert.match(custody, /store_custody_delegation_outbox/);
  assert.match(custody, /persist_custody_delegated_result/);
  assert.match(custody, /persist_custody_delegation_rejected_result/);
  assert.match(custody, /insert_result_inbox/);
  assert.match(jobs, /claim_next_job/);
  assert.match(jobs, /recover_expired_jobs/);
  assert.match(jobs, /verify_claim/);
  assert.doesNotMatch(
    `${library}\n${observations}\n${custody}\n${jobs}`,
    /hermes_(?:communications|attachment_security|blob|kernel|mail|telegram|whatsapp|zulip)/,
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
