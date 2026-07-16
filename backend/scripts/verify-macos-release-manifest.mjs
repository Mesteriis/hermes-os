#!/usr/bin/env node

import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { lstatSync, readFileSync, statSync } from 'node:fs';

const digestPattern = /^[a-f0-9]{64}$/;
const teamIdPattern = /^[A-Z0-9]{10}$/;

function fail(message) {
  process.stderr.write(`macos-release-manifest: ${message}\n`);
  process.exitCode = 1;
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function isAbsolutePath(value) {
  return typeof value === 'string' && value.startsWith('/');
}

export function validateManifest(manifest) {
  if (!isObject(manifest)) return ['manifest must be an object'];
  const errors = [];
  if (manifest.schema_version !== 1) errors.push('schema_version must be 1');
  if (manifest.deployment_profile !== 'macos_tauri_embedded_v1') {
    errors.push('deployment_profile must be macos_tauri_embedded_v1');
  }
  if (manifest.runtime_lifecycle !== 'managed_child') {
    errors.push('runtime_lifecycle must be managed_child');
  }
  if (!isObject(manifest.signing) || !teamIdPattern.test(manifest.signing.team_id ?? '')) {
    errors.push('signing.team_id must be a 10-character Apple Team ID');
  }
  if (!isObject(manifest.artifacts)) {
    errors.push('artifacts must be an object');
    return errors;
  }
  const { tauri_bundle: bundle, kernel_sidecar: sidecar } = manifest.artifacts;
  if (!isObject(bundle) || !isAbsolutePath(bundle.path)) {
    errors.push('artifacts.tauri_bundle.path must be absolute');
  }
  if (!isObject(sidecar) || !isAbsolutePath(sidecar.path) || !digestPattern.test(sidecar.sha256 ?? '')) {
    errors.push('artifacts.kernel_sidecar must have an absolute path and lowercase sha256 digest');
  }
  return errors;
}

function parseManifest(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch (error) {
    fail(`cannot parse manifest: ${error.message}`);
    return null;
  }
}

function rejectSymlink(path, label) {
  if (lstatSync(path).isSymbolicLink()) throw new Error(`${label} must not be a symlink`);
}

function codesignDetails(path) {
  const result = spawnSync('/usr/bin/codesign', ['-dvv', path], { encoding: 'utf8' });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(result.stderr || 'codesign could not read artifact identity');
  }
  // `codesign -dvv` reports signing metadata on stderr even on success.
  return result.stderr;
}

function verifyArtifact(path, expectedTeamId) {
  execFileSync('/usr/bin/codesign', ['--verify', '--strict', '--verbose=4', path], { stdio: 'inherit' });
  const details = codesignDetails(path);
  if (!details.includes(`TeamIdentifier=${expectedTeamId}`)) {
    throw new Error('signed artifact team identity does not match the release manifest');
  }
}

export function main(argv = process.argv.slice(2)) {
  if (argv.length !== 1) {
    fail('usage: verify-macos-release-manifest.mjs <manifest.json>');
    return;
  }
  if (process.platform !== 'darwin') {
    fail('macOS release verification must run on macOS');
    return;
  }
  const manifest = parseManifest(argv[0]);
  if (!manifest) return;
  const errors = validateManifest(manifest);
  if (errors.length > 0) {
    for (const error of errors) fail(error);
    return;
  }
  const { tauri_bundle: bundle, kernel_sidecar: sidecar } = manifest.artifacts;
  try {
    rejectSymlink(bundle.path, 'Tauri bundle');
    if (!statSync(bundle.path).isDirectory()) throw new Error('Tauri bundle must be a directory');
    rejectSymlink(sidecar.path, 'Kernel sidecar');
    if (!statSync(sidecar.path).isFile()) throw new Error('Kernel sidecar must be a regular file');
    verifyArtifact(bundle.path, manifest.signing.team_id);
    execFileSync('/usr/sbin/spctl', ['--assess', '--type', 'execute', '--verbose=4', bundle.path], { stdio: 'inherit' });
    verifyArtifact(sidecar.path, manifest.signing.team_id);
    const actualDigest = createHash('sha256').update(readFileSync(sidecar.path)).digest('hex');
    if (actualDigest !== sidecar.sha256) throw new Error('Kernel sidecar digest does not match release manifest');
  } catch (error) {
    fail(`artifact verification failed: ${error.message}`);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
