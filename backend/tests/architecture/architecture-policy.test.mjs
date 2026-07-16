import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { validatePolicy } from '../../scripts/lib/policy-schema.mjs';
import { collectSourceEntries } from '../../scripts/lib/repository-scan.mjs';
import { validateSourceEntries } from '../../scripts/lib/source-boundaries.mjs';
import { canonicalPolicyForTests as policy } from './support/canonical-policy.mjs';

function codes(violations) {
  return new Set(violations.map(({ code }) => code));
}

test('accepts the canonical registry and current development allowlist', () => {
  assert.deepEqual(validatePolicy(policy()), []);
});

test('requires architecture policy schema version 2', () => {
  const invalid = policy();
  invalid.schemaVersion = 1;

  assert.ok(codes(validatePolicy(invalid)).has('policy_schema'));
});

test('requires an exact current implementation slice inventory', () => {
  const invalid = policy();
  delete invalid.implementation;

  assert.ok(codes(validatePolicy(invalid)).has('implementation_slice_policy'));
});

test('keeps the current slice recovery-only and closed to undeclared production packages', () => {
  const mutations = [
    (implementation) => { implementation.currentSlice = 'module_control_plane_v1'; },
    (implementation) => { implementation.productionPackages.push({
      name: 'hermes-storage-protocol', role: 'platform', owner: 'storage', surface: 'contract',
    }); },
    (implementation) => { implementation.ownerInventory.domains.push('communications'); },
    (implementation) => { implementation.ownerInventory.businessCapabilities.push('client_rpc'); },
    (implementation) => { implementation.workspaceDependencyAllowlist['hermes-kernel'].push('hermes-events-protocol'); },
    (implementation) => { implementation.thirdPartyDependencyAllowlist['hermes-kernel'].push({ name: 'reqwest', kind: 'normal', source: 'crates_io' }); },
    (implementation) => { implementation.forbiddenDependencies.pop(); },
    (implementation) => { implementation.forbiddenDependencyPrefixes.push('hermes-ai-'); },
    (implementation) => { implementation.cargoFeaturesEnabled = true; },
    (implementation) => { implementation.developmentProfile.networkListenerEnabled = false; },
    (implementation) => { implementation.developmentProfile.productionGateEvidenceAllowed = true; },
    (implementation) => { implementation.developmentProfile.privateKeyStorage = 'keychain'; },
    (implementation) => { implementation.developmentProfile.automaticProductionFallbackAllowed = true; },
    (implementation) => { implementation.developmentProfile.simulatedTargets.pop(); },
    (implementation) => { implementation.targetPolicy['hermes-kernel'].primaryKind = 'lib'; },
    (implementation) => { implementation.kernelProfile.maximumState = 'ready'; },
    (implementation) => { implementation.kernelProfile.allowedStates.push('ready'); },
    (implementation) => { implementation.kernelProfile.forbiddenStates.pop(); },
    (implementation) => { implementation.kernelProfile.activeComponents.push('event_hub'); },
    (implementation) => { implementation.kernelProfile.externalServices.push('postgresql'); },
    (implementation) => { implementation.kernelProfile.networkListenerEnabled = true; },
    (implementation) => { implementation.kernelProfile.clock.moduleCapabilityEnabled = true; },
    (implementation) => { implementation.exitGates.pop(); },
    (implementation) => { implementation.compatibilityMode = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.implementation);
    assert.ok(codes(validatePolicy(invalid)).has('implementation_slice_policy'));
  }
});

test('requires fail-closed phase gates before expanding the recovery-only slice', () => {
  const invalid = policy();
  delete invalid.phaseGates;

  assert.ok(codes(validatePolicy(invalid)).has('phase_gate_policy'));
});

