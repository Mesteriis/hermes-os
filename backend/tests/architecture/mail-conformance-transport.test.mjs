import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Mail provider transport seams are compile-time conformance-only', async () => {
  const [
    apiManifest,
    api,
    gmailManifest,
    gmail,
    imapManifest,
    imap,
    runtimeManifest,
    settings,
    harness,
  ] =
    await Promise.all([
      readFile(new URL('src/mail-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-gmail/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-gmail/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-imap/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-imap/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-runtime/src/settings.rs', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
        'utf8',
      ),
    ]);

  for (const manifest of [
    apiManifest,
    gmailManifest,
    imapManifest,
    runtimeManifest,
  ]) {
    assert.match(manifest, /\[features\]\s+default = \[\]/);
  }
  assert.match(
    runtimeManifest,
    /"hermes-mail-api\/conformance-test-support"/,
  );
  assert.match(
    runtimeManifest,
    /"hermes-mail-imap\/conformance-test-support"/,
  );
  assert.match(
    runtimeManifest,
    /"hermes-mail-gmail\/conformance-test-support"/,
  );

  assert.match(
    api,
    /pub fn valid_port[\s\S]*#\[cfg\(not\(feature = "conformance-test-support"\)\)\]/,
  );
  assert.match(api, /port == IMAP_PORT/);
  assert.match(
    imap,
    /#\[cfg\(not\(feature = "conformance-test-support"\)\)\]\s+async fn open_session/,
  );
  assert.match(imap, /TlsConnector::new\(\)/);
  assert.match(
    imap,
    /#\[cfg\(feature = "conformance-test-support"\)\]\s+async fn open_session/,
  );
  assert.match(
    imap,
    /matches!\(host, "127\.0\.0\.1" \| "::1" \| "localhost"\)/,
  );
  assert.match(api, /endpoint\.host == GMAIL_API_HOST/);
  assert.match(api, /endpoint\.port == GMAIL_API_HTTPS_PORT/);
  assert.match(api, /endpoint\.ca_certificate_pem\.is_none\(\)/);
  assert.match(api, /"127\.0\.0\.1" \| "localhost"/);
  assert.match(
    gmail,
    /#\[cfg\(any\(test, feature = "conformance-test-support"\)\)\]\s+pub fn for_conformance_endpoint/,
  );
  assert.match(gmail, /"127\.0\.0\.1" \| "localhost"/);
  assert.match(gmail, /valid_bearer_token\(access_token\)/);
  assert.match(gmail, /GMAIL_OPERATION_TIMEOUT/);
  assert.match(gmail, /TlsConnector::new\(\)\.add_root_certificate/);
  assert.doesNotMatch(`${gmail}\n${settings}`, /std::env|var_os|GMAIL_API_URL/);
  assert.match(
    harness,
    /--features',\s+'[^']*hermes-mail-runtime\/conformance-test-support[^']*'/,
  );
});

test('Mail event routes are independent capability approval units', async () => {
  const [admission, liveSetup] = await Promise.all([
    readFile(new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL(
        'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);

  for (const [symbol, capabilityId] of [
    [
      'MAIL_ATTACHMENT_ANCHOR_CONSUME_CAPABILITY_ID',
      'mail.attachment-anchor.consume.v1',
    ],
    [
      'MAIL_ATTACHMENT_BLOB_ADMISSION_PUBLISH_CAPABILITY_ID',
      'mail.attachment-blob-admission.publish.v1',
    ],
    [
      'MAIL_COMMUNICATION_OBSERVED_PUBLISH_CAPABILITY_ID',
      'mail.communication-observed.publish.v1',
    ],
  ]) {
    assert.match(
      admission,
      new RegExp(`${symbol}: &str =[\\s\\S]{0,80}"${capabilityId.replaceAll('.', '\\.')}"`),
    );
  }
  assert.doesNotMatch(`${admission}\n${liveSetup}`, /MAIL_EVENTS_CAPABILITY_ID|mail\.events\.v1/);
  assert.match(
    admission,
    /mail_attachment_anchor_consume_capability_v1\(\)[\s\S]*requests: vec!\[CapabilityRequestV1/,
  );
  assert.match(
    admission,
    /mail_attachment_blob_admission_publish_capability_v1\(\)[\s\S]*requests: vec!\[communication_attachment_blob_admission_observed_publish_request_v1\(\)\]/,
  );
  assert.match(
    admission,
    /mail_communication_observed_publish_capability_v1\(\)[\s\S]*requests: vec!\[communication_observed_publish_request_v1\(\)\]/,
  );
});
