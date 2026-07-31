import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);

const MAIL_PACKAGES = [
  'hermes-mail-api',
  'hermes-mail-core',
  'hermes-mail-imap',
  'hermes-mail-gmail',
  'hermes-mail-smtp',
  'hermes-mail-persistence',
  'hermes-mail-runtime',
  'hermes-mail-assembly',
];

const PROVIDER_DELIVERY_CONTRACT_PACKAGES = [
  'hermes-mail-delivery-intent-contract',
  'hermes-telegram-delivery-intent-contract',
  'hermes-whatsapp-delivery-intent-contract',
  'hermes-zulip-delivery-intent-contract',
];

const STAGED_OLLAMA_PACKAGES = [
  'hermes-ollama-ai-api',
  'hermes-ollama-ai-assembly',
  'hermes-ollama-ai-core',
  'hermes-ollama-ai-http',
  'hermes-ollama-ai-persistence',
  'hermes-ollama-ai-runtime',
];

const MAIL_CAPABILITIES = [
  'mail.attachment-anchor.consume.v1',
  'mail.attachment-blob-admission.publish.v1',
  'mail.attachment-safety-state.consume.v1',
  'mail.attachment.scan-candidate.publish.v1',
  'mail.blob.v1',
  'mail.communication-observed.publish.v1',
  'mail.delivery.query.v1',
  'mail.delivery.v1',
  'mail.gmail.credentials.v1',
  'mail.gmail.oauth-refresh.credentials.v1',
  'mail.gmail.oauth-setup.credentials.v1',
  'mail.imap.credentials.v1',
  'mail.oauth.complete.v1',
  'mail.oauth.query.v1',
  'mail.oauth.refresh.v1',
  'mail.oauth.start.v1',
  'mail.smtp.credentials.v1',
  'mail.storage.v1',
  'mail.sync.v1',
];

const MAIL_CARGO_FEATURES = {
  'hermes-communication-cross-channel-forward-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-communication-delayed-delivery-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-communication-delivery-intent-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-api': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-imap': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-gmail': {
    default: [],
    'conformance-test-support': ['hermes-mail-api/conformance-test-support'],
  },
  'hermes-mail-persistence': {
    default: [],
    'conformance-test-support': [],
  },
  'hermes-mail-runtime': {
    default: [],
    'conformance-test-support': [
      'hermes-mail-api/conformance-test-support',
      'hermes-mail-gmail/conformance-test-support',
      'hermes-mail-imap/conformance-test-support',
    ],
  },
};

test('Mail outbound attachments keep provider delivery contracts as separate integration units', async () => {
  const policy = JSON.parse(await readFile(POLICY_PATH, 'utf8'));
  const inventory = policy.implementation.ownerInventory;

  assert.equal(
    policy.implementation.currentSlice,
    'ollama_ai_provider_v1',
  );
  assert.deepEqual(inventory.domains, ['communications', 'review']);
  assert.deepEqual(inventory.integrations, ['mail']);
  assert.deepEqual(inventory.workflows, [
    'communication_cross_channel_forward',
    'communication_delivery_intent',
    'communication_reply_suggestion',
    'communications_export',
  ]);
  assert.deepEqual(inventory.engines, [
    'ai',
    'attachment_archive_inspection',
    'attachment_security',
  ]);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ role }) => role === 'integration')
      .map(({ name }) => name),
    [...MAIL_PACKAGES, ...PROVIDER_DELIVERY_CONTRACT_PACKAGES, ...STAGED_OLLAMA_PACKAGES],
  );
  assert.deepEqual(
    inventory.businessCapabilities.filter((capability) => capability.startsWith('mail.')),
    MAIL_CAPABILITIES,
  );
  assert.equal(policy.implementation.cargoFeaturesEnabled, false);
  assert.deepEqual(policy.implementation.cargoFeatureAllowlist, MAIL_CARGO_FEATURES);
  assert.deepEqual(
    policy.phaseGates.requires.mail_outbound_mime_attachments_v1,
    [
      'managed_launch_trust_v1',
      'vault_v1',
      'storage_control_v1',
      'nats_data_plane_v1',
      'blob_v1',
      'attachment_security_engine_v1',
      'client_gateway_v1',
    ],
  );
  assert.equal(
    policy.phaseGates.notAuthorized.includes('mail_outbound_mime_attachments_v1'),
    false,
  );
});

