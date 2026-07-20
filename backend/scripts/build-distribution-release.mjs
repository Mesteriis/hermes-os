#!/usr/bin/env node

import {
  assertReleaseArtifactAbsent,
  assertReleaseDistributionAbsent,
  compileReleaseDistribution,
  loadReleaseSigningKey,
  materializeReleaseDistribution,
  readReleaseCompilerInput,
  removeReleaseArtifact,
  removeReleaseDistribution,
  writeReleaseArtifact,
} from './lib/release-distribution-compiler.mjs';

function usage() {
  process.stderr.write('usage: build-distribution-release.mjs --input <release.json> --signing-key <p256-pem> --trust-root <output.pb> --signed-manifest <output.pb> --distribution-root <output-directory>\n');
}

function parseArguments(argv) {
  if (argv.length !== 10) return null;
  const values = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (!['--input', '--signing-key', '--trust-root', '--signed-manifest', '--distribution-root'].includes(option)
      || typeof value !== 'string' || value.length === 0 || values.has(option)) {
      return null;
    }
    values.set(option, value);
  }
  return values.size === 5 ? values : null;
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (!options) {
    usage();
    process.exitCode = 2;
    return;
  }
  const trustRootPath = options.get('--trust-root');
  const signedManifestPath = options.get('--signed-manifest');
  const distributionRoot = options.get('--distribution-root');
  let distributionMaterialized = false;
  try {
    if (trustRootPath === signedManifestPath
      || trustRootPath === distributionRoot
      || signedManifestPath === distributionRoot) {
      throw new Error('release output paths must be distinct');
    }
    assertReleaseArtifactAbsent(trustRootPath);
    assertReleaseArtifactAbsent(signedManifestPath);
    assertReleaseDistributionAbsent(distributionRoot);
    const input = readReleaseCompilerInput(options.get('--input'));
    const privateKey = loadReleaseSigningKey(options.get('--signing-key'));
    const artifacts = await compileReleaseDistribution(input, privateKey);
    await materializeReleaseDistribution(artifacts.artifacts, distributionRoot);
    distributionMaterialized = true;
    writeReleaseArtifact(trustRootPath, artifacts.trustRoot);
    try {
      writeReleaseArtifact(signedManifestPath, artifacts.signedManifest);
    } catch (error) {
      removeReleaseArtifact(trustRootPath);
      removeReleaseDistribution(distributionRoot);
      throw error;
    }
  } catch (error) {
    if (distributionMaterialized) {
      removeReleaseArtifact(trustRootPath);
      removeReleaseArtifact(signedManifestPath);
      removeReleaseDistribution(distributionRoot);
    }
    process.stderr.write(`distribution-release: ${error.message}\n`);
    process.exitCode = 1;
  }
}

if (import.meta.url === `file://${process.argv[1]}`) await main();
