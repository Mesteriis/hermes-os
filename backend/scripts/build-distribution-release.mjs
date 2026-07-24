#!/usr/bin/env node

import {
  assertReleaseArtifactAbsent,
  assertReleaseDistributionAbsent,
  compileReleaseDistribution,
  composeReleaseCompilerInput,
  loadReleaseSigningKey,
  materializeReleaseDistribution,
  readReleaseArtifactFragment,
  readReleaseCompilerInput,
  removeReleaseArtifact,
  removeReleaseDistribution,
  writeReleaseArtifact,
} from './lib/release-distribution-compiler.mjs';

function usage() {
  process.stderr.write('usage: build-distribution-release.mjs --input <release.json> [--artifact-fragment <module-artifacts.json> ...] --signing-key <p256-pem> --trust-root <output.pb> --signed-manifest <output.pb> --distribution-root <output-directory>\n');
}

function parseArguments(argv) {
  if (argv.length < 10 || argv.length % 2 !== 0) return null;
  const values = new Map();
  const artifactFragments = [];
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (option === '--artifact-fragment') {
      if (typeof value !== 'string' || value.length === 0) return null;
      artifactFragments.push(value);
      continue;
    }
    if (!['--input', '--signing-key', '--trust-root', '--signed-manifest', '--distribution-root'].includes(option)
      || typeof value !== 'string' || value.length === 0 || values.has(option)) {
      return null;
    }
    values.set(option, value);
  }
  return values.size === 5 ? { values, artifactFragments } : null;
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (!options) {
    usage();
    process.exitCode = 2;
    return;
  }
  const trustRootPath = options.values.get('--trust-root');
  const signedManifestPath = options.values.get('--signed-manifest');
  const distributionRoot = options.values.get('--distribution-root');
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
    const baseInput = readReleaseCompilerInput(options.values.get('--input'));
    const input = options.artifactFragments.length === 0
      ? baseInput
      : composeReleaseCompilerInput(
        baseInput,
        options.artifactFragments.map(readReleaseArtifactFragment),
      );
    const privateKey = loadReleaseSigningKey(options.values.get('--signing-key'));
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
