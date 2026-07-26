import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);

async function source(path) {
  return readFile(new URL(path, BACKEND_ROOT), 'utf8');
}

test('Telegram Calls contract, core and persistence are separate integration build units', async () => {
  const [apiManifest, coreManifest, persistenceManifest, runtimeManifest] = await Promise.all([
    source('src/telegram-calls-api/Cargo.toml'),
    source('src/telegram-calls-core/Cargo.toml'),
    source('src/telegram-calls-persistence/Cargo.toml'),
    source('src/telegram-runtime/Cargo.toml'),
  ]);

  for (const manifest of [apiManifest, coreManifest, persistenceManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "telegram"/);
    assert.doesNotMatch(manifest, /communications-domain|kernel|gateway/);
  }
  assert.doesNotMatch(apiManifest, /telegram-calls-core|sqlx|telegram-runtime/);
  assert.doesNotMatch(coreManifest, /sqlx|prost|telegram-runtime|telegram-tdlib/);
  assert.match(persistenceManifest, /hermes-telegram-calls-core/);
  assert.doesNotMatch(persistenceManifest, /hermes-telegram-calls-api|telegram-tdlib/);
  assert.match(runtimeManifest, /hermes-telegram-calls-api/);
  assert.match(runtimeManifest, /hermes-telegram-calls-core/);
  assert.match(runtimeManifest, /hermes-telegram-calls-persistence/);
});

test('Telegram Call history admits query and replay without opening signaling commands', async () => {
  const [admission, runtimePort, assembly, fixture] = await Promise.all([
    source('src/telegram-runtime/src/admission.rs'),
    source('src/telegram-runtime/src/calls_client_port.rs'),
    source('src/telegram-assembly/src/lib.rs'),
    source('tests/fixtures/telegram-tdjson/tdjson.c'),
  ]);

  assert.match(admission, /telegram_calls_client_capability_v1\(TelegramCallsContractV1::Query\)/);
  assert.match(
    admission,
    /telegram_calls_client_capability_v1\(TelegramCallsContractV1::Realtime\)/,
  );
  assert.doesNotMatch(
    admission,
    /telegram_calls_client_capability_v1\(TelegramCallsContractV1::Command\)/,
  );
  assert.match(runtimePort, /Telegram Calls command route is not admitted/);
  assert.doesNotMatch(runtimePort, /hermes_communications|hermes_telegram_tdlib/);
  assert.match(assembly, /telegram_storage_bundle_with_calls_v3/);
  assert.match(assembly, /telegram_calls_storage_migration_v1/);
  assert.match(fixture, /updateCall/);
  assert.match(fixture, /callStateDiscarded/);
});

test('Telegram Calls contracts are typed and do not expose media secrets', async () => {
  const [contract, proto, schema] = await Promise.all([
    source('src/telegram-calls-api/src/contract.rs'),
    source('src/telegram-calls-api/proto/hermes/telegram/calls/v1/calls.proto'),
    source('src/telegram-calls-persistence/src/schema.rs'),
  ]);

  for (const identity of [
    'telegram.calls.query.v1',
    'telegram.calls.command.v1',
    'telegram.calls.realtime.v1',
  ]) {
    assert.match(contract, new RegExp(identity.replaceAll('.', '\\.')));
  }
  assert.match(proto, /service TelegramCallsQueryService/);
  assert.match(proto, /service TelegramCallsCommandService/);
  assert.match(proto, /service TelegramCallsRealtimeService/);
  assert.doesNotMatch(proto, /\bgoogle\.protobuf\.Any\b|\bmap\s*</);
  for (const privateField of [
    'encryption_key',
    'custom_parameters',
    'raw_json',
    'audio_bytes',
    'debug_log',
  ]) {
    assert.doesNotMatch(proto, new RegExp(privateField));
    assert.doesNotMatch(schema, new RegExp(privateField));
  }
  assert.doesNotMatch(schema, /communications_/);
});

