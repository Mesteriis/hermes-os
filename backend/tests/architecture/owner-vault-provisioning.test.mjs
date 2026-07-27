import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  adr: new URL(
    'docs/adr/ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md',
    PROJECT_ROOT,
  ),
  inventory: new URL(
    'architecture/communications-settings-reconstruction.json',
    BACKEND_ROOT,
  ),
  command: new URL(
    'src/platform/vault/protocol/src/operations/command.rs',
    BACKEND_ROOT,
  ),
  receipt: new URL(
    'src/platform/vault/protocol/src/operations/provisioning.rs',
    BACKEND_ROOT,
  ),
  service: new URL(
    'src/platform/vault/runtime/src/service/runtime.rs',
    BACKEND_ROOT,
  ),
  persistence: new URL(
    'src/platform/vault/store_sqlcipher/src/actor/provisioning.rs',
    BACKEND_ROOT,
  ),
  schema: new URL(
    'src/platform/vault/store_sqlcipher/src/database/store.rs',
    BACKEND_ROOT,
  ),
};

test('owner Vault provisioning primitive is write-only durable and platform-neutral', async () => {
  const [adr, inventorySource, command, receipt, service, persistence, schema] =
    await Promise.all([
      readFile(paths.adr, 'utf8'),
      readFile(paths.inventory, 'utf8'),
      readFile(paths.command, 'utf8'),
      readFile(paths.receipt, 'utf8'),
      readFile(paths.service, 'utf8'),
      readFile(paths.persistence, 'utf8'),
      readFile(paths.schema, 'utf8'),
    ]);
  const inventory = JSON.parse(inventorySource);
  const backend = inventory.slices.find(
    ({ gate }) => gate === 'owner_vault_provisioning_backend_v1',
  );

  assert.deepEqual(backend, {
    gate: 'owner_vault_provisioning_backend_v1',
    role: 'platform',
    owner: 'vault',
    state: 'planned',
    dependsOn: ['client_gateway_v1', 'vault_v1'],
  });
  assert.match(command, /ProvisionLease/);
  assert.match(command, /operation_id: \[u8; 16\]/);
  assert.match(receipt, /VaultProvisioningReceiptV1/);
  assert.match(receipt, /secret_revision/);
  assert.doesNotMatch(receipt, /record_id|payload|credential/);
  assert.match(service, /provision_current_once/);
  assert.match(persistence, /vault_owner_provisioning_receipts/);
  assert.match(persistence, /transaction\.commit/);
  assert.match(persistence, /expected_intent_digest/);
  assert.match(schema, /CREATE TABLE vault_owner_provisioning_receipts/);
  assert.match(adr, /Prepare[\s\S]*Authorize[\s\S]*Commit/);
  assert.doesNotMatch(
    `${command}\n${receipt}\n${service}\n${persistence}`,
    /hermes_(?:mail|telegram|whatsapp|zulip|communications)|Mail|Telegram|WhatsApp|Zulip/,
  );
});
