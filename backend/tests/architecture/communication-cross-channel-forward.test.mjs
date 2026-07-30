import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('cross-channel forward persistence is owner-local durable and bodyless', async () => {
  const [
    adr,
    inventorySource,
    policySource,
    apiManifest,
    coreManifest,
    api,
    core,
    contract,
    persistenceManifest,
    migration,
    operations,
    workQueue,
    cleanup,
    realtime,
    postgresConformance,
    storageRunner,
  ] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0346-cross-channel-communication-forward-workflow.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'architecture/communications-settings-reconstruction.json',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('architecture/policy.json', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('src/communication-cross-channel-forward-api/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-core/Cargo.toml', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-api/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL('src/communication-cross-channel-forward-core/src/lib.rs', BACKEND_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-api/proto/hermes/communication_cross_channel_forward/v1/forward.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/migrations/0001_forward_state.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/operations.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/work_queue.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/cleanup.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/communication-cross-channel-forward-persistence/src/realtime.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'tests/support/communication-cross-channel-forward/tests/postgres_live.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('scripts/test-authenticated-storage.mjs', BACKEND_ROOT), 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);
  const gate = inventory.slices.find(
    ({ gate: name }) => name === 'communication_cross_channel_forward_v1',
  );

  assert.deepEqual(gate, {
    gate: 'communication_cross_channel_forward_v1',
    role: 'workflow',
    owner: 'communication_cross_channel_forward',
    state: 'planned',
    dependsOn: [
      'communication_delivery_intent_v1',
      'communications_content_read_v1',
      'capability_routed_module_request_rpc_v1',
      'blob_v1',
    ],
  });
  assert.match(adr, /Caller не передаёт provider, account, provider chat locator или plaintext body/);
  assert.match(adr, /target-bound Blob delegation/);
  assert.match(adr, /не\s+импортирует provider contracts, SDK, runtime или persistence/);
  assert.match(adr, /Kernel[\s\S]*не декодирует source metadata, content или delivery payload/);
  assert.match(adr, /Core capability router[\s\S]*не содержит cross-channel business method/);
  assert.match(adr, /не хранит plaintext body/);
  assert.match(adr, /Принятый ADR сам по себе не\s+открывает/);
  assert.equal(
    policy.implementation.currentSlice,
    'communication_cross_channel_forward_persistence_v1',
  );
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communication_cross_channel_forward')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'hermes-communication-cross-channel-forward-api:contract',
      'hermes-communication-cross-channel-forward-core:implementation',
      'hermes-communication-cross-channel-forward-persistence:persistence',
    ],
  );
  assert.match(apiManifest, /role = "workflow"[\s\S]*surface = "contract"/);
  assert.match(coreManifest, /role = "workflow"[\s\S]*surface = "implementation"/);
  assert.match(
    persistenceManifest,
    /role = "workflow"[\s\S]*surface = "persistence"/,
  );
  assert.doesNotMatch(
    `${apiManifest}\n${coreManifest}\n${persistenceManifest}`,
    /hermes-(?:communications-domain|mail|telegram|whatsapp|zulip|kernel)/,
  );
  assert.match(api, /COMMUNICATION_CROSS_CHANNEL_FORWARD_CAPABILITY_ID_V1/);
  assert.match(core, /CrossChannelForwardTransitionV1/);
  assert.match(core, /RevisionExhausted/);
  assert.doesNotMatch(
    contract,
    /provider_id|account_id|body_utf8|blob_reference|\bAny\b|\bmap\s*</,
  );
  assert.match(migration, /communication_cross_channel_forward_operations/);
  assert.match(migration, /communication_cross_channel_forward_cleanup/);
  assert.match(migration, /communication_cross_channel_forward_realtime/);
  assert.match(migration, /attempt_count BETWEEN 0 AND 32/);
  assert.doesNotMatch(migration, /body_utf8|provider|mail_|telegram_|whatsapp_|zulip_/);
  assert.match(operations, /request_fingerprint/);
  assert.match(operations, /ON CONFLICT \(logical_owner_id, forward_id\) DO NOTHING/);
  assert.match(workQueue, /FOR UPDATE SKIP LOCKED/);
  assert.match(workQueue, /claim_epoch = operation\.claim_epoch \+ 1/);
  assert.match(workQueue, /LEAST\(attempt_count \+ 1, 32\)/);
  assert.match(cleanup, /next_cleanup/);
  assert.match(cleanup, /reschedule_cleanup/);
  assert.match(realtime, /client_realtime_window/);
  assert.match(postgresConformance, /survives_reconnect/);
  assert.match(postgresConformance, /ClaimLost/);
  assert.match(storageRunner, /HERMES_COMMUNICATION_CROSS_CHANNEL_FORWARD_POSTGRES/);
});