test('requires exact evidence fields before authorizing every later phase', () => {
  const mutations = [
    (phaseGates) => { phaseGates.transitionAuthority = 'runtime_flag'; },
    (phaseGates) => { phaseGates.notAuthorized.shift(); },
    (phaseGates) => { phaseGates.notAuthorized.push('provider_v1'); },
    (phaseGates) => { phaseGates.requiredDecisionFields.nats_data_plane_v1.pop(); },
    (phaseGates) => { delete phaseGates.requiredDecisionFields.blob_v1; },
    (phaseGates) => { phaseGates.requires.first_owner_v1 = []; },
    (phaseGates) => { phaseGates.requires.storage_control_v1.pop(); },
    (phaseGates) => { phaseGates.requires.nats_data_plane_v1.pop(); },
    (phaseGates) => { phaseGates.requires.scheduler_v1.pop(); },
    (phaseGates) => { phaseGates.requires.client_gateway_v1.pop(); },
    (phaseGates) => { delete phaseGates.conditionalRequires.first_owner_v1.scheduler_v1; },
    (phaseGates) => { delete phaseGates.conditionalRequires.whole_instance_backup_v1.scheduler_v1; },
    (phaseGates) => { phaseGates.overrideAllowed = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.phaseGates);
    assert.ok(codes(validatePolicy(invalid)).has('phase_gate_policy'));
  }
});

test('requires AI context to be supplied by explicit use-case workflows', () => {
  const invalid = policy();
  delete invalid.aiContext;

  assert.ok(codes(validatePolicy(invalid)).has('ai_context_policy'));
});

test('prevents AI read-all access, generic context APIs and durable context projections', () => {
  const mutations = [
    (invalid) => { invalid.aiContext.acquisitionMode = 'direct_database_read'; },
    (invalid) => { invalid.aiContext.aiDirectOwnerQueryAccessEnabled = true; },
    (invalid) => { invalid.aiContext.aiDirectCrossOwnerQueryOrchestrationEnabled = true; },
    (invalid) => { invalid.aiContext.aiCrossOwnerSqlEnabled = true; },
    (invalid) => { invalid.aiContext.genericContextApiEnabled = true; },
    (invalid) => { invalid.aiContext.durableContextProjectionEnabled = true; },
    (invalid) => { invalid.aiContext.wireShape = 'global_fragment_union'; },
    (invalid) => { invalid.aiContext.globalFragmentUnionEnabled = true; },
    (invalid) => { invalid.aiContext.opaquePayloadBytesEnabled = true; },
    (invalid) => { delete invalid.aiContext.schemaBinding; },
    (invalid) => { invalid.aiContext.remoteEgressRequiresExplicitPolicy = false; },
    (invalid) => { invalid.projections.blockedOwners = invalid.projections.blockedOwners.filter((owner) => owner !== 'context'); },
    (invalid) => { invalid.aiContext.readOnlyGrantsAllowed = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid);
    assert.ok(codes(validatePolicy(invalid)).has('ai_context_policy'));
  }
});

test('requires registered domains to be partitioned between allowed and blocked', () => {
  const invalid = policy();
  invalid.domains.developmentAllowlist = invalid.domains.developmentAllowlist.filter(
    (owner) => owner !== 'ai',
  );

  assert.ok(codes(validatePolicy(invalid)).has('domain_partition'));
});

test('requires Event Hub, Telemetry control and Settings Registry to remain constitutional Kernel components', () => {
  for (const requiredComponent of ['event_hub', 'telemetry_control', 'settings_registry']) {
    const invalid = policy();
    invalid.kernel.constitutionalComponents = invalid.kernel.constitutionalComponents.filter(
      (component) => component !== requiredComponent,
    );

    assert.ok(codes(validatePolicy(invalid)).has('kernel_components'));
  }
});

test('requires Settings Registry to remain exclusive to Kernel', () => {
  const invalid = policy();
  invalid.kernel.exclusiveComponents = invalid.kernel.exclusiveComponents.filter(
    (component) => component !== 'settings_registry',
  );

  assert.ok(codes(validatePolicy(invalid)).has('kernel_components'));
});

test('requires an exact registry of Kernel-owned packages', () => {
  const invalid = policy();
  delete invalid.kernel.packages;

  assert.ok(codes(validatePolicy(invalid)).has('kernel_package_policy'));
});

