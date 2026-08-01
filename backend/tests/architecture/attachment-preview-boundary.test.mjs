import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const REPOSITORY_ROOT = new URL('../', BACKEND_ROOT);

test('attachment preview is a planned workflow and not a Communications facade', async () => {
  const [inventorySource, policySource, adr] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('docs/adr/ADR-0373-bounded-attachment-preview-workflow.md', REPOSITORY_ROOT),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const slice = inventory.slices.find(({ gate }) => gate === 'attachment_preview_v1');

  assert.deepEqual(slice, {
    gate: 'attachment_preview_v1',
    role: 'workflow',
    owner: 'attachment_preview',
    state: 'planned',
    dependsOn: ['blob_v1', 'attachment_security_engine_v1'],
  });
  assert.equal(policy.implementation.currentSlice, 'attachment_preview_pdf_adapter_v1');
  assert(policy.implementation.ownerInventory.workflows.includes('attachment_preview'));
  assert(policy.implementation.ownerInventory.businessCapabilities.includes(
    'attachment.preview.v1',
  ));
  assert.match(adr, /Состояние реализации: staged PDF-adapter slice/);
  assert.match(adr, /DOCX adapter,[\s\S]*persistence, managed runtime/);
  assert.match(adr, /Workflow не вызывает Communications или Attachment Security RPC/);
  assert.match(adr, /Legacy base64 `data:` URL не восстанавливается/);
  assert.match(adr, /exact twelve-unit package inventory/);
});