test('generated Mail client carries only bounded canonical attachment anchor IDs', async () => {
  const [proto, api, wire] = await Promise.all([
    readFile(
      new URL('src/mail-api/proto/hermes/mail/v1/client.proto', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/mail-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-api/src/client_wire.rs', BACKEND_ROOT), 'utf8'),
  ]);
  const request = proto.match(/message SendMailRequestV1 \{([\s\S]*?)\n\}/)?.[1];

  assert.ok(request, 'SendMailRequestV1 must remain a generated typed contract');
  assert.match(request, /repeated bytes attachment_anchor_id = 6;/);
  assert.match(request, /repeated string cc_recipient = 7;/);
  assert.match(request, /repeated string bcc_recipient = 8;/);
  assert.doesNotMatch(
    request,
    /\b(?:blob|reference|receipt|custody|path|url|content|mime|verdict|safe)\b/i,
  );
  assert.match(api, /pub const MAX_DELIVERY_ATTACHMENTS: usize = 16/);
  assert.match(api, /pub attachment_anchor_ids: Vec<\[u8; 16\]>/);
  assert.match(api, /pub cc_recipients: Vec<String>/);
  assert.match(api, /pub bcc_recipients: Vec<String>/);
  assert.match(wire, /attachment_anchor_ids\.len\(\) > MAX_DELIVERY_ATTACHMENTS/);
  assert.match(wire, /anchor_id\.iter\(\)\.all\(\|byte\| \*byte == 0\)/);
  assert.match(
    wire,
    /request\.attachment_anchor_ids\[\.\.index\]\.contains\(anchor_id\)/,
  );
  assert.match(wire, /encode_delivery_request\(&request\) != bytes/);
});

test('Mail core, provider adapters and composition root keep functional SRP boundaries', async () => {
  const [
    coreManifest,
    mime,
    smtpManifest,
    gmailManifest,
    runtimeManifest,
  ] = await Promise.all([
    readFile(new URL('src/mail-core/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-core/src/outbound_mime.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-smtp/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-gmail/Cargo.toml', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
  ]);

  assert.doesNotMatch(
    mime,
    /hermes_(?:communications|attachment_security|blob|storage|mail_(?:smtp|gmail|persistence|runtime))/,
  );
  assert.doesNotMatch(
    `${coreManifest}\n${smtpManifest}\n${gmailManifest}`,
    /hermes-(?:communications-(?:api|domain|persistence|runtime)|attachment-security|blob|mail-(?:persistence|runtime))/,
  );
  assert.doesNotMatch(
    `${smtpManifest}\n${gmailManifest}`,
    /hermes-(?:communications|attachment-security|blob|storage)/,
  );
  assert.match(runtimeManifest, /^hermes-communications-attachment-contract =/m);
  assert.match(runtimeManifest, /^hermes-communications-ingress =/m);
  assert.match(runtimeManifest, /^hermes-blob-client =/m);
  assert.doesNotMatch(
    runtimeManifest,
    /hermes-communications-(?:api|domain|persistence|runtime)/,
  );
  assert.match(mime, /MAX_OUTBOUND_ATTACHMENT_BYTES: usize = 16 \* 1024 \* 1024/);
  assert.match(mime, /MAX_OUTBOUND_RFC822_BYTES: usize = 24 \* 1024 \* 1024/);
  assert.match(mime, /Content-Type: multipart\/mixed/);
  assert.match(mime, /Content-Transfer-Encoding: base64/);
  assert.match(mime, /filename\*=UTF-8''/);
  assert.match(mime, /Sha256::digest\(&attachment\.bytes\)/);
  assert.doesNotMatch(mime, /TcpStream|UnixStream|sqlx|postgres|jetstream/i);
});

test('canonical attachment safety reaches Mail only as a durable event projection', async () => {
  const [admission, projection, runtimeManifest] = await Promise.all([
    readFile(new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/mail-runtime/src/attachment_safety_projection.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(
    admission,
    /MAIL_ATTACHMENT_SAFETY_STATE_CONSUME_CAPABILITY_ID: &str =\s*"mail\.attachment-safety-state\.consume\.v1"/,
  );
  assert.match(admission, /EventRouteDirectionV1::Consume/);
  assert.match(admission, /DurableEnvelopeKindV1::Event/);
  assert.match(
    admission,
    /communication_attachment_safety_state_changed_contract_reference_v1/,
  );
  assert.match(projection, /consume_next_attachment_safety_state_changed_v1/);
  assert.match(projection, /apply_attachment_safety_transition/);
  assert.match(projection, /exact_contract/);
  assert.match(projection, /exact_permit_contract/);
  assert.doesNotMatch(
    `${projection}\n${runtimeManifest}`,
    /hermes_(?:communications_(?:api|domain|persistence|runtime))|communications\.(?:query|request)/,
  );
});

test('Mail owns durable materialization while Blob owns attachment bytes', async () => {
  const [attachments, durable, schema, admission, managed] = await Promise.all([
    readFile(new URL('src/mail-persistence/src/attachments.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-persistence/src/durable.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-persistence/src/schema.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT), 'utf8'),
  ]);

  assert.match(attachments, /mail_attachment_safety_projections/);
  assert.match(attachments, /mail_attachment_materializations/);
  assert.match(attachments, /mail_delivery_attachment_manifest/);
  assert.match(attachments, /receipt_sha256 BYTEA NOT NULL/);
  assert.match(attachments, /blob_reference_id BYTEA NOT NULL UNIQUE/);
  assert.doesNotMatch(
    attachments,
    /BYTEA NOT NULL[^;]*(?:attachment_content|content_bytes|raw_bytes)/i,
  );
  assert.doesNotMatch(
    `${attachments}\n${durable}`,
    /hermes_data\.(?:communications|attachment_security)_/,
  );
  assert.match(durable, /request_sha256/);
  assert.match(durable, /rendered_rfc822_sha256/);
  assert.match(durable, /ORDER BY causal_sequence ASC/);
  assert.match(schema, /MAIL_STORAGE_BUNDLE_REVISION_V6/);
  assert.match(schema, /mail_communications_outbox_causal_order/);
  assert.match(admission, /BlobQuotaOperationV1::Write/);
  assert.match(admission, /BlobQuotaOperationV1::ReadRange/);
  assert.match(admission, /"mail\.attachment\.content\.v1"/);
  assert.doesNotMatch(admission, /ReadAll|foreign/i);
  assert.match(managed, /materialize_delivery_attachments/);
  assert.match(managed, /receipt_sha256/);
  assert.match(managed, /rendered_rfc822_sha256/);
});

test('live conformance inventory covers safety negatives, restart, outage and both providers', async () => {
  const [attachments, delivery, gmailOAuth, setup, fixture] = await Promise.all([
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_outbound_attachment_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_delivery_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_gmail_oauth_flow.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_smtp_fixture.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);

  assert.match(attachments, /managed_mail_delivers_only_canonical_safe_attachment/);
  assert.match(attachments, /managed_gmail_materializes_then_delivers_canonical_safe_attachment/);
  assert.match(attachments, /assert_mail_rejects_stale_and_unknown_safety_events/);
  assert.match(attachments, /SafeForDelivery/);
  assert.match(attachments, /Quarantined/);
  assert.match(attachments, /restart_mail_delivery_runtime/);
  assert.match(attachments, /assert_delivery_outcome_unknown/);
  assert.match(attachments, /extract_attachment_part/);
  assert.match(delivery, /set_authenticated_nats_container_running\(false\)/);
  assert.match(delivery, /set_authenticated_nats_container_running\(true\)/);
  assert.match(gmailOAuth, /OutcomeUnknown/);
  assert.match(setup, /storage_successor::reserve/);
  assert.match(fixture, /disconnect_after_data/);
});
