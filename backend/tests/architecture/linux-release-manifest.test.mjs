import assert from 'node:assert/strict';
import test from 'node:test';

import {
  validateManifest,
  validateManifestSignerTrust,
} from '../../scripts/verify-linux-release-manifest.mjs';
import { renderCompose } from '../../scripts/lib/linux-release-compose.mjs';
import { renderSystemdUnit } from '../../scripts/lib/linux-release-systemd.mjs';

const digest = 'sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';
const serviceNames = ['kernel', 'vault', 'telemetry', 'storage-control', 'postgres', 'pgbouncer', 'nats'];

function validManifest() {
  return {
    schema_version: 1,
    deployment_profile: 'linux_docker_server_v1',
    runtime_lifecycle: 'external_compose',
    docker_socket_access: false,
    service_contract: 'hermes_platform_service_v1',
    cosign: {
      certificate_identity: 'https://release.example.invalid/hermes',
      oidc_issuer: 'https://issuer.example.invalid',
    },
    services: Object.fromEntries(serviceNames.map((name) => [
      name,
      { image: `registry.example.invalid/hermes/${name}@${digest}` },
    ])),
  };
}

test('accepts a digest-pinned Linux external-compose release manifest', () => {
  assert.deepEqual(validateManifest(validManifest()), []);
  assert.deepEqual(validateManifestSignerTrust(validManifest(), {
    certificateIdentity: 'https://release.example.invalid/hermes',
    oidcIssuer: 'https://issuer.example.invalid',
  }), []);
});

test('rejects a release manifest whose declared signer differs from explicit release trust', () => {
  assert.ok(validateManifestSignerTrust(validManifest(), {
    certificateIdentity: 'https://release.example.invalid/other',
    oidcIssuer: 'https://issuer.example.invalid',
  }).some((error) => error.includes('pinned release signer')));
});

test('rejects tags, Docker socket access and incomplete platform service manifests', () => {
  const tagged = validManifest();
  tagged.services.kernel.image = 'registry.example.invalid/hermes/kernel:latest';
  assert.ok(validateManifest(tagged).some((error) => error.includes('immutable sha256')));

  const socket = validManifest();
  socket.docker_socket_access = true;
  assert.ok(validateManifest(socket).some((error) => error.includes('docker_socket_access')));

  const incomplete = validManifest();
  delete incomplete.services.nats;
  assert.ok(validateManifest(incomplete).some((error) => error.includes('exactly declare')));

  const unknownServiceContract = validManifest();
  unknownServiceContract.service_contract = 'image-default-healthcheck';
  assert.ok(validateManifest(unknownServiceContract).some((error) => error.includes('service_contract')));

  const emptySigner = validManifest();
  emptySigner.cosign.certificate_identity = '';
  assert.ok(validateManifest(emptySigner).some((error) => error.includes('cosign')));
});

test('renders a private, digest-only Compose contour without secret values or Docker socket access', () => {
  const compose = renderCompose(validManifest(), '/var/lib/hermes/release-secrets');

  for (const name of serviceNames) {
    assert.match(compose, new RegExp(`^  ${name}:$`, 'm'));
    assert.match(compose, new RegExp(`image: registry\\.example\\.invalid/hermes/${name}@${digest}`));
  }
  assert.match(compose, /^networks:\n  hermes-private:\n    internal: true$/m);
  assert.doesNotMatch(compose, /docker\.sock/);
  assert.doesNotMatch(compose, /HERMES_POSTGRES_PASSWORD|change-me|DATABASE_URL/);
  assert.match(compose, /POSTGRES_PASSWORD_FILE: \/run\/secrets\/postgres_bootstrap_password/);
  assert.match(compose, /postgres_bootstrap_password:\n    file: "\/var\/lib\/hermes\/release-secrets\/postgres_bootstrap_password"/);
  assert.match(compose, /condition: service_healthy/);
  assert.match(compose, /test: \["CMD", "\/usr\/local\/bin\/hermes-platform-healthcheck"\]/);
  assert.match(compose, /pids_limit: 256/);
});

test('renders an explicit systemd lifecycle unit for an external Compose project', () => {
  const unit = renderSystemdUnit('/opt/hermes/releases/2026-07-16/hermes.compose.yaml');

  assert.match(unit, /Type=oneshot/);
  assert.match(unit, /RemainAfterExit=yes/);
  assert.match(unit, /ExecStart=\/usr\/bin\/docker compose --project-name hermes-platform --file \/opt\/hermes\/releases\/2026-07-16\/hermes\.compose\.yaml up --detach --remove-orphans/);
  assert.match(unit, /ExecStop=\/usr\/bin\/docker compose --project-name hermes-platform --file \/opt\/hermes\/releases\/2026-07-16\/hermes\.compose\.yaml down --timeout 30/);
  assert.doesNotMatch(unit, /Restart=/);
  assert.throws(() => renderSystemdUnit('relative/hermes.compose.yaml'), /absolute/);
  assert.throws(() => renderSystemdUnit('/opt/hermes/release dir/hermes.compose.yaml'), /safe characters/);
});
