import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const MAIL_RUNTIME_ROOT = new URL('src/mail-runtime/src/', BACKEND_ROOT);
const MAIL_PERSISTENCE_ROOT = new URL('src/mail-persistence/src/', BACKEND_ROOT);

test('Mail runtime uses one correlated managed-control frame pump', async () => {
  const [managed, main, durable] = await Promise.all([
    readFile(new URL('managed.rs', MAIL_RUNTIME_ROOT), 'utf8'),
    readFile(new URL('main.rs', MAIL_RUNTIME_ROOT), 'utf8'),
    readFile(new URL('durable.rs', MAIL_PERSISTENCE_ROOT), 'utf8'),
  ]);

  assert.match(managed, /ManagedControlChannelV2<UnixStream>/);
  assert.match(managed, /ManagedProviderCredentialClientV2/);
  assert.match(managed, /request_managed_runtime_event_access_v2/);
  assert.match(managed, /request_managed_blob_session_v2/);
  assert.match(managed, /InheritedKernelVaultRouteV2/);
  assert.match(managed, /ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS/);
  assert.doesNotMatch(managed, /\.try_clone\(/);
  assert.doesNotMatch(managed, /ManagedProviderCredentialClientV1/);
  assert.doesNotMatch(managed, /request_managed_runtime_event_access\(/);
  assert.doesNotMatch(managed, /request_managed_blob_session\(/);
  assert.doesNotMatch(managed, /MSG_PEEK/);
  assert.doesNotMatch(managed, /\.initialize\(\)/);

  assert.match(
    main,
    /Err\(MailCommunicationsOutboxRelayError::Unavailable\)\s*=>\s*\{/,
  );
  assert.match(
    main,
    /Err\(MailCommunicationsOutboxRelayError::Persistence\(_\)\)\s*=>\s*\{/,
  );

  assert.match(durable, /\.database\(binding\.access\(\)\.pool_alias\(\)\)/);
  assert.match(
    durable,
    /binding\.access\(\)\.effective_budgets\(\)\.max_connections\(\)/,
  );
  assert.doesNotMatch(durable, /\.database\(binding\.access\(\)\.database_id\(\)\)/);
});
