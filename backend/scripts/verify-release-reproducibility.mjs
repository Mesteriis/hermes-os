#!/usr/bin/env node

import { timingSafeEqual } from 'node:crypto';

import {
  compileUnsignedReleaseContent,
  readReleaseCompilerInput,
} from './lib/release-distribution-compiler.mjs';

function usage() {
  process.stderr.write('usage: verify-release-reproducibility.mjs --first-input <release.json> --second-input <release.json>\n');
}

function parseArguments(argv) {
  if (argv.length !== 4) return null;
  const values = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (!['--first-input', '--second-input'].includes(option)
      || typeof value !== 'string' || value.length === 0 || values.has(option)) return null;
    values.set(option, value);
  }
  return values.size === 2 ? values : null;
}

export async function main(argv = process.argv.slice(2)) {
  const options = parseArguments(argv);
  if (!options) {
    usage();
    process.exitCode = 2;
    return;
  }
  try {
    const [first, second] = await Promise.all([
      compileUnsignedReleaseContent(readReleaseCompilerInput(options.get('--first-input'))),
      compileUnsignedReleaseContent(readReleaseCompilerInput(options.get('--second-input'))),
    ]);
    if (first.rawManifest.length !== second.rawManifest.length
      || !timingSafeEqual(first.rawManifest, second.rawManifest)) {
      throw new Error('unsigned content manifests differ; do not sign this release');
    }
    process.stdout.write(`release-reproducibility: verified ${first.artifacts.length} artifacts before signing\n`);
  } catch (error) {
    process.stderr.write(`release-reproducibility: ${error.message}\n`);
    process.exitCode = 1;
  }
}

if (import.meta.url === `file://${process.argv[1]}`) await main();