test('requires zero-config bootstrap with only OS-default and explicit CLI data-dir sources', () => {
  const mutations = [
    (bootstrap) => { bootstrap.configurationFileRequired = true; },
    (bootstrap) => { bootstrap.dataDirectorySources = ['os_standard_default']; },
    (bootstrap) => { bootstrap.dataDirectorySources.push('environment'); },
    (bootstrap) => { bootstrap.requiredExternalServicesBeforeRecoveryOnly.push('postgresql'); },
    (bootstrap) => { delete bootstrap.requiredExternalServicesBeforeRecoveryOnly; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.bootstrap);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_bootstrap_policy'));
  }
});

test('keeps Control Store failure in bounded local recovery', () => {
  const mutations = [
    (bootstrap) => { bootstrap.controlStoreUnavailable.state = 'ready'; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.transport = 'http'; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.onlineOperations = ['validate', 'export']; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.onlineOperations.push('restore'); },
    (bootstrap) => { bootstrap.controlStoreUnavailable.lifecycleOperations = []; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.lifecycleOperations.push('restart'); },
    (bootstrap) => { bootstrap.controlStoreUnavailable.onlineMutationsEnabled = true; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.offlineOperations = ['restore']; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.offlineOperations.push('wipe'); },
    (bootstrap) => { bootstrap.controlStoreUnavailable.kernelStoppedRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.exclusiveLockRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.explicitDataDirectoryRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.interactiveConfirmationRequired = false; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.businessDataPlaneEnabled = true; },
    (bootstrap) => { bootstrap.controlStoreUnavailable.automaticResetEnabled = true; },
    (bootstrap) => { delete bootstrap.controlStoreUnavailable; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.bootstrap);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_recovery_policy'));
  }
});

test('requires per-device owner identity without shared secrets or OS-identity trust', () => {
  const mutations = [
    (identity) => { identity.ownerModel = 'shared_root_key'; },
    (identity) => { identity.deviceKeySuite = 'client_selected'; },
    (identity) => { identity.devicePrivateKeyBoundary = 'kernel'; },
    (identity) => { identity.devicePublicKeyRegistry = 'platform_signer'; },
    (identity) => { identity.initialDesktopEnrollment = 'public_endpoint'; },
    (identity) => { identity.clientSessionStorage = 'control_store'; },
    (identity) => { identity.osIdentityAloneAuthorizesOwner = true; },
    (identity) => { identity.sharedOwnerSecretAllowed = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.identity);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_identity_policy'));
  }

  const missing = policy();
  delete missing.kernel.identity;
  assert.ok(codes(validatePolicy(missing)).has('kernel_identity_policy'));
});

test('keeps registration open but fail-closes managed launch integrity', () => {
  const mutations = [
    (trust) => { trust.externalRegistration = 'signed_allowlist'; },
    (trust) => { trust.managedLaunchVerification = 'registration_time_only'; },
    (trust) => { trust.bundledManagedVerification = 'unsigned_manifest'; },
    (trust) => { trust.promotedExternalManagedVerification = 'self_reported_digest'; },
    (trust) => { trust.integrityFailureState = 'degraded'; },
    (trust) => { trust.kernelExecutableDownloadsEnabled = true; },
    (trust) => { trust.rollbackMode = 'automatic_previous_version'; },
    (trust) => { trust.automaticFallbackEnabled = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.kernel.distributionTrust);

    assert.ok(codes(validatePolicy(invalid)).has('kernel_distribution_trust_policy'));
  }

  const missing = policy();
  delete missing.kernel.distributionTrust;
  assert.ok(codes(validatePolicy(missing)).has('kernel_distribution_trust_policy'));
});

