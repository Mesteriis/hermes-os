#!/usr/bin/env node

import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, renameSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

import { renderCompose } from './lib/linux-release-compose.mjs';

const digestPattern = /^[a-z0-9][a-z0-9._/-]*@sha256:[a-f0-9]{64}$/;
const requiredServiceNames = ['kernel', 'vault', 'telemetry', 'storage-control', 'postgres', 'pgbouncer', 'nats'];

function fail(message) {
  process.stderr.write(`linux-release-manifest: ${message}\n`);
  process.exitCode = 1;
}

function parseManifest(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch (error) {
    fail(`cannot parse manifest: ${error.message}`);
    return null;
  }
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

export function validateManifest(manifest) {
  if (!isObject(manifest)) return ['manifest must be an object'];
  const errors = [];
  if (manifest.schema_version !== 1) errors.push('schema_version must be 1');
  if (manifest.deployment_profile !== 'linux_docker_server_v1') {
    errors.push('deployment_profile must be linux_docker_server_v1');
  }
  if (manifest.runtime_lifecycle !== 'external_compose') {
    errors.push('runtime_lifecycle must be external_compose');
  }
  if (manifest.docker_socket_access !== false) {
    errors.push('docker_socket_access must be false');
  }
  if (manifest.service_contract !== 'hermes_platform_service_v1') {
    errors.push('service_contract must be hermes_platform_service_v1');
  }
  if (!isObject(manifest.cosign) || typeof manifest.cosign.certificate_identity !== 'string'
    || typeof manifest.cosign.oidc_issuer !== 'string') {
    errors.push('cosign certificate identity and OIDC issuer are required');
  }
  if (!isObject(manifest.services)) {
    errors.push('services must be an object');
    return errors;
  }
  const serviceNames = Object.keys(manifest.services).sort();
  if (serviceNames.join(',') !== [...requiredServiceNames].sort().join(',')) {
    errors.push('services must exactly declare the Linux platform runtime set');
  }
  for (const [name, service] of Object.entries(manifest.services)) {
    if (!isObject(service) || typeof service.image !== 'string' || !digestPattern.test(service.image)) {
      errors.push(`${name} must use an immutable sha256 image digest`);
    }
  }
  return errors;
}

export function main(argv = process.argv.slice(2)) {
  const [manifestPath, option, composePath] = argv;
  if (argv.length !== 1 && (argv.length !== 3 || option !== '--write-compose')) {
    fail('usage: verify-linux-release-manifest.mjs <manifest.json> [--write-compose <compose.yaml>]');
    return;
  }
  const manifest = parseManifest(manifestPath);
  if (!manifest) return;
  const errors = validateManifest(manifest);
  if (errors.length > 0) {
    for (const error of errors) fail(error);
    return;
  }
  try {
    for (const service of Object.values(manifest.services)) {
      execFileSync('cosign', [
        'verify',
        '--certificate-identity', manifest.cosign.certificate_identity,
        '--certificate-oidc-issuer', manifest.cosign.oidc_issuer,
        service.image,
      ], { stdio: 'inherit' });
    }
  } catch (error) {
    fail(`Cosign verification failed: ${error.message}`);
    return;
  }
  if (composePath) {
    const outputDirectory = dirname(composePath);
    const temporaryPath = `${composePath}.tmp-${process.pid}`;
    try {
      mkdirSync(outputDirectory, { recursive: true, mode: 0o700 });
      writeFileSync(temporaryPath, renderCompose(manifest, `${outputDirectory}/secrets`), { mode: 0o600 });
      renameSync(temporaryPath, composePath);
    } catch (error) {
      fail(`cannot write Compose descriptor: ${error.message}`);
    }
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
