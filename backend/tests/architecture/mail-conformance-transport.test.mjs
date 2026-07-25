import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

test('Mail plaintext IMAP transport is compile-time conformance-only', async () => {
  const [apiManifest, api, imapManifest, imap, runtimeManifest, harness] =
    await Promise.all([
      readFile(new URL('src/mail-api/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-api/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-imap/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-imap/src/lib.rs', BACKEND_ROOT), 'utf8'),
      readFile(new URL('src/mail-runtime/Cargo.toml', BACKEND_ROOT), 'utf8'),
      readFile(
        new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT),
        'utf8',
      ),
    ]);

  for (const manifest of [apiManifest, imapManifest, runtimeManifest]) {
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
  assert.match(
    harness,
    /--features',\s+'hermes-mail-runtime\/conformance-test-support'/,
  );
});