test('requires the exact module runtime protocol policy', () => {
  const mutations = [
    (runtimeProtocol) => { runtimeProtocol.protocolPackage = 'hermes-runtime-contracts'; },
    (runtimeProtocol) => { runtimeProtocol.role = 'core'; },
    (runtimeProtocol) => { runtimeProtocol.owner = 'kernel'; },
    (runtimeProtocol) => { runtimeProtocol.surface = 'implementation'; },
    (runtimeProtocol) => { runtimeProtocol.serialization = 'json'; },
    (runtimeProtocol) => { runtimeProtocol.descriptorMajorVersion = 2; },
    (runtimeProtocol) => { runtimeProtocol.descriptorDigestSource = 'decoded_reserialization'; },
    (runtimeProtocol) => { runtimeProtocol.descriptorCarriesOwnDigest = true; },
    (runtimeProtocol) => { runtimeProtocol.approvalUnit = 'module'; },
    (runtimeProtocol) => { runtimeProtocol.grantRule = 'requested_or_approved'; },
    (runtimeProtocol) => { runtimeProtocol.dependencyBinding = 'module_id'; },
    (runtimeProtocol) => { runtimeProtocol.managedArtifactBinding = 'self_reported_digest'; },
    (runtimeProtocol) => { runtimeProtocol.unknownMajorState = 'degraded'; },
    (runtimeProtocol) => { runtimeProtocol.automaticFormatFallbackEnabled = true; },
    (runtimeProtocol) => { runtimeProtocol.wildcardPermissionsAllowed = true; },
    (runtimeProtocol) => { runtimeProtocol.arbitraryMapsAllowed = true; },
    (runtimeProtocol) => { runtimeProtocol.hostSandboxEnforcement = 'provided_by_grant_set'; },
    (runtimeProtocol) => { runtimeProtocol.forbiddenDescriptorData.pop(); },
    (runtimeProtocol) => { runtimeProtocol.forbiddenDependencies.pop(); },
    (runtimeProtocol) => { runtimeProtocol.unversionedCompatibilityAlias = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.runtimeProtocol);

    assert.ok(codes(validatePolicy(invalid)).has('runtime_protocol_policy'));
  }

  const missing = policy();
  delete missing.runtimeProtocol;
  assert.ok(codes(validatePolicy(missing)).has('runtime_protocol_policy'));
});

test('requires the exact Kernel Settings Registry policy', () => {
  assert.ok(
    policy().settings.forbiddenState.includes(
      'secret_references_and_credential_bindings',
    ),
  );

  const mutations = [
    (settings) => { settings.registryOwner = 'module'; },
    (settings) => { settings.registryComponent = 'module_settings'; },
    (settings) => { settings.schemaProtocolPackage = 'hermes-settings-protocol'; },
    (settings) => { settings.schemaSerialization = 'json'; },
    (settings) => { settings.sourceOfTruth = 'postgresql'; },
    (settings) => { settings.postgresRequired = true; },
    (settings) => { settings.authorities = ['operator_managed']; },
    (settings) => { settings.clientVisibilities = ['editable', 'hidden']; },
    (settings) => { settings.authorityVisibilityRule = 'authority_implies_visibility'; },
    (settings) => { settings.targetScopes.push('runtime_instance'); },
    (settings) => { settings.composition = 'domain_merges_integrations'; },
    (settings) => { settings.crossOwnerAtomicMutationEnabled = true; },
    (settings) => { settings.schemaBinding = 'module_id_only'; },
    (settings) => { settings.valueModel = 'json_document'; },
    (settings) => { settings.arbitraryJsonMapsOrAnyAllowed = true; },
    (settings) => { settings.secretValuesAllowed = true; },
    (settings) => { settings.credentialDelivery = 'inline_setting_value'; },
    (settings) => { settings.operatorManagedWriter = 'module'; },
    (settings) => { settings.kernelManagedWriter = 'any_module'; },
    (settings) => { settings.grantMutationAllowed = true; },
    (settings) => { settings.revisionModel = 'desired_only'; },
    (settings) => { settings.mutationConcurrency = 'last_write_wins'; },
    (settings) => { settings.applyModes.pop(); },
    (settings) => { settings.managedRestart = 'kill_and_start'; },
    (settings) => { settings.externalRestart = 'kernel_process_restart'; },
    (settings) => { settings.applyFailureState = 'degraded'; },
    (settings) => { settings.automaticRollbackEnabled = true; },
    (settings) => { settings.forbiddenState.pop(); },
    (settings) => { settings.compatibilityDocument = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.settings);

    assert.ok(codes(validatePolicy(invalid)).has('settings_policy'));
  }

  const missing = policy();
  delete missing.settings;
  assert.ok(codes(validatePolicy(missing)).has('settings_policy'));
});

