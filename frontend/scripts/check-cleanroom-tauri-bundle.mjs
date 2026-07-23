#!/usr/bin/env node

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const frontendRoot = resolve(fileURLToPath(new URL('..', import.meta.url)));
const configPath = resolve(frontendRoot, 'src-tauri/tauri.conf.json');
const sourcePath = resolve(frontendRoot, 'src-tauri/src/lib.rs');
const mainCapabilityPath = resolve(frontendRoot, 'src-tauri/capabilities/default.json');
const appRootPath = resolve(frontendRoot, 'src/app/layout/AppLayoutRoot.vue');
const config = JSON.parse(readFileSync(configPath, 'utf8'));
const source = readFileSync(sourcePath, 'utf8');
const mainCapability = JSON.parse(readFileSync(mainCapabilityPath, 'utf8'));
const appRoot = readFileSync(appRootPath, 'utf8');
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
if (mainCapability.permissions.some((permission) => permission.startsWith('allow-'))) {
  failures.push('Tauri main window must not receive provider host-bridge permissions before route admission');
}
if (/CommunicationsWorkspaceView|PersonasWorkspaceView|@\/integrations\//.test(appRoot)) {
  failures.push('Tauri recovery shell must not mount disabled product routes or provider host bridges');
}
if (!source.includes('#[cfg(feature = "whatsapp-host-webview")]\nmod whatsapp_companion;')) {
  failures.push('Tauri provider companion module must be excluded from the default recovery build');
}
if (!/#\[cfg\(feature = "whatsapp-host-webview"\)\]\s+let builder = builder\.invoke_handler/.test(source)) {
  failures.push('Tauri provider host commands must be excluded from the default recovery invoke handler');
}

if (failures.length > 0) {
  for (const failure of failures) process.stderr.write(`cleanroom-tauri-bundle: ${failure}\n`);
  process.exitCode = 1;
}
