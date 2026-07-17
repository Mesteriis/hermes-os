#!/usr/bin/env node

import { generateReleaseSigningKey } from './lib/release-distribution-compiler.mjs';

export function main(argv = process.argv.slice(2)) {
  if (argv.length !== 2 || argv[0] !== '--output' || argv[1].length === 0) {
    process.stderr.write('usage: generate-release-signing-key.mjs --output <absolute-p256-pem>\n');
    process.exitCode = 2;
    return;
  }
  try {
    generateReleaseSigningKey(argv[1]);
  } catch (error) {
    process.stderr.write(`release-signing-key: ${error.message}\n`);
    process.exitCode = 1;
  }
}

if (import.meta.url === `file://${process.argv[1]}`) main();