test('requires an exact Control Store secret-exclusion policy', () => {
  const mutations = [
    (controlStore) => { controlStore.owner = 'vault'; },
    (controlStore) => { controlStore.storageBoundary = 'postgresql'; },
    (controlStore) => { controlStore.forbiddenData.pop(); },
    (controlStore) => { controlStore.forbiddenData.push('runtime_health'); },
    (controlStore) => { controlStore.secretCompatibilityAlias = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.controlStore);

    assert.ok(codes(validatePolicy(invalid)).has('control_store_policy'));
  }

  const missing = policy();
  delete missing.controlStore;
  assert.ok(codes(validatePolicy(missing)).has('control_store_policy'));
});

test('requires an exact Vault process, lease and secret-carrier policy', () => {
  const canonicalVault = policy().vault;
  assert.equal(
    canonicalVault.storageFormat,
    'sqlcipher_full_database_plus_xchacha20poly1305_records',
  );
  assert.equal(
    canonicalVault.platformKeyStorage,
    'macos_data_protection_keychain_device_only_non_sync',
  );
  assert.equal(
    canonicalVault.recoveryModel,
    'independent_wrapped_root_key_offline_restore',
  );
  assert.deepEqual(canonicalVault.unlockModes, [
    'platform_auto',
    'owner_presence',
    'manual_local',
    'recovery_offline',
  ]);
  assert.equal(canonicalVault.defaultUnlockMode, 'platform_auto');
  assert.equal(canonicalVault.defaultLeaseTtlSeconds, 600);
  assert.equal(canonicalVault.maximumLeaseTtlSeconds, 3600);
  assert.equal(canonicalVault.leaseResolveLimit, 1);
  assert.equal(
    canonicalVault.leaseRenewal,
    'new_lease_full_authorization_check',
  );
  assert.equal(canonicalVault.automaticInitializeOrRestoreEnabled, false);
  assert.ok(canonicalVault.leaseBindingFields.includes('secret_revision'));

  const mutations = [
    (vault) => { vault.role = 'core'; },
    (vault) => { vault.owner = 'kernel'; },
    (vault) => { vault.protocolPackage = 'hermes-vault-contracts'; },
    (vault) => { vault.keyProviderPackage = 'hermes-platform-key-provider'; },
    (vault) => { vault.runtimePackage = 'hermes-kernel'; },
    (vault) => { vault.storePackage = 'hermes-vault-store'; },
    (vault) => { vault.platformKeyAdapterPackages = []; },
    (vault) => { vault.runtimeComponent = 'vault'; },
    (vault) => { vault.processBoundary = 'kernel_process'; },
    (vault) => { vault.managementMode = 'external_allowed'; },
    (vault) => { vault.externalRegistrationEnabled = true; },
    (vault) => { vault.alternativeTopologyEnabled = true; },
    (vault) => { vault.storageFormat = 'plaintext_sqlite'; },
    (vault) => { vault.platformKeyStorage = 'file'; },
    (vault) => { vault.recoveryModel = 'root_key_export'; },
    (vault) => { vault.recoveryKeyEncoding = 'hex'; },
    (vault) => { vault.unlockModes.pop(); },
    (vault) => { vault.defaultUnlockMode = 'recovery_offline'; },
    (vault) => { vault.automaticInitializeOrRestoreEnabled = true; },
    (vault) => { vault.transport = 'plaintext_local_rpc'; },
    (vault) => { vault.transportScope = 'shared_host_channel'; },
    (vault) => { vault.kernelPayloadVisibility = 'plaintext'; },
    (vault) => { vault.leasePersistence = 'sqlite'; },
    (vault) => { vault.defaultLeaseTtlSeconds = 3_600; },
    (vault) => { vault.maximumLeaseTtlSeconds = 86_400; },
    (vault) => { vault.leaseResolveLimit = 2; },
    (vault) => { vault.leaseRenewal = 'extend_in_place'; },
    (vault) => { vault.credentialPayloadMaxBytes = 4_194_304; },
    (vault) => { vault.sessionCredentialBlobMaxBytes = 1_073_741_824; },
    (vault) => { vault.leaseBindingFields.pop(); },
    (vault) => { vault.runtimeGenerationChangeInvalidatesLeases = false; },
    (vault) => { vault.grantEpochChangeInvalidatesLeases = false; },
    (vault) => { vault.forbiddenSecretCarriers.pop(); },
    (vault) => { vault.forbiddenProtocolDependencies.pop(); },
    (vault) => { vault.forbiddenOwnerDependencies.pop(); },
    (vault) => { vault.compatibilityProcess = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.vault);

    assert.ok(codes(validatePolicy(invalid)).has('vault_policy'));
  }

  const missing = policy();
  delete missing.vault;
  assert.ok(codes(validatePolicy(missing)).has('vault_policy'));
});

