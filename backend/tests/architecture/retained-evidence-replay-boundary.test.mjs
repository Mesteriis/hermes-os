import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const backendRoot = new URL('../../', import.meta.url);

async function read(path) {
  return readFile(new URL(path, backendRoot), 'utf8');
}

test('retained evidence replay protocol is an isolated workflow contract', async () => {
  const [manifest, policySource] = await Promise.all([
    read('src/attachment-preview-evidence-replay-protocol/Cargo.toml'),
    read('architecture/policy.json'),
  ]);
  const policy = JSON.parse(policySource);
  const descriptor = policy.implementation.productionPackages.find(
    ({ name }) => name === 'hermes-retained-evidence-replay-protocol',
  );

  assert.deepEqual(descriptor, {
    name: 'hermes-retained-evidence-replay-protocol',
    role: 'workflow',
    owner: 'attachment_preview_evidence_replay',
    surface: 'contract',
  });
  assert.match(manifest, /owner = "attachment_preview_evidence_replay"/);
  assert.doesNotMatch(manifest, /hermes-kernel/);
  assert.doesNotMatch(manifest, /hermes-events-jetstream/);
  assert.doesNotMatch(manifest, /sqlx/);
  assert.ok(
    policy.implementation.ownerInventory.workflows.includes(
      'attachment_preview_evidence_replay',
    ),
  );
});

test('replay selector is exact bounded and carries no generic query surface', async () => {
  const proto = await read(
    'src/attachment-preview-evidence-replay-protocol/proto/hermes/events/replay/v1/retained_evidence_replay.proto',
  );
  const implementation = await read(
    'src/attachment-preview-evidence-replay-protocol/src/lib.rs',
  );

  assert.match(proto, /string producer_registration_id = 5;/);
  assert.match(proto, /uint64 producer_runtime_generation = 6;/);
  assert.match(proto, /uint64 producer_grant_epoch = 7;/);
  assert.match(proto, /repeated bytes original_message_ids = 9;/);
  assert.match(implementation, /RETAINED_EVIDENCE_REPLAY_MAX_MESSAGES_V1: usize = 16/);
  assert.doesNotMatch(proto, /subject/);
  assert.doesNotMatch(proto, /predicate/);
  assert.doesNotMatch(proto, /payload_bytes/);
  assert.doesNotMatch(proto, /map</);
});

test('Communications retained replay persistence is an owner-local build unit', async () => {
  const [manifest, repository, migration, storageBundle, policySource] = await Promise.all([
    read('src/communications-retained-evidence-replay-persistence/Cargo.toml'),
    read('src/communications-retained-evidence-replay-persistence/src/repository.rs'),
    read(
      'src/communications-retained-evidence-replay-persistence/migrations/0001_retained_evidence_replay.sql',
    ),
    read('src/communications-runtime/src/storage_bundle.rs'),
    read('architecture/policy.json'),
  ]);
  const policy = JSON.parse(policySource);
  const descriptor = policy.implementation.productionPackages.find(
    ({ name }) =>
      name === 'hermes-communications-retained-evidence-replay-persistence',
  );

  assert.deepEqual(descriptor, {
    name: 'hermes-communications-retained-evidence-replay-persistence',
    role: 'domain',
    owner: 'communications',
    surface: 'persistence',
  });
  assert.match(manifest, /owner = "communications"/);
  assert.match(manifest, /surface = "persistence"/);
  assert.doesNotMatch(manifest, /hermes-(?:mail|attachment-security|kernel)/);
  assert.match(repository, /communications_domain_outbox/);
  assert.match(repository, /OutboxRecordV1::accept/);
  assert.match(repository, /decode_envelope_v1/);
  assert.match(repository, /COMMUNICATIONS_ATTACHMENT_LIFECYCLE_SCHEMA_SHA256/);
  assert.match(repository, /ON CONFLICT \(operation_id, original_message_id, logical_attempt, phase\) DO NOTHING/);
  assert.match(storageBundle, /append_communications_retained_evidence_replay_storage_v1/);
  assert.match(migration, /REFERENCES hermes_data\.communications_domain_outbox/);
  assert.doesNotMatch(migration, /REFERENCES hermes_data\.(?:mail|attachment_security)_/);
  assert.doesNotMatch(migration, /\b(?:UPDATE|DELETE)\b/);
});

