import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('desktop recording decision keeps host integration workflow and domain separate', async () => {
  const adr = await read(
    'docs/adr/ADR-0394-desktop-call-recording-host-capture-and-consent-authority.md',
  );

  for (const unit of [
    'hermes-desktop-call-recording-api',
    'hermes-desktop-call-recording-core',
    'hermes-desktop-call-recording-persistence',
    'hermes-desktop-call-recording-runtime',
    'hermes-desktop-call-recording-assembly',
    'hermes-call-transcription-ingress',
  ]) {
    assert.match(adr, new RegExp(`\\b${unit}\\b`));
  }

  assert.match(adr, /Tauri `desktop_call_recording_host` adapter/);
  assert.match(adr, /Kernel.*не определяют[\s\S]*consent/);
  assert.match(adr, /target-bound[\s\S]*call_transcription/);
  assert.match(adr, /one-use\/expiry\/wrong-device consent/);
  assert.match(adr, /RIFF\/WAVE[\s\S]*mono[\s\S]*16 kHz[\s\S]*16-bit/);
  assert.match(adr, /pre-opened shared SSE[\s\S]*without polling/);
});

test('desktop recording decision rejects the legacy boolean path based recorder', async () => {
  const adr = await read(
    'docs/adr/ADR-0394-desktop-call-recording-host-capture-and-consent-authority.md',
  );

  for (const required of [
    'consent_attested: bool',
    'filesystem path',
    'external executable path',
    'hidden/autostart capture',
    'legacy Telemost recorder commands',
  ]) {
    assert.match(adr, new RegExp(required.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  assert.match(adr, /Audio.*private host-bridge request/);
  assert.match(adr, /запрещено в durable[\s\S]*PostgreSQL[\s\S]*SSE/);
  assert.match(adr, /Состояние реализации: planned/);
});