test('requires the exact canonical durable envelope policy', () => {
  const mutations = [
    (events) => { events.protocolPackage = 'hermes-events-contracts'; },
    (events) => { events.role = 'core'; },
    (events) => { events.owner = 'kernel'; },
    (events) => { events.surface = 'implementation'; },
    (events) => { events.serialization = 'json'; },
    (events) => { events.envelopeMajorVersion = 2; },
    (events) => { events.kinds = events.kinds.filter((kind) => kind !== 'ack'); },
    (events) => { events.kinds = ['command', 'event', 'observation', 'result', 'result']; },
    (events) => { events.kinds.push('dead_letter'); },
    (events) => { events.kindMetadata = 'flat_fields'; },
    (events) => { events.payloadBinding = 'protobuf_type_url'; },
    (events) => { events.payloadVisibility = 'decoded_by_event_hub'; },
    (events) => { events.outboxPublishMode = 'relay_reencodes'; },
    (events) => { events.clientEnvelopeReuseAllowed = true; },
    (events) => { events.brokerAckIsEnvelopeAck = true; },
    (events) => { events.unknownMajorVersion = 'best_effort'; },
    (events) => { events.automaticFormatFallbackEnabled = true; },
    (events) => { events.forbiddenPayloadData.pop(); },
    (events) => { events.forbiddenPayloadData.push('public_status'); },
    (events) => { events.forbiddenDependencies.pop(); },
    (events) => { events.forbiddenDependencies.push('reqwest'); },
    (events) => { events.unversionedCompatibilityAlias = true; },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.events);

    assert.ok(codes(validatePolicy(invalid)).has('events_protocol_policy'));
  }

  const missing = policy();
  delete missing.events;
  assert.ok(codes(validatePolicy(missing)).has('events_protocol_policy'));
});

test('requires an exact integration ingress package allowlist', () => {
  const invalid = policy();
  invalid.dependencies.integrationDomainContractPackages = ['communications'];

  assert.ok(codes(validatePolicy(invalid)).has('dependency_policy'));
});

test('requires explicit compile-isolation policy', () => {
  const invalid = policy();
  invalid.compileIsolation.forbidSameOwnerRuntimeDependencies = false;

  assert.ok(codes(validatePolicy(invalid)).has('compile_isolation_policy'));
});

test('requires an explicit host-only integration registry', () => {
  const invalid = policy();
  invalid.integrations.hostOnlyOwners = [];

  assert.ok(codes(validatePolicy(invalid)).has('integration_policy'));
});