test('Telegram Calls history keeps volatile TDLib identity scoped to runtime generation', async () => {
  const [core, schema] = await Promise.all([
    source('src/telegram-calls-core/src/lib.rs'),
    source('src/telegram-calls-persistence/src/schema.rs'),
  ]);

  assert.match(core, /runtime_generation/);
  assert.match(core, /tdlib_call_id/);
  assert.match(core, /provider_call_unique_id/);
  assert.match(schema, /UNIQUE \(account_id, runtime_generation, tdlib_call_id\)/);
  assert.match(schema, /UNIQUE \(account_id, provider_call_unique_id\)/);
  assert.match(schema, /telegram_call_state_history/);
  assert.match(schema, /telegram_call_realtime_frames/);
});

test('Telegram call media contract and tgcalls adapter remain separate integration units', async () => {
  const [contractManifest, adapterManifest, contract, adapter] = await Promise.all([
    source('src/telegram-call-media-contract/Cargo.toml'),
    source('src/telegram-call-media-tgcalls/Cargo.toml'),
    source('src/telegram-call-media-contract/src/lib.rs'),
    source('src/telegram-call-media-tgcalls/src/lib.rs'),
  ]);

  for (const manifest of [contractManifest, adapterManifest]) {
    assert.match(manifest, /role = "integration"/);
    assert.match(manifest, /owner = "telegram"/);
    assert.doesNotMatch(manifest, /communications-domain|kernel|gateway|sqlx|prost/);
  }
  assert.doesNotMatch(contractManifest, /libloading|telegram-runtime|telegram-tdlib/);
  assert.match(adapterManifest, /hermes-telegram-call-media-contract/);
  assert.match(adapterManifest, /libloading/);
  assert.doesNotMatch(
    adapterManifest,
    /telegram-runtime|telegram-tdlib|telegram-persistence|telegram-assembly/,
  );
  assert.match(contract, /TelegramCallReadyPlanV1/);
  assert.match(contract, /TelegramCallSecretBytesV1/);
  assert.match(contract, /TelegramCallMediaEventV1/);
  assert.match(adapter, /TgCallsMediaAdapter/);
  assert.match(adapter, /load_exact/);
  assert.doesNotMatch(adapter, /Library::new\(["'][^"']+["']\)/);
});

test('Telegram tgcalls native build is pinned, system-audio backed and secret-negative', async () => {
  const [bridge, bridgeBuild, patch, buildScript] = await Promise.all([
    source('src/telegram-call-media-tgcalls/native/bridge.cpp'),
    source('src/telegram-call-media-tgcalls/native/BUILD.bazel'),
    source(
      'src/telegram-call-media-tgcalls/native/patches/telegram-ios-macos-audio-device.patch',
    ),
    source('scripts/build-telegram-tgcalls-bridge-macos.sh'),
  ]);

  assert.match(bridgeBuild, /\/\/submodules\/TgVoipWebrtc:tgcalls_core/);
  assert.match(bridgeBuild, /\/\/third-party\/webrtc:hermes_macos_audio_device/);
  assert.doesNotMatch(bridgeBuild, /FakeAudioDeviceModule|SineRecorder|NoOpRenderer/);
  assert.match(bridge, /createAudioDeviceModule = \{\}/);
  assert.match(bridge, /SetLoggingFunction/);
  assert.match(bridge, /std::fill\(typed->key->begin\(\), typed->key->end\(\), 0\)/);
  assert.doesNotMatch(bridge, /fprintf|std::cout|std::cerr|printf\(/);
  assert.match(patch, /audio_device_impl\.cc/);
  assert.match(patch, /audio_device_mac\.cc/);
  assert.match(buildScript, /6ad963e5b62d354da79040f388ae2b9132fb17b8/);
  assert.match(buildScript, /e3069322a3d1e16ecb11a5e302242e59ddd7f09e/);
  assert.match(buildScript, /3817e906cb6c22ec9cc62023b073e1a668d9cb33/);
  assert.match(buildScript, /45e9388abf21d1107e146ea366ad080eb93cb6a5f3a4a3b048f78de0bc3faffa/);
  assert.match(buildScript, /da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768/);
  assert.match(buildScript, /readonly XCODE_VERSION="26\.2"/);
  assert.doesNotMatch(buildScript, /submodule update --remote|--branch\s/);
});
