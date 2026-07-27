import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0298-mail-operational-read-projection-and-client-contract.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  proto: new URL(
    'src/mail-api/proto/hermes/mail/operational/v1/client.proto',
    BACKEND_ROOT,
  ),
  validator: new URL('src/mail-api/src/operational.rs', BACKEND_ROOT),
  wire: new URL('src/mail-api/src/operational_wire.rs', BACKEND_ROOT),
  contract: new URL('src/mail-api/src/client_contract.rs', BACKEND_ROOT),
  persistence: new URL(
    'src/mail-persistence/src/operational.rs',
    BACKEND_ROOT,
  ),
  runtime: new URL('src/mail-runtime/src/managed.rs', BACKEND_ROOT),
  clientPort: new URL('src/mail-runtime/src/client_port.rs', BACKEND_ROOT),
  admission: new URL('src/mail-runtime/src/admission.rs', BACKEND_ROOT),
  managedSetup: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs',
    BACKEND_ROOT,
  ),
  managedFlow: new URL(
    'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_operational_flow.rs',
    BACKEND_ROOT,
  ),
  build: new URL('src/mail-api/build.rs', BACKEND_ROOT),
  api: new URL('src/mail-api/src/lib.rs', BACKEND_ROOT),
};

test('Mail operational read contract is typed, bounded and not falsely admitted', async () => {
  const [
    adr,
    inventorySource,
    proto,
    validator,
    wire,
    contract,
    persistence,
    runtime,
    clientPort,
    admission,
    managedSetup,
    managedFlow,
    build,
    api,
  ] = await Promise.all(
    Object.values(paths).map((path) => readFile(path, 'utf8')),
  );
  const inventory = JSON.parse(inventorySource);
  const slice = inventory.slices.find(
    ({ gate }) => gate === 'mail_operational_read_v1',
  );

  assert.deepEqual(slice, {
    gate: 'mail_operational_read_v1',
    role: 'integration',
    owner: 'mail',
    state: 'planned',
    dependsOn: [
      'client_gateway_v1',
      'mail_account_lifecycle_v1',
      'mail.sync.v1',
    ],
  });
  assert.match(
    adr,
    /owner-local persistence,\s+bounded scoped queries, атомарная IMAP\/Gmail sync materialization/,
  );
  assert.match(
    adr,
    /exact\s+runtime client route и managed Gateway conformance реализованы/,
  );
  assert.match(
    adr,
    /managed Gateway conformance реализованы и подтверждены\s+live host contour/,
  );
  assert.match(adr, /frontend cutover ещё не завершён/);
  assert.match(adr, /Core Gateway[\s\S]*не декодирует Mail payload/);
  assert.match(adr, /Mail does not import Communications/);
  assert.match(adr, /full body[\s\S]*communications_content_read_v1/);
  assert.match(adr, /Runtime is not assembly/);

  assert.match(proto, /package hermes\.mail\.operational\.v1/);
  assert.match(
    proto,
    /oneof query[\s\S]*list_folders[\s\S]*list_threads[\s\S]*list_messages[\s\S]*get_message/,
  );
  assert.match(proto, /service MailOperationalQueryService/);
  assert.match(proto, /bytes observation_anchor_id = 13/);
  assert.doesNotMatch(
    proto,
    /\b(?:password|secret|token|cookie|raw_mime|html|provider_cursor|metadata)\b/i,
  );
  assert.doesNotMatch(proto, /\bmap\s*</);

  assert.match(validator, /MAX_OPERATIONAL_PAGE_SIZE: u32 = 200/);
  assert.match(validator, /validate_operational_query/);
  assert.match(validator, /validate_operational_response/);
  assert.match(validator, /limit == 0 \|\| limit > MAX_OPERATIONAL_PAGE_SIZE/);
  assert.match(wire, /encode_operational_query/);
  assert.match(wire, /decode_operational_query_response/);
  assert.match(wire, /encode_operational_query_response\(&response\)\? != bytes/);
  assert.match(contract, /MAIL_CLIENT_CONTRACT_REVISION: u32 = 6/);
  assert.match(contract, /mail\.operational\.query\.v1/);
  assert.match(
    contract,
    /\/hermes\.mail\.operational\.v1\.MailOperationalQueryService\/Query/,
  );
  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS hermes_data\.mail_operational_folders/,
  );
  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS hermes_data\.mail_operational_threads/,
  );
  assert.match(
    persistence,
    /CREATE TABLE IF NOT EXISTS hermes_data\.mail_operational_messages/,
  );
  assert.match(persistence, /require_cursor_anchor/);
  assert.match(
    persistence,
    /record_operational_materializations_in_transaction/,
  );
  assert.match(runtime, /ProviderProvenanceV1::MailImap[\s\S]*MailOperationalMaterializationV1/);
  assert.match(runtime, /ProviderProvenanceV1::MailGmail[\s\S]*MailOperationalMaterializationV1/);
  assert.match(
    runtime,
    /operational_query_connection_id\(query\) != self\.account\.connection_id/,
  );
  assert.match(clientPort, /MailClientRequestV1::OperationalQuery/);
  assert.match(clientPort, /operational_wire::encode_operational_query/);
  assert.match(clientPort, /operational_wire::decode_operational_query_response/);
  assert.match(admission, /MailClientContractV1::OperationalQuery/);
  assert.match(managedSetup, /MAIL_STORAGE_BUNDLE_REVISION_V9/);
  assert.match(
    managedSetup,
    /MailClientContractV1::OperationalQuery[\s\S]*MailClientContractV1::Sync/,
  );
  assert.match(managedFlow, /assert_cross_account_query_is_rejected/);
  assert.match(managedFlow, /assert_cursor_scope_is_enforced/);
  assert.match(managedFlow, /assert_stale_cursor_is_rejected/);
  assert.match(managedFlow, /assert_stale_operational_generation_is_rejected/);
  assert.match(managedFlow, /managed-mail-imap-password/);
  assert.match(
    build,
    /proto\/hermes\/mail\/operational\/v1\/client\.proto/,
  );
  assert.match(api, /pub mod operational;/);
  assert.doesNotMatch(
    `${proto}\n${validator}\n${wire}\n${persistence}`,
    /hermes_communications|domains\/communications|mail-runtime|mail-persistence|hermes-kernel/i,
  );
});