test('public Preview contract separates status ticket and private client blob bytes', async () => {
  const [manifest, controlProto, readProto, source] = await Promise.all([
    readFile(new URL('src/attachment-preview-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-preview-api/proto/hermes/attachment_preview/v1/preview.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/attachment-preview-api/proto/hermes/attachment_preview/read/v1/read.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/attachment-preview-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /role = "workflow"/);
  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(manifest, /hermes-(?:communications|attachment-security|blob|kernel)/);
  assert.match(controlProto, /rpc Start/);
  assert.match(controlProto, /rpc Get/);
  assert.match(controlProto, /rpc IssueRead/);
  const status = controlProto.slice(
    controlProto.indexOf('message GetAttachmentPreviewResponseV1'),
    controlProto.indexOf('message IssueAttachmentPreviewReadRequestV1'),
  );
  const realtime = controlProto.slice(
    controlProto.indexOf('message AttachmentPreviewStatusChangedV1'),
    controlProto.indexOf('service AttachmentPreviewCommandService'),
  );
  assert.doesNotMatch(status, /ticket|blob|bytes preview|data_url|text =/i);
  assert.doesNotMatch(realtime, /ticket|blob|data_url|text =/i);
  assert.match(readProto, /bytes opaque_read_ticket = 2/);
  assert.doesNotMatch(readProto, /blob_reference|custody|provider|filename|content_type/);
  assert.match(source, /ATTACHMENT_PREVIEW_READ_BLOB_PATH_V1/);
  assert.match(source, /ATTACHMENT_PREVIEW_READ_TICKET_BYTES_V1: usize = 32/);
  assert.match(source, /ATTACHMENT_PREVIEW_READ_TICKET_TTL_SECONDS_V1: i64 = 30/);
  assert.doesNotMatch(
    controlProto,
    /\b(?:provider|account_id|filename|filesystem|source_path|data_url|map)\b/,
  );
});

test('pure Preview core owns evidence join lifecycle and output policy only', async () => {
  const [manifest, source, join, lifecycle, policy] = await Promise.all([
    readFile(new URL('src/attachment-preview-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/join.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/lifecycle.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-core/src/policy.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /hermes-attachment-preview-api/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\b(?:sqlx|tokio)\s*=/,
  );
  assert.match(join, /AttachmentPreviewCustodyDelegationIntentV1/);
  const intent = join.slice(
    join.indexOf('struct AttachmentPreviewCustodyDelegationIntentV1'),
    join.indexOf('struct AttachmentPreviewEvidenceJoinV1'),
  );
  assert.doesNotMatch(
    intent,
    /\b(?:blob_reference_id|declared_size|receipt_sha256|custody_transfer_source_proof)\b/,
  );
  assert.match(lifecycle, /AttachmentPreviewStateV1/);
  assert.match(policy, /validate_preview_output_v1/);
  assert.doesNotMatch(
    `${source}\n${join}\n${lifecycle}\n${policy}`,
    /TcpStream|File::|sqlx|postgres|nats|jetstream|hermes_communications|hermes_attachment_security/,
  );
});

test('target-owned Preview ingress carries event custody without caller authority', async () => {
  const [manifest, proto, source, envelope] = await Promise.all([
    readFile(new URL('src/attachment-preview-ingress/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'src/attachment-preview-ingress/proto/hermes/attachment_preview/ingress/v1/custody_delegation.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('src/attachment-preview-ingress/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-ingress/src/envelope.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications|attachment-security|blob|kernel|attachment-preview-(?:api|core|runtime|persistence|assembly))/,
  );
  assert.match(proto, /message RequestAttachmentPreviewCustodyDelegationV1/);
  assert.match(proto, /message AttachmentPreviewCustodyDelegatedV1/);
  const request = proto.slice(
    proto.indexOf('message RequestAttachmentPreviewCustodyDelegationV1'),
    proto.indexOf('message AttachmentPreviewCustodyDelegatedV1'),
  );
  assert.doesNotMatch(
    request,
    /\b(?:source_reference_id|custody_transfer_source_proof|target_owner_id|target_module_id|target_capability_id|provider_id|filename|content_type)\b/,
  );
  assert.match(source, /ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1/);
  assert.match(source, /ATTACHMENT_PREVIEW_BLOB_TARGET_OWNER_ID_V1/);
  assert.match(envelope, /DurableEnvelopeV1/);
  assert.match(envelope, /ResultMetadataV1/);
});

test('renderer contract is byte-only and metadata cannot select behavior', async () => {
  const [manifest, source] = await Promise.all([
    readFile(
      new URL('src/attachment-preview-renderer-contract/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/attachment-preview-renderer-contract/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  assert.match(manifest, /owner = "attachment_preview"/);
  assert.match(manifest, /surface = "contract"/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications|attachment-security|blob|events|runtime|storage|kernel)/,
  );
  assert.match(source, /detect_attachment_preview_source_format_v1/);
  assert.match(source, /source_bytes: &'a \[u8\]/);
  assert.match(source, /DocxContainerCandidate/);
  assert.match(source, /ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1/);
  assert.doesNotMatch(
    source,
    /\b(?:filename|content_type_hint|provider|account_id|filesystem|source_path|url)\b/,
  );
});

test('safe text image and media adapters are three isolated byte-only units', async () => {
  const [textManifest, text, imageManifest, image, mediaManifest, media] = await Promise.all([
    readFile(new URL('src/attachment-preview-text/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-text/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-image/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-image/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-media/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-media/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  for (const manifest of [textManifest, imageManifest, mediaManifest]) {
    assert.match(manifest, /hermes-attachment-preview-renderer-contract/);
    assert.doesNotMatch(
      manifest,
      /hermes-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\b(?:sqlx|tokio)\s*=/,
    );
  }
  assert.doesNotMatch(textManifest, /\bimage\s*=/);
  assert.doesNotMatch(mediaManifest, /\bimage\s*=/);
  assert.match(imageManifest, /image = \{ version = "=0\.25\.9", default-features = false/);
  assert.match(text, /normalized_visible_utf8_v1/);
  assert.match(text, /ATTACHMENT_PREVIEW_MAX_TEXT_BYTES_V1/);
  assert.match(image, /write_to\(&mut output, ImageFormat::Png\)/);
  assert.match(image, /ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1/);
  assert.match(media, /validate_mp3_v1/);
  assert.match(media, /validate_mp4_v1/);
  assert.match(media, /allowed_mp4_brand/);
  assert.doesNotMatch(
    `${text}\n${image}\n${media}`,
    /\b(?:filename|provider|account_id|filesystem|source_path|data_url|url)\b/,
  );
});

test('PDF adapter rasterizes one bounded page without native or owner authority', async () => {
  const [manifest, source] = await Promise.all([
    readFile(new URL('src/attachment-preview-pdf/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/attachment-preview-pdf/src/lib.rs', BACKEND_ROOT), 'utf8'),
  ]);
  assert.match(manifest, /hayro = \{ version = "=0\.7\.1", default-features = true \}/);
  assert.match(manifest, /image = \{ version = "=0\.25\.9", default-features = false, features = \["png"\] \}/);
  assert.doesNotMatch(
    manifest,
    /hermes-(?:communications|attachment-security|blob|events|runtime|storage|kernel)|\b(?:sqlx|tokio)\s*=/,
  );
  assert.match(source, /render_first_page_v1/);
  assert.match(source, /MAX_RENDER_DIMENSION_V1/);
  assert.match(source, /ATTACHMENT_PREVIEW_MAX_IMAGE_PIXELS_V1/);
  assert.match(source, /FORBIDDEN_ACTIVE_MARKERS_V1/);
  assert.match(source, /catch_unwind/);
  assert.match(source, /AttachmentPreviewKindV1::Document/);
  assert.doesNotMatch(
    source,
    /Command::|TcpStream|File::|filesystem|source_path|provider|account_id|filename|content_type_hint|data_url|url/,
  );
});