test('requires the exact Storage Control topology and fail-closed lifecycle policy', () => {
  const canonical = policy().storage;
  assert.deepEqual(canonical.fixedSchemas, [
    'hermes_data',
    'hermes_platform',
    'hermes_extensions',
  ]);
  assert.ok(canonical.bindingFields.includes('role_epoch'));
  assert.ok(canonical.bindingFields.includes('storage_bundle_digest'));

  const mutations = [
    (storage) => { storage.runtimeComponent = 'storage_manager'; },
    (storage) => { storage.managementMode = 'external_or_managed'; },
    (storage) => { storage.clusterTopology = 'one_cluster_per_module'; },
    (storage) => { storage.databaseTopology = 'one_database_per_module'; },
    (storage) => { storage.fixedSchemas = ['public']; },
    (storage) => { storage.runtimePath = 'direct_postgresql'; },
    (storage) => { storage.poolMode = 'session'; },
    (storage) => { storage.moduleSelfMigrationsEnabled = true; },
    (storage) => { storage.destructiveMigrationsEnabled = true; },
    (storage) => { storage.kernelSqlProxyEnabled = true; },
    (storage) => { storage.regexOnlyValidationAllowed = true; },
    (storage) => { storage.crossOwnerBusinessSqlEnabled = true; },
    (storage) => { storage.directSharedTechnicalDmlEnabled = true; },
    (storage) => { storage.sharedTechnicalFunctions.pop(); },
    (storage) => { storage.pgbouncerSoleBudgetBoundary = true; },
    (storage) => { storage.bindingFields.pop(); },
    (storage) => { storage.revocationSequence.pop(); },
    (storage) => { storage.forbiddenProtocolDependencies.pop(); },
  ];

  for (const mutate of mutations) {
    const invalid = policy();
    mutate(invalid.storage);
    assert.ok(codes(validatePolicy(invalid)).has('storage_policy'));
  }

  const missing = policy();
  delete missing.storage;
  assert.ok(codes(validatePolicy(missing)).has('storage_policy'));
});

test('requires explicit production and test-only workspace roots', () => {
  const invalid = policy();
  delete invalid.tests;

  assert.ok(codes(validatePolicy(invalid)).has('test_layout_policy'));
});

test('requires an explicit production source-size limit', () => {
  const invalid = policy();
  invalid.source.maxProductionSourceLines = 801;

  assert.ok(codes(validatePolicy(invalid)).has('source_policy'));
});

test('requires an explicit single-layout policy', () => {
  const invalid = policy();
  delete invalid.layout;

  assert.ok(codes(validatePolicy(invalid)).has('layout_policy'));
});

for (const owner of ['relationships', 'projects', 'obligations', 'decisions', 'knowledge', 'review']) {
  test(`rejects a production path for blocked domain ${owner}`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `modules/${owner}/src/lib.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('blocked_source_owner'));
  });
}

test('rejects singular aliases of blocked owners in production paths', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/project/src/lib.rs', content: '' },
  ]);

  assert.ok(codes(violations).has('blocked_source_owner'));
});

for (const owner of ['graph', 'timeline', 'search', 'context']) {
  test(`rejects a production path for blocked projection ${owner}`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `crates/hermes-${owner}-runtime/src/lib.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('blocked_projection'));
  });
}

test('rejects SQL ownership for a blocked domain', () => {
  const violations = validateSourceEntries(policy(), [
    {
      path: 'modules/tasks/migrations/0001.sql',
      content: 'CREATE TABLE projects (id UUID PRIMARY KEY);',
    },
  ]);

  assert.ok(codes(violations).has('blocked_sql_owner'));
});

test('does not treat SQL comments as ownership declarations', () => {
  const violations = validateSourceEntries(policy(), [
    {
      path: 'modules/tasks/migrations/0001.sql',
      content: '-- CREATE TABLE projects (id UUID);\nCREATE TABLE tasks (id UUID);',
    },
  ]);

  assert.deepEqual(violations, []);
});

