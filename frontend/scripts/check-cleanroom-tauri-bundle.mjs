#!/usr/bin/env node

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const frontendRoot = resolve(fileURLToPath(new URL('..', import.meta.url)));
const configPath = resolve(frontendRoot, 'src-tauri/tauri.conf.json');
const sourcePath = resolve(frontendRoot, 'src-tauri/src/lib.rs');
const config = JSON.parse(readFileSync(configPath, 'utf8'));
const source = readFileSync(sourcePath, 'utf8');
const failures = [];

const resources = config.bundle?.resources ?? {};
if (Object.keys(resources).some((resource) => resource.includes('google-oauth'))) {
  failures.push('Tauri bundle must not package legacy Google OAuth resources');
}
if (!Array.isArray(config.bundle?.externalBin) || !config.bundle.externalBin.includes('binaries/hermes-kernel')) {
  failures.push('Tauri bundle must declare the hermes-kernel sidecar');
}
if (/HERMES_GOOGLE_OAUTH_CLIENT_CONFIG|HERMES_LOCAL_API_SECRET/.test(source)) {
  failures.push('Tauri sidecar source must not forward legacy OAuth or local API secrets');
}

if (failures.length > 0) {
  for (const failure of failures) process.stderr.write(`cleanroom-tauri-bundle: ${failure}\n`);
  process.exitCode = 1;
}