test('Communications replay storage is an additive exact revision 17 successor', async () => {
  const schema = await read(
    'src/communications-retained-evidence-replay-persistence/src/schema.rs',
  );

  assert.match(
    schema,
    /COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1: u32 = 17/,
  );
  assert.match(schema, /predecessor\.owner_id != "communications"/);
  assert.match(schema, /predecessor\.bundle_id != "communications_state"/);
  assert.match(schema, /predecessor\.steps\.push\(StorageMigrationStepV1/);
});

test('Mail retained replay persistence is integration-owned and storage-isolated', async () => {
  const [manifest, repository, migration, runtimeBundle, assembly, policySource] =
    await Promise.all([
      read('src/mail-retained-evidence-replay-persistence/Cargo.toml'),
      read('src/mail-retained-evidence-replay-persistence/src/repository.rs'),
      read(
        'src/mail-retained-evidence-replay-persistence/migrations/0001_retained_evidence_replay.sql',
      ),
      read('src/mail-runtime/src/storage_bundle.rs'),
      read('src/mail-assembly/src/lib.rs'),
      read('architecture/policy.json'),
    ]);
  const policy = JSON.parse(policySource);
  const descriptor = policy.implementation.productionPackages.find(
    ({ name }) => name === 'hermes-mail-retained-evidence-replay-persistence',
  );

  assert.deepEqual(descriptor, {
    name: 'hermes-mail-retained-evidence-replay-persistence',
    role: 'integration',
    owner: 'mail',
    surface: 'persistence',
  });
  assert.match(manifest, /role = "integration"/);
  assert.match(manifest, /owner = "mail"/);
  assert.doesNotMatch(manifest, /hermes-communications-(?:domain|persistence|runtime)/);
  assert.doesNotMatch(manifest, /hermes-(?:attachment-security-runtime|kernel)/);
  assert.match(repository, /mail_attachment_security_outbox/);
  assert.match(repository, /OutboxRecordV1::accept/);
  assert.match(repository, /ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256/);
  assert.match(repository, /ON CONFLICT \(operation_id, original_message_id, logical_attempt, phase\) DO NOTHING/);
  assert.match(runtimeBundle, /append_mail_retained_evidence_replay_storage_v1/);
  assert.match(assembly, /mail_runtime_storage_bundle_v1/);
  assert.match(migration, /REFERENCES hermes_data\.mail_attachment_security_outbox/);
  assert.doesNotMatch(migration, /REFERENCES hermes_data\.communications_/);
  assert.doesNotMatch(migration, /\b(?:UPDATE|DELETE)\b/);
});

test('Mail replay storage is an additive exact revision 23 successor', async () => {
  const schema = await read('src/mail-retained-evidence-replay-persistence/src/schema.rs');

  assert.match(
    schema,
    /MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1: u32 = 23/,
  );
  assert.match(schema, /predecessor\.revision != MAIL_RETAINED_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1 - 1/);
  assert.match(schema, /predecessor\.owner_id != "mail"/);
  assert.match(schema, /predecessor\.bundle_id != "mail_state"/);
  assert.match(schema, /predecessor\.steps\.push\(StorageMigrationStepV1/);
});

test('producer replay routes are separate owner-specific contract units', async () => {
  const [communicationsManifest, communicationsContract, mailManifest, mailContract] =
    await Promise.all([
      read('src/communications-retained-evidence-replay-contract/Cargo.toml'),
      read('src/communications-retained-evidence-replay-contract/src/lib.rs'),
      read('src/mail-retained-evidence-replay-contract/Cargo.toml'),
      read('src/mail-retained-evidence-replay-contract/src/lib.rs'),
    ]);

  assert.match(communicationsManifest, /role = "domain"/);
  assert.match(communicationsManifest, /owner = "communications"/);
  assert.doesNotMatch(communicationsManifest, /hermes-(?:mail|retained-evidence-replay-protocol)/);
  assert.match(mailManifest, /role = "integration"/);
  assert.match(mailManifest, /owner = "mail"/);
  assert.doesNotMatch(mailManifest, /hermes-(?:communications|retained-evidence-replay-protocol)/);
  assert.match(communicationsContract, /communications_retained_evidence_replay_command/);
  assert.match(communicationsContract, /communications_retained_evidence_replay_result/);
  assert.match(mailContract, /mail_retained_evidence_replay_command/);
  assert.match(mailContract, /mail_retained_evidence_replay_result/);
  assert.doesNotMatch(communicationsContract, /mail_/);
  assert.doesNotMatch(mailContract, /communications_/);
});

test('producer adapters publish only verified original bytes with append-only audit', async () => {
  const [communications, mail] = await Promise.all([
    read('src/communications-runtime/src/retained_evidence_replay.rs'),
    read('src/mail-runtime/src/retained_evidence_replay.rs'),
  ]);
  for (const adapter of [communications, mail]) {
    assert.match(adapter, /producer_registration_id != context\.registration_id/);
    assert.match(adapter, /producer_runtime_generation != context\.runtime_generation/);
    assert.match(adapter, /producer_grant_epoch != context\.grant_epoch/);
    assert.match(
      adapter,
      /publish_exact\(\s*original_contract_publish_permit,\s*retained\.record\.exact_bytes\(\),?\s*\)/,
    );
    assert.match(adapter, /ReplayPhaseV1::Authorized/);
    assert.match(adapter, /ReplayPhaseV1::Published/);
    assert.match(adapter, /ReplayPhaseV1::PublishUnavailable/);
    assert.doesNotMatch(adapter, /mark_.*published|published_at|decode_envelope/);
  }
  assert.match(communications, /retained_attachment_safety_event_by_message_id/);
  assert.match(mail, /retained_scan_candidate_by_message_id/);
  assert.doesNotMatch(communications, /hermes_mail|mail_/);
  assert.doesNotMatch(mail, /hermes_communications|communications_/);
});

test('producer replay delivery is durable owner-local and never rewrites source evidence', async () => {
  const [communicationsDelivery, communicationsMigration, communicationsBundle,
    mailDelivery, mailMigration, mailBundle] = await Promise.all([
    read('src/communications-retained-evidence-replay-persistence/src/delivery.rs'),
    read(
      'src/communications-retained-evidence-replay-persistence/migrations/0002_retained_evidence_replay_delivery.sql',
    ),
    read('src/communications-runtime/src/storage_bundle.rs'),
    read('src/mail-retained-evidence-replay-persistence/src/delivery.rs'),
    read(
      'src/mail-retained-evidence-replay-persistence/migrations/0002_retained_evidence_replay_delivery.sql',
    ),
    read('src/mail-runtime/src/storage_bundle.rs'),
  ]);

  for (const [delivery, migration] of [
    [communicationsDelivery, communicationsMigration],
    [mailDelivery, mailMigration],
  ]) {
    assert.match(delivery, /command_message_id: \[u8; 16\]/);
    assert.match(delivery, /command_envelope_sha256: \[u8; 32\]/);
    assert.match(delivery, /operation_id: \[u8; 16\]/);
    assert.match(delivery, /FOR UPDATE/);
    assert.match(delivery, /OutboxRecordV1::accept/);
    assert.match(delivery, /decode_envelope_v1/);
    assert.match(delivery, /SET state = 1/);
    assert.match(delivery, /published_at_unix_seconds IS NULL/);
    assert.match(migration, /command_inbox/);
    assert.match(migration, /result_outbox/);
    assert.match(migration, /exact_envelope_bytes/);
    assert.doesNotMatch(migration, /UPDATE hermes_data/);
    assert.doesNotMatch(delivery, /domain_outbox|attachment_security_outbox/);
  }

  assert.match(
    communicationsBundle,
    /append_communications_retained_evidence_replay_delivery_storage_v1/,
  );
  assert.match(mailBundle, /append_mail_retained_evidence_replay_delivery_storage_v1/);
  assert.doesNotMatch(communicationsDelivery, /hermes_mail|mail_/);
  assert.doesNotMatch(mailDelivery, /hermes_communications|communications_/);
});

test('replay delivery migrations are additive exact successor revisions', async () => {
  const [communicationsSchema, mailSchema] = await Promise.all([
    read('src/communications-retained-evidence-replay-persistence/src/schema.rs'),
    read('src/mail-retained-evidence-replay-persistence/src/schema.rs'),
  ]);

  assert.match(
    communicationsSchema,
    /COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1: u32 = 18/,
  );
  assert.match(
    mailSchema,
    /MAIL_RETAINED_EVIDENCE_REPLAY_DELIVERY_STORAGE_BUNDLE_REVISION_V1: u32 = 24/,
  );
  assert.match(
    communicationsSchema,
    /append_communications_retained_evidence_replay_delivery_storage_v1/,
  );
  assert.match(mailSchema, /append_mail_retained_evidence_replay_delivery_storage_v1/);
});

test('producer contracts build exact workflow commands and causal terminal results', async () => {
  const [communications, mail] = await Promise.all([
    read('src/communications-retained-evidence-replay-contract/src/envelope.rs'),
    read('src/mail-retained-evidence-replay-contract/src/envelope.rs'),
  ]);

  for (const source of [communications, mail]) {
    assert.match(source, /Semantics::Command\(CommandMetadataV1/);
    assert.match(source, /kind: ActorKindV1::OwnerDevice/);
    assert.match(source, /SOURCE_MODULE_ID_V1[\s\S]{0,80}\.as_bytes\(\)/);
    assert.match(source, /target_capability: .*REPLAY_CAPABILITY_ID_V1/);
    assert.match(source, /Semantics::Result\(ResultMetadataV1/);
    assert.match(source, /causation_message_id: command_message_id\.to_vec\(\)/);
    assert.match(source, /validate_envelope_v1/);
    assert.match(source, /OutboxRecordV1::accept/);
    assert.doesNotMatch(source, /subject|predicate|payload_bytes|map</);
  }
  assert.doesNotMatch(communications, /hermes_mail|mail_/);
  assert.doesNotMatch(mail, /hermes_communications|communications_/);
});

test('producer consumers persist terminal result before Ack and retry infrastructure outage', async () => {
  const [communicationsConsumer, communicationsRelay, mailConsumer, mailRelay,
    communicationsManagedFixture, mailManagedFixture, testkitManifest] =
    await Promise.all([
      read('src/communications-runtime/src/retained_evidence_replay_consumer.rs'),
      read('src/communications-runtime/src/retained_evidence_replay_result.rs'),
      read('src/mail-runtime/src/retained_evidence_replay_consumer.rs'),
      read('src/mail-runtime/src/retained_evidence_replay_result.rs'),
      read('tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/communications_setup.rs'),
      read('tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/mail_managed_setup.rs'),
      read('tests/support/kernel-recovery/Cargo.toml'),
    ]);

  for (const source of [communicationsConsumer, mailConsumer]) {
    assert.match(source, /try_receive_runtime_pull_delivery/);
    assert.match(source, /return Ok\(None\)/);
    const accept = source.indexOf('.accept_replay_command(');
    const replay = source.indexOf('replay_retained_', accept);
    const complete = source.indexOf('.complete_replay_command(', replay);
    const acknowledge = source.indexOf('.acknowledge()');
    assert.ok(accept >= 0 && replay > accept && complete > replay);
    assert.ok(acknowledge >= 0);
    assert.match(
      source,
      /let outcome = accept_[\s\S]+?\.await\?;[\s\S]+?delivery\s*\.acknowledge\(\)/,
    );
    assert.match(source, /DuplicateCompleted/);
    assert.match(source, /PublishUnavailable => return None/);
    assert.match(source, /StorageUnavailable[\s\S]{0,80}return None/);
    assert.match(source, /ReplayRetryable/);
    assert.match(source, /SOURCE_MODULE_ID_V1/);
    assert.doesNotMatch(source, /UPDATE|DELETE|domain_outbox|attachment_security_outbox/);
  }

  for (const relay of [communicationsRelay, mailRelay]) {
    assert.match(relay, /pending_replay_results\(1\)/);
    assert.match(relay, /publish_exact\(permit, record\.exact_bytes\(\)\)/);
    assert.ok(
      relay.indexOf('publish_exact') < relay.indexOf('mark_replay_result_published'),
    );
  }
  assert.doesNotMatch(communicationsConsumer, /hermes_mail|mail_/);
  assert.doesNotMatch(mailConsumer, /hermes_communications|communications_/);
  assert.match(communicationsManagedFixture, /COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID/);
  assert.match(communicationsManagedFixture, /communications_replay_command_contract_reference_v1/);
  assert.match(communicationsManagedFixture, /communications_replay_result_contract_reference_v1/);
  assert.match(mailManagedFixture, /MAIL_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID/);
  assert.match(mailManagedFixture, /MAIL_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1/);
  assert.match(mailManagedFixture, /mail_runtime_storage_bundle_v1/);
  assert.match(testkitManifest, /hermes-communications-retained-evidence-replay-contract/);
});

test('replay coordination is a separate workflow with owner-local operation storage', async () => {
  const [api, coreManifest, persistenceManifest, migration, runtimeManifest, assemblyManifest] =
    await Promise.all([
      read('src/attachment-preview-evidence-replay-api/proto/hermes/attachment_preview_evidence_replay/v1/replay.proto'),
      read('src/attachment-preview-evidence-replay-core/Cargo.toml'),
      read('src/attachment-preview-evidence-replay-persistence/Cargo.toml'),
      read('src/attachment-preview-evidence-replay-persistence/migrations/0001_attachment_preview_evidence_replay.sql'),
      read('src/attachment-preview-evidence-replay-runtime/Cargo.toml'),
      read('src/attachment-preview-evidence-replay-assembly/Cargo.toml'),
    ]);

  for (const manifest of [coreManifest, persistenceManifest, runtimeManifest, assemblyManifest]) {
    assert.match(manifest, /role = "workflow"/);
    assert.match(manifest, /owner = "attachment_preview_evidence_replay"/);
    assert.doesNotMatch(manifest, /hermes-(?:communications-runtime|mail-runtime|kernel)/);
  }
  assert.doesNotMatch(api, /logical_owner_id|owner_device_actor|subject|predicate|payload_bytes|map</);
  assert.match(api, /ReplayProducerSelectionV1 communications/);
  assert.match(api, /ReplayProducerSelectionV1 mail/);
  assert.match(migration, /attachment_preview_evidence_replay_command_outbox/);
  assert.match(migration, /attachment_preview_evidence_replay_result_inbox/);
  assert.doesNotMatch(migration, /communications_domain_outbox|mail_attachment_security_outbox/);
  assert.doesNotMatch(migration, /provider|subject|payload_bytes|blob/);
});

test('workflow command and result delivery preserves exact bytes and commit-before-Ack', async () => {
  const [client, persistence, relay, consumer, admission] = await Promise.all([
    read('src/attachment-preview-evidence-replay-runtime/src/client_port.rs'),
    read('src/attachment-preview-evidence-replay-persistence/src/repository.rs'),
    read('src/attachment-preview-evidence-replay-runtime/src/outbox.rs'),
    read('src/attachment-preview-evidence-replay-runtime/src/result_consumer.rs'),
    read('src/attachment-preview-evidence-replay-runtime/src/admission.rs'),
  ]);

  assert.match(client, /module_request\.logical_owner_id\.clone\(\)/);
  assert.match(client, /authenticated_device_id/);
  assert.match(client, /build_communications_replay_command_outbox_v1/);
  assert.match(client, /build_mail_replay_command_outbox_v1/);
  assert.ok(client.indexOf('command_records(&request') < client.indexOf('.create_operation('));
  assert.match(persistence, /ON CONFLICT \(operation_id\) DO NOTHING/);
  assert.match(persistence, /FOR UPDATE/);
  assert.match(persistence, /request_fingerprint_v1/);
  assert.match(relay, /publish_exact\(permit, &command\.exact_envelope_bytes\)/);
  assert.ok(relay.indexOf('publish_exact') < relay.indexOf('mark_command_published'));
  assert.ok(consumer.indexOf('.accept_producer_result(') < consumer.indexOf('.acknowledge()'));
  assert.match(admission, /communications_replay_command_publish_request_v1/);
  assert.match(admission, /mail_replay_command_publish_request_v1/);
  assert.match(admission, /communications_replay_result_consume_request_v1/);
  assert.match(admission, /mail_replay_result_consume_request_v1/);
  assert.match(consumer, /try_receive_runtime_pull_delivery/);
  assert.doesNotMatch(persistence, /communications_domain_outbox|mail_attachment_security_outbox/);
});

test('replay workflow is an admitted managed runtime with exact event grants', async () => {
  const [managed, main, communicationsAdmission, communicationsRuntime, mailAdmission, mailRuntime,
    developmentRelease, developmentAssembly, managedSetup, managedFlow, managedScript] = await Promise.all([
    read('src/attachment-preview-evidence-replay-runtime/src/managed_runtime.rs'),
    read('src/attachment-preview-evidence-replay-runtime/src/main.rs'),
    read('src/communications-runtime/src/admission.rs'),
    read('src/communications-runtime/src/event_runtime.rs'),
    read('src/mail-runtime/src/admission.rs'),
    read('src/mail-runtime/src/managed.rs'),
    read('scripts/materialize-dev-release.sh'),
    read('development/assembly/src/main.rs'),
    read('tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_evidence_replay_managed_setup.rs'),
    read('tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/attachment_preview_evidence_replay_managed_flow.rs'),
    read('scripts/test-authenticated-storage.mjs'),
  ]);

  assert.match(main, /"serve-inherited" => serve_inherited/);
  assert.match(main, /validate_managed_workflow_runtime_configuration/);
  assert.match(main, /inherited_control_channel/);
  assert.match(managed, /StorageVaultLeaseAdapterV1/);
  assert.match(managed, /request_managed_runtime_event_access_v2/);
  assert.match(managed, /if permits\.len\(\) != 2/);
  assert.match(managed, /communications_replay_result_contract_reference_v1/);
  assert.match(managed, /mail_replay_result_contract_reference_v1/);
  assert.match(managed, /Operation::ClientDelivery/);
  assert.match(managed, /dispatch_replay_client_request_v1/);
  assert.match(managed, /relay_replay_commands_once_v1/);
  assert.doesNotMatch(managed, /hermes_(?:communications_runtime|mail_runtime|kernel)/);
  assert.match(communicationsAdmission, /communications_replay_command_consume_request_v1/);
  assert.match(communicationsAdmission, /communications_replay_result_publish_request_v1/);
  assert.match(communicationsRuntime, /consume_next_communications_replay_command_v1/);
  assert.match(communicationsRuntime, /relay_communications_replay_result_once_v1/);
  assert.match(mailAdmission, /mail_replay_command_consume_request_v1/);
  assert.match(mailAdmission, /mail_replay_result_publish_request_v1/);
  assert.match(mailRuntime, /consume_next_mail_replay_command_v1/);
  assert.match(mailRuntime, /relay_mail_replay_result_once_v1/);
  assert.match(developmentRelease, /hermes-attachment-preview-evidence-replay-assembly/);
  assert.match(developmentAssembly, /ModuleRuntimeKindV1::Workflow/);
  assert.match(developmentAssembly, /ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT/);
  assert.match(managedSetup, /attachment_preview_evidence_replay_storage_bundle_v1/);
  assert.match(managedSetup, /start_reserved_workflow/);
  assert.match(managedSetup, /storage_successor::reserve/);
  assert.match(managedFlow, /managed_attachment_preview_evidence_replay_runtime_starts_with_exact_signed_contracts/);
  assert.match(managedFlow, /runtime_generation, 2/);
  assert.match(managedScript, /HERMES_ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_BIN/);
});
