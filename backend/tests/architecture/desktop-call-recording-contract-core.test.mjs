import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('desktop recording contract core and target ingress are isolated build units', async () => {
  const [policySource, api, core, ingress] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/desktop-call-recording-api/Cargo.toml'),
    read('backend/src/desktop-call-recording-core/Cargo.toml'),
    read('backend/src/call-transcription-ingress/Cargo.toml'),
  ]);
  const packages = new Map(
    JSON.parse(policySource).implementation.productionPackages.map((item) => [item.name, item]),
  );

  assert.deepEqual(packages.get('hermes-desktop-call-recording-api'), {
    name: 'hermes-desktop-call-recording-api',
    role: 'integration',
    owner: 'desktop_call_recording',
    surface: 'contract',
  });
  assert.deepEqual(packages.get('hermes-desktop-call-recording-core'), {
    name: 'hermes-desktop-call-recording-core',
    role: 'integration',
    owner: 'desktop_call_recording',
    surface: 'implementation',
  });
  assert.deepEqual(packages.get('hermes-call-transcription-ingress'), {
    name: 'hermes-call-transcription-ingress',
    role: 'workflow',
    owner: 'call_transcription',
    surface: 'contract',
  });
  assert.doesNotMatch(`${api}\n${core}`, /hermes-communications|call-transcription/);
  assert.doesNotMatch(ingress, /desktop-call-recording|communications/);
});

test('public recording surface is metadata-only while private host completion is bounded audio', async () => {
  const [proto, core] = await Promise.all([
    read('backend/src/desktop-call-recording-api/proto/hermes/desktop_call_recording/v1/recording.proto'),
    read('backend/src/desktop-call-recording-core/src/lib.rs'),
  ]);
  const publicSurface = proto.slice(0, proto.indexOf('message DesktopRecordingHostHandshakeV1'));

  assert.doesNotMatch(publicSurface, /audio|blob|custody|path|device|consent_attested/);
  assert.match(proto, /DesktopCaptureCompletedV1[\s\S]*bytes canonical_wav_bytes/);
  assert.match(core, /WAV_BYTES_PER_SECOND_V1: u64 = 32_000/);
  assert.match(core, /&bytes\[0\.\.4\] != b"RIFF"/);
  assert.match(core, /RecordingStateV1::Ready[\s\S]*InvalidTransition/);
  assert.doesNotMatch(core.split('#[cfg(test)]')[0], /std::process|Command::new|filesystem|telemost/i);
});

test('recording ready event is target-owned and keeps audio outside durable envelopes', async () => {
  const proto = await read(
    'backend/src/call-transcription-ingress/proto/hermes/call_transcription/ingress/v1/recording.proto',
  );
  assert.match(proto, /message RecordingReadyV1/);
  for (const required of [
    'consent_receipt_id',
    'target_blob_reference_id',
    'custody_transfer_source_proof',
    'audio_sha256',
  ]) {
    assert.match(proto, new RegExp(`\\b${required}\\b`));
  }
  assert.doesNotMatch(proto, /audio_bytes|filesystem_path|provider_id|device_id|participant/);
});

test('recording persistence owns lifecycle leases and exact outbox without private capture bytes', async () => {
  const [policySource, manifest, migration, repository] = await Promise.all([
    read('backend/architecture/policy.json'),
    read('backend/src/desktop-call-recording-persistence/Cargo.toml'),
    read('backend/src/desktop-call-recording-persistence/migrations/0001_desktop_call_recording.sql'),
    read('backend/src/desktop-call-recording-persistence/src/repository.rs'),
  ]);
  const policy = JSON.parse(policySource);
  assert.deepEqual(
    policy.implementation.productionPackages.find(
      ({ name }) => name === 'hermes-desktop-call-recording-persistence',
    ),
    {
      name: 'hermes-desktop-call-recording-persistence',
      role: 'integration',
      owner: 'desktop_call_recording',
      surface: 'persistence',
    },
  );
  assert.match(manifest, /hermes-desktop-call-recording-core/);
  assert.doesNotMatch(manifest, /communications|call-transcription/);
  for (const required of [
    'hermes_data.desktop_call_recording_runs',
    'hermes_data.desktop_call_recording_host_commands',
    'hermes_data.desktop_call_recording_outbox',
    'hermes_data.desktop_call_recording_realtime',
  ]) {
    assert.match(migration, new RegExp(required.replaceAll('.', '\\.')));
  }
  for (const source of [migration, repository]) {
    assert.doesNotMatch(
      source,
      /audio_bytes|canonical_wav|filesystem_path|audio_input_label|consent_body|custody_transfer_source_proof/,
    );
  }
  assert.match(repository, /FOR UPDATE SKIP LOCKED/);
  assert.match(repository, /exact_envelope_bytes/);
  assert.match(repository, /accept_or_replay/);
  assert.match(repository, /recording_revision=\$1[\s\S]*run_state=4/);
});
