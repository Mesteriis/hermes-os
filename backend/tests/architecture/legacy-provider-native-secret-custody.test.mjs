import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('legacy provider source secrets stay in the native recovery and Vault hosts', async () => {
  const [
    inventory,
    adr,
    recoveryManifest,
    recoveryLibrary,
    vaultHostManifest,
    vaultHostLibrary,
    developmentHost,
    vite,
    ensemble,
    probe,
  ] = await Promise.all([
    readFile(
      new URL('architecture/communications-settings-reconstruction.json', BACKEND_ROOT),
      'utf8',
    ).then(JSON.parse),
    readFile(
      new URL(
        'docs/adr/ADR-0321-legacy-provider-recovery-bundle-and-native-secret-custody.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/Cargo.toml', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/legacy-provider-recovery/src/lib.rs', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/owner-vault-provisioning-host/Cargo.toml', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL('frontend/native/owner-vault-provisioning-host/src/lib.rs', PROJECT_ROOT),
      'utf8',
    ),
    readFile(
      new URL(
        'frontend/native/owner-vault-provisioning-host/src/bin/development_host.rs',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(new URL('frontend/vite.config.ts', PROJECT_ROOT), 'utf8'),
    readFile(new URL('scripts/dev-ensemble.sh', BACKEND_ROOT), 'utf8'),
    readFile(
      new URL('scripts/probe-dev-legacy-provider-recovery.mjs', BACKEND_ROOT),
      'utf8',
    ),
  ]);

  const bundleGate = inventory.slices.find(
    (slice) => slice.gate === 'legacy_provider_recovery_bundle_v1',
  );
  const custodyGate = inventory.slices.find(
    (slice) => slice.gate === 'legacy_provider_native_secret_custody_v1',
  );
  assert.equal(bundleGate?.state, 'implemented');
  assert.equal(custodyGate?.state, 'planned');
  assert.match(adr, /Legacy `telegram_session_key`.*игнорируется/s);
  assert.match(adr, /Gmail legacy OAuth secret не получает native handle/);

  assert.match(recoveryManifest, /hermes-legacy-provider-recovery/);
  assert.match(recoveryLibrary, /LegacyProviderRecoverySessionsV1/);
  assert.match(recoveryLibrary, /LegacyProviderRecoverySecretPurposeV1/);
  assert.match(vaultHostManifest, /hermes-legacy-provider-recovery/);
  assert.match(vaultHostLibrary, /pub fn seal_custodied/);
  assert.match(vaultHostLibrary, /Zeroizing<Vec<u8>>/);

  assert.match(developmentHost, /RECOVERY_START_PATH/);
  assert.match(developmentHost, /RECOVERY_SEAL_SOURCE_PATH/);
  assert.match(developmentHost, /GeneratedTelegramSessionStoreKey/);
  assert.match(developmentHost, /LegacyProviderRecoverySecretPurposeV1::IcloudImapPassword/);
  assert.match(developmentHost, /LegacyProviderRecoverySecretPurposeV1::TelegramApiHash/);
  assert.doesNotMatch(developmentHost, /LegacyProviderRecoverySecretPurposeV1::TelegramSession/);
  assert.doesNotMatch(developmentHost, /println!\([^)]*(?:source|secret|email|username)/s);

  assert.match(vite, /\/__hermes\/legacy-provider-recovery\/v1/);
  assert.match(ensemble, /HERMES_LEGACY_PROVIDER_RECOVERY_BUNDLE_ROOT/);
  assert.match(ensemble, /VITE_HERMES_LEGACY_PROVIDER_RECOVERY/);
  assert.match(ensemble, /probe-dev-legacy-provider-recovery\.mjs/);
  assert.match(probe, /\/__hermes\/legacy-provider-recovery\/v1\/start/);
  assert.match(probe, /\/__hermes\/legacy-provider-recovery\/v1\/cancel/);
  assert.doesNotMatch(probe, /console\.(?:log|info|debug|error)/);
});
