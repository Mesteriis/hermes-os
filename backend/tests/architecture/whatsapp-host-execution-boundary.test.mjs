import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const packageNames = [
  'whatsapp-api',
  'whatsapp-core',
  'whatsapp-persistence',
  'whatsapp-runtime',
];

test('WhatsApp backend integration keeps provider browser execution in the host', () => {
  for (const packageName of packageNames) {
    const manifest = readFileSync(
      join(backendRoot, 'src', packageName, 'Cargo.toml'),
      'utf8',
    );

    assert.doesNotMatch(manifest, /^tauri\s*=/m);
    assert.doesNotMatch(manifest, /^(?:wry|webkit2gtk|webview2-com)\s*=/m);
  }
});