test('allows source paths for enabled domains including AI', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/communications/src/lib.rs', content: '' },
    { path: 'modules/contacts/src/lib.rs', content: '' },
    { path: 'modules/organizations/src/lib.rs', content: '' },
    { path: 'modules/tasks/src/lib.rs', content: '' },
    { path: 'modules/calendar/src/lib.rs', content: '' },
    { path: 'modules/documents/src/lib.rs', content: '' },
    { path: 'modules/ai/src/lib.rs', content: '' },
  ]);

  assert.deepEqual(violations, []);
});

test('rejects an oversized production source file', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/platform/kernel/src/main.rs',
    content: Array.from({ length: 801 }, () => 'let value = 1;').join('\n'),
  }]);

  assert.ok(codes(violations).has('production_source_too_large'));
});

test('does not infer ownership from ordinary filenames below an allowed owner', () => {
  const violations = validateSourceEntries(policy(), [
    { path: 'modules/tasks/src/request_context.rs', content: '' },
    { path: 'crates/hermes-documents-runtime/src/search_adapter.rs', content: '' },
  ]);

  assert.deepEqual(violations, []);
});

for (const directory of ['tests', 'fixtures', 'snapshots']) {
  test(`rejects package-local ${directory} directories in production source`, () => {
    const violations = validateSourceEntries(policy(), [
      { path: `src/domains/tasks/runtime/${directory}/api.rs`, content: '' },
    ]);

    assert.ok(codes(violations).has('test_in_production_source'));
  });
}

for (const path of [
  'src/domains/tasks/implementation/src/task_test.rs',
  'src/domains/tasks/implementation/src/test_task.rs',
]) {
  test(`rejects production test filename ${path}`, () => {
    const violations = validateSourceEntries(policy(), [{ path, content: '' }]);

    assert.ok(codes(violations).has('test_in_production_source'));
  });
}

test('rejects snapshot files in production source', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/state.snap',
    content: '',
  }]);

  assert.ok(codes(violations).has('test_in_production_source'));
});

test('rejects inline cfg(test) modules in production Rust', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '#[cfg(test)]\nmod tests;',
  }]);

  assert.ok(codes(violations).has('test_in_production_source'));
});

test('rejects compound Rust cfg attributes that include test code', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '#[cfg(all(test, feature = "slow"))]\nmod slow_tests;',
  }]);

  assert.ok(codes(violations).has('test_in_production_source'));
});

test('does not treat cfg(test) text inside a Rust comment as an inline test', () => {
  const violations = validateSourceEntries(policy(), [{
    path: 'src/domains/tasks/implementation/src/lib.rs',
    content: '// #[cfg(test)] is forbidden by ADR-0211',
  }]);

  assert.deepEqual(violations, []);
});

test('filesystem scan reads Rust content and exposes nested test paths to policy', async (context) => {
  const root = await mkdtemp(join(tmpdir(), 'hermes-architecture-'));
  context.after(() => rm(root, { recursive: true, force: true }));
  const packageSource = join(root, 'src', 'domains', 'tasks', 'implementation', 'src');
  await mkdir(join(packageSource, 'tests'), { recursive: true });
  await writeFile(join(packageSource, 'lib.rs'), '#[cfg(test)]\nmod tests;\n');
  await writeFile(join(packageSource, 'tests', 'api.rs'), 'fn helper() {}\n');
  await symlink(join(root, 'outside-source'), join(packageSource, 'linked-source'));

  const entries = await collectSourceEntries(root, policy());
  const violations = validateSourceEntries(policy(), entries);

  assert.match(
    entries.find(({ path }) => path.endsWith('/lib.rs'))?.content ?? '',
    /cfg\(test\)/u,
  );
  assert.ok(entries.some(({ path }) => path.includes('/tests/')));
  assert.ok(entries.some(({ isSymbolicLink }) => isSymbolicLink === true));
  assert.ok(codes(violations).has('test_in_production_source'));
  assert.ok(codes(violations).has('source_symlink'));
});
