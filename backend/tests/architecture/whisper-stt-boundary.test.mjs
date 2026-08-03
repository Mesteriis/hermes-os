import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import test from 'node:test';

const PROJECT_ROOT = resolve(import.meta.dirname, '../../..');
const read = (path) => readFile(resolve(PROJECT_ROOT, path), 'utf8');

test('Whisper transcript artifact is a private bounded engine contract', async () => {
  const [manifest, proto, validation, policy] = await Promise.all([
    read('backend/src/speech-transcript-artifact/Cargo.toml'),
    read('backend/src/speech-transcript-artifact/proto/hermes/speech_transcript/v1/transcript.proto'),
    read('backend/src/speech-transcript-artifact/src/lib.rs'),
    read('backend/architecture/policy.json').then(JSON.parse),
  ]);

  assert.match(manifest, /role = "engine"/);
  assert.match(manifest, /owner = "speech_to_text"/);
  assert.match(manifest, /surface = "contract"/);
  assert.match(proto, /Private Blob document/);
  assert.match(proto, /repeated SpeechTranscriptSegmentV1 segments/);
  assert.match(proto, /bytes content_utf8/);
  for (const forbidden of [
    'provider_name',
    'model_name',
    'filesystem_path',
    'custody_proof',
    'map<',
  ]) {
    assert.ok(!proto.includes(forbidden), `forbidden artifact token ${forbidden}`);
  }
  assert.match(validation, /document\.encoded_len\(\) > encoded_limit/);
  assert.match(validation, /segment\.start_millis < previous_end/);
  assert.match(validation, /std::str::from_utf8/);
  assert.ok(
    policy.dependencies.integrationEngineContractPackages.includes(
      'hermes-speech-transcript-artifact',
    ),
  );
});

test('Whisper core and native process are separate integration units', async () => {
  const [coreManifest, core, processManifest, process, adr] = await Promise.all([
    read('backend/src/whisper-stt-core/Cargo.toml'),
    read('backend/src/whisper-stt-core/src/lib.rs'),
    read('backend/src/whisper-stt-process/Cargo.toml'),
    read('backend/src/whisper-stt-process/src/lib.rs'),
    read('docs/adr/ADR-0391-whisper-stt-provider-integration.md'),
  ]);

  for (const manifest of [coreManifest, processManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "whisper_stt"/);
    assert.doesNotMatch(manifest, /communications|call-transcription/);
  }
  assert.doesNotMatch(core, /std::process|Command::new|filesystem_path|provider_name/);
  const production = process.split('#[cfg(test)]')[0];
  assert.match(production, /Command::new\(&configuration\.executable\)/);
  assert.match(production, /\.env_clear\(\)/);
  assert.match(production, /\.stdin\(Stdio::null\(\)\)/);
  assert.match(production, /\.stdout\(Stdio::null\(\)\)/);
  assert.match(production, /\.stderr\(Stdio::null\(\)\)/);
  assert.match(production, /--output-json/);
  assert.match(production, /child\.kill\(\)/);
  assert.doesNotMatch(production, /Command::new\("(?:sh|bash|zsh)"\)|\.arg\("-c"\)/);
  assert.doesNotMatch(production, /std::env|provider_name|communications|call_transcription/);
  assert.match(adr, /`whisper_stt` является отдельной bundled integration/);
  assert.match(adr, /System executable\/model fallback/);
});

test('Whisper production gate remains closed until managed native conformance exists', async () => {
  const inventory = JSON.parse(
    await read('backend/architecture/communications-settings-reconstruction.json'),
  );
  const provider = inventory.slices.find((slice) => slice.gate === 'whisper_stt_provider_v1');
  const engine = inventory.slices.find((slice) => slice.gate === 'speech_to_text_engine_v1');
  assert.equal(provider.role, 'integration');
  assert.equal(provider.owner, 'whisper_stt');
  assert.equal(provider.state, 'planned');
  assert.equal(engine.state, 'planned');
});
