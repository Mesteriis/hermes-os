import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);

test('Attachment Security candidate contract is provider-neutral and payload-bounded', async () => {
  const [proto, admission, manifest] = await Promise.all([
    readFile(
      new URL(
        'src/attachment-security-contract/proto/hermes/attachment_security/v1/scan_candidate.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-contract/src/admission.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-contract/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
  ]);
  const schema = proto.replaceAll(/\/\/.*$/gm, '');
  const fields = [...schema.matchAll(
    /^\s*(bytes|uint64|int64)\s+([a-z0-9_]+)\s*=\s*(\d+);$/gm,
  )].map(([, type, name, number]) => `${type} ${name} ${number}`);

  assert.deepEqual(fields, [
    'bytes attachment_anchor_id 1',
    'bytes blob_reference_id 2',
    'uint64 declared_size 3',
    'bytes blob_receipt_sha256 4',
    'int64 observed_at_unix_seconds 5',
  ]);
  assert.doesNotMatch(
    schema,
    /\b(?:provider|locator|filename|media_type|path|scanner|setting|content|payload|map)\b/i,
  );
  assert.match(admission, /DurableEnvelopeKindV1::Observation/);
  assert.match(admission, /EventRouteDirectionV1::Publish/);
  assert.doesNotMatch(admission, /EventRouteDirectionV1::Subscribe/);
  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_security"/);
  assert.match(manifest, /surface = "contract"/);
});

test('Attachment Security core and ClamAV adapter remain separate engine units', async () => {
  const [coreManifest, core, clamavManifest, endpoint, instream] = await Promise.all([
    readFile(new URL('src/attachment-security-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-core/src/join.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-clamav/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-clamav/src/endpoint.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-clamav/src/instream.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(coreManifest, /hermes-attachment-security-contract/);
  assert.doesNotMatch(
    coreManifest,
    /hermes-(?:communications|blob|attachment-security-clamav|runtime-protocol|storage-protocol)/,
  );
  assert.doesNotMatch(
    core,
    /hermes_communications|TcpStream|std::io|postgres|sqlx|nats|jetstream/i,
  );

  assert.match(clamavManifest, /hermes-attachment-security-contract/);
  assert.match(clamavManifest, /hermes-attachment-security-core/);
  assert.doesNotMatch(
    clamavManifest,
    /hermes-(?:communications|blob|runtime-protocol|storage-protocol)/,
  );
  assert.match(endpoint, /Ipv4Addr::LOCALHOST/);
  assert.match(instream, /const INSTREAM_COMMAND: &\[u8\] = b"zINSTREAM\\0"/);
  assert.match(instream, /response == b"stream: OK"/);
  assert.doesNotMatch(instream, /enum ClamAvScanErrorV1[\s\S]*?\bString\b/);
  assert.doesNotMatch(`${endpoint}\n${instream}`, /hermes_communications|blob_store|postgres|sqlx/i);
});

test('Attachment Security persistence owns the durable join, bounded jobs and exact outbox', async () => {
  const [manifest, schema, observation, jobs] = await Promise.all([
    readFile(new URL('src/attachment-security-persistence/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-security-persistence/migrations/0001_attachment_security_state.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-security-persistence/src/observation.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/attachment-security-persistence/src/jobs.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /surface = "persistence"/);
  assert.match(manifest, /hermes-attachment-security-core/);
  assert.match(manifest, /hermes-communications-attachment-contract/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications-(?!attachment-contract)|blob|kernel|attachment-security-clamav)/,
  );
  assert.equal(schema.match(/CREATE TABLE hermes_data\./g)?.length, 7);
  assert.doesNotMatch(schema, /hermes_data\.(?:communications|mail|telegram|zulip|whatsapp)_/);
  assert.match(schema, /attachment_security_event_inbox/);
  assert.match(schema, /envelope_sha256/);
  assert.match(schema, /max_attempts INTEGER NOT NULL CHECK \(max_attempts BETWEEN 1 AND 32\)/);
  assert.match(observation, /attachment_security_join_locks/);
  assert.match(observation, /FOR UPDATE/);
  assert.match(observation, /decide_scan_join_v1/);
  assert.match(jobs, /FOR UPDATE SKIP LOCKED/);
  assert.match(jobs, /attempt_count >= max_attempts/);
  assert.match(jobs, /attempt_count = \$4 FOR UPDATE/);
  assert.match(jobs, /AttachmentSafetyVerdictOutboxRecordV1/);
  assert.match(jobs, /AttachmentSafetyExpectedStateV1::BlobAdmitted/);
  assert.match(jobs, /exact_envelope_bytes/);
  assert.match(jobs, /OutboxHashConflict/);
  assert.doesNotMatch(
    `${observation}\n${jobs}`,
    /hermes_communications_(?!attachment_contract)|provider_(?:id|locator|sdk)|scanner_signature/i,
  );
});

test('Mail publishes scan candidates through one exact contract and a separate durable outbox', async () => {
  const [manifest, admission, managed, durable, relay, main] = await Promise.all([
    readFile(new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-persistence/src/durable.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/mail-runtime/src/attachment_security_outbox.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/mail-runtime/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /^hermes-attachment-security-contract =/m);
  assert.doesNotMatch(
    manifest,
    /hermes-attachment-security-(?:core|clamav|persistence|runtime|assembly)/,
  );
  assert.match(
    admission,
    /MAIL_ATTACHMENT_SCAN_CANDIDATE_PUBLISH_CAPABILITY_ID: &str =\s*"mail\.attachment\.scan-candidate\.publish\.v1"/,
  );
  assert.match(
    admission,
    /attachment_security_scan_candidate_observed_publish_request_v1\(\)/,
  );
  assert.match(managed, /build_attachment_security_scan_candidate_outbox_record_v1/);
  assert.match(managed, /blob_reference_id: write\.reference_id/);
  assert.match(managed, /blob_receipt_sha256: write\.receipt_sha256/);
  assert.match(
    managed,
    /complete_attachment_blob_admission\([\s\S]*attachment_security_record\.as_ref\(\)/,
  );
  assert.match(durable, /mail_attachment_security_outbox/);
  assert.match(durable, /insert_attachment_security_outbox\(/);
  assert.match(relay, /pending_attachment_security_outbox/);
  assert.match(relay, /publish_exact\(permit, record\.exact_bytes\(\)\)/);
  assert.match(relay, /mark_attachment_security_outbox_published/);
  assert.match(main, /relay_attachment_security_outbox\(now\)/);
  assert.doesNotMatch(
    `${managed}\n${durable}\n${relay}`,
    /hermes_attachment_security_(?:core|clamav|persistence|runtime|assembly)/,
  );
});

test('Attachment Security runtime is a managed engine with event-only business boundaries', async () => {
  const [manifest, admission, runtime, scanner, decoder, outbox] = await Promise.all([
    readFile(new URL('src/attachment-security-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/runtime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/scan.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/event_decode.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-runtime/src/outbox.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_security"/);
  assert.match(manifest, /surface = "runtime"/);
  for (const dependency of [
    'hermes-attachment-security-contract',
    'hermes-attachment-security-core',
    'hermes-attachment-security-clamav',
    'hermes-attachment-security-persistence',
    'hermes-communications-attachment-contract',
    'hermes-blob-client',
    'hermes-events-jetstream',
  ]) {
    assert.match(manifest, new RegExp(`^${dependency} =`, 'm'));
  }
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications-(?!attachment-contract)|mail|telegram|whatsapp|zulip|kernel)/,
  );
  assert.deepEqual(
    [...admission.matchAll(
      /pub const ATTACHMENT_SECURITY_[A-Z_]+_CAPABILITY_ID: &str =\s*"([^"]+)";/g,
    )].map(([, capability]) => capability).sort(),
    [
      'attachment_security.blob.read.v1',
      'attachment_security.candidate.observe.v1',
      'attachment_security.communications-state.observe.v1',
      'attachment_security.storage.v1',
      'attachment_security.verdict.publish.v1',
    ],
  );
  assert.match(admission, /ModuleKindV1::Engine/);
  assert.match(runtime, /ManagedControlChannelV2/);
  assert.match(runtime, /request_managed_runtime_event_access_v2/);
  assert.match(scanner, /receipt_sha256: Some\(&claimed\.job\.blob_receipt_sha256\)/);
  assert.match(scanner, /scan_clamav_loopback_v1/);
  assert.match(runtime, /retry_scan_job/);
  assert.match(runtime, /complete_scan_job_with_outbox/);
  assert.match(decoder, /Semantics::Observation/);
  assert.match(decoder, /Semantics::Event/);
  assert.match(decoder, /BlobPending/);
  assert.match(decoder, /BlobAdmitted/);
  assert.match(outbox, /publish_exact\(permit, record\.exact_bytes\(\)\)/);
  assert.doesNotMatch(
    `${runtime}\n${scanner}\n${decoder}\n${outbox}`,
    /hermes_(?:communications_(?:domain|runtime|persistence|api)|mail|telegram|whatsapp|zulip|kernel)/,
  );
});

test('Attachment Security Blob reads are one-use and receipt-bound below the engine', async () => {
  const [protocol, client, kernelSession, serviceSession, service] = await Promise.all([
    readFile(
      new URL(
        'src/platform/runtime_protocol/proto/hermes/runtime/v1/blob_runtime.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/platform/blob/client/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/kernel/src/platform/blob/session.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/platform/blob/service/src/control/data/session.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/platform/blob/service/src/control/data/service.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(protocol, /bytes expected_plaintext_sha256 = 21;/);
  assert.match(client, /BlobDataOperationV1::BlobDataOperationReadRangeV1/);
  assert.match(client, /exact_receipt_binding\(&grant\.expected_plaintext_sha256/);
  assert.match(kernelSession, /expected_plaintext_sha256: request\.receipt_sha256\.clone\(\)/);
  assert.match(serviceSession, /expected_plaintext_sha256: Option<\[u8; 32\]>/);
  assert.match(service, /exact_read_range_binding/);
  assert.match(service, /Sha256::digest\(plaintext\)/);
  assert.doesNotMatch(
    `${kernelSession}\n${service}`,
    /hermes_(?:communications|attachment_security)|clamav/i,
  );
});

test('Attachment Security release assembly is a separate unsigned engine unit', async () => {
  const [manifest, assembly, command] = await Promise.all([
    readFile(new URL('src/attachment-security-assembly/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-assembly/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-security-assembly/src/main.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const dependencySection = manifest.split('[dependencies]\n')[1] ?? '';
  const dependencies = [...dependencySection.matchAll(/^([a-z0-9_-]+)\s*=/gm)]
    .map(([, dependency]) => dependency)
    .sort();

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "attachment_security"/);
  assert.match(manifest, /surface = "assembly"/);
  assert.deepEqual(dependencies, [
    'hermes-attachment-security-persistence',
    'hermes-attachment-security-runtime',
    'hermes-runtime-protocol',
    'hermes-storage-protocol',
    'prost',
    'serde',
    'serde_json',
  ]);
  for (const file of [
    'attachment-security.runtime.descriptor.pb',
    'attachment-security.runtime.settings.pb',
    'attachment-security.storage.bundle.pb',
    'attachment-security.release-artifacts.json',
  ]) {
    assert.ok(assembly.includes(file), `assembly must materialize ${file}`);
  }
  assert.match(assembly, /validate_descriptor_v1/);
  assert.match(assembly, /validate_settings_schema_v1/);
  assert.match(assembly, /validate_storage_bundle/);
  assert.match(assembly, /create_new\(true\)/);
  assert.match(command, /--build-id/);
  assert.match(command, /--output-dir/);
  assert.match(command, /--runtime/);
  assert.doesNotMatch(
    `${manifest}\n${assembly}\n${command}`,
    /hermes-(?:communications|mail|telegram|whatsapp|zulip|kernel|blob|events)|SigningKey|sign_manifest|ed25519|p256/,
  );
});

test('staged Attachment Security packages do not open the production engine gate', async () => {
  const policy = JSON.parse(await readFile(POLICY_PATH, 'utf8'));
  const productionPackages = policy.implementation.productionPackages;

  assert.deepEqual(policy.implementation.ownerInventory.engines, []);
  assert.equal(
    productionPackages.some(({ name }) => name.startsWith('hermes-attachment-security-')),
    false,
  );
  assert.equal(
    policy.implementation.ownerInventory.businessCapabilities.some(
      (capability) => capability.startsWith('attachment_security.'),
    ),
    false,
  );
});
