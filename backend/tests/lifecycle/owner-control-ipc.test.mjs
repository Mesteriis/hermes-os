import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import {
  kernelCommand,
  openOwnerSession,
  openRuntimeSession,
  ownerApprove,
  ownerBegin,
  ownerBindExternal,
  ownerComplete,
  ownerStatus,
  ownerTelemetryDiagnostics,
  ownerTransition,
  ownerUpdateSettings,
  registerModule,
  runtimeKeyPair,
  runtimeSubmitSchema,
  startKernel,
  stopKernel,
} from './support/production-kernel-ipc.mjs';
import {
  decode,
  errorCode,
  field,
  request,
  stringValue,
  text,
  uint64,
} from './support/protobuf-ipc.mjs';

function schemaArtifact() {
  const flags = Buffer.from([0x18, 0x06, 0x20, 0x01, 0x28, 0x01, 0x30, 0x01, 0x38, 0x01]);
  const definition = Buffer.concat([
    text(1, 'poll_interval'), text(2, 'capability.read'), flags, text(10, 'Poll interval'),
  ]);
  return Buffer.concat([uint64(1, 1), uint64(2, 1), field(3, definition)]);
}

function moduleDescriptor(schema) {
  const capability = Buffer.concat([
    text(1, 'capability.read'), uint64(2, 1), uint64(3, 1), text(9, 'poll_interval'),
  ]);
  const schemaRef = Buffer.concat([
    uint64(1, 1), uint64(2, 1), uint64(3, schema.length),
    field(4, createHash('sha256').update(schema).digest()),
  ]);
  return Buffer.concat([
    uint64(1, 1), uint64(2, 1), text(3, 'module-ipc'), text(4, 'owner-ipc'),
    uint64(5, 2), text(6, '1'), text(7, 'build-1'),
    field(9, capability), field(10, schemaRef),
  ]);
}

function settingsSnapshot(registrationId) {
  const entry = Buffer.concat([
    text(1, 'poll_interval'), field(2, Buffer.from([0x30, 0xe8, 0x07])),
  ]);
  return Buffer.concat([text(1, registrationId), uint64(2, 1), field(3, entry)]);
}

function bindBundledManagedRelease(registrationId, artifactId, sessionId) {
  return field(8, Buffer.concat([
    text(1, registrationId), text(2, artifactId), text(3, sessionId),
  ]));
}

function startBundledManagedRuntime(registrationId, sessionId) {
  return field(9, Buffer.concat([text(1, registrationId), text(2, sessionId)]));
}

function bindPlatformVaultRelease(sessionId) {
  return field(10, text(1, sessionId));
}

function startPlatformVaultRuntime(sessionId) {
  return field(11, text(1, sessionId));
}

async function proveOwnerSessionEnforcement(socketPath, registrationId) {
  assert.equal(errorCode(await request(
    socketPath, ownerApprove(registrationId, 'missing'),
  )), 'operation_denied');
  const begun = decode(await request(socketPath, ownerBegin())).get(4);
  const challengeId = stringValue(decode(begun), 1);
  assert.match(challengeId, /^[0-9a-f]{64}$/);
  assert.equal(errorCode(await request(
    socketPath, ownerComplete(challengeId, Buffer.alloc(64)),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, ownerComplete(challengeId, Buffer.alloc(64)),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, bindPlatformVaultRelease('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, startPlatformVaultRuntime('missing'),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPath, ownerTelemetryDiagnostics('missing'),
  )), 'operation_denied');
}

async function exerciseOwnerMutations(context, dataDir, descriptor, schema) {
  const { socketPaths, registrationId } = context;
  const ownerSessionId = await openOwnerSession(socketPaths.owner, dataDir);
  await request(socketPaths.owner, ownerApprove(registrationId, ownerSessionId));
  assert.equal(errorCode(await request(
    socketPaths.owner,
    bindBundledManagedRelease(registrationId, 'runtime.mail', ownerSessionId),
  )), 'operation_denied');
  assert.equal(errorCode(await request(
    socketPaths.owner,
    startBundledManagedRuntime(registrationId, ownerSessionId),
  )), 'operation_denied');
  const { pair, publicKey } = runtimeKeyPair();
  const bindingSession = await openOwnerSession(socketPaths.owner, dataDir);
  const bound = decode(await request(
    socketPaths.owner,
    ownerBindExternal(registrationId, publicKey, bindingSession),
  )).get(7);
  assert.equal(decode(bound).get(2), 3);
  assert.equal(errorCode(await request(
    socketPaths.owner,
    ownerBindExternal(registrationId, Buffer.alloc(65, 4), 'missing'),
  )), 'operation_denied');
  await admitSchema(context, pair, descriptor, schema);
  const status = await request(socketPaths.owner, ownerStatus(registrationId));
  assert.ok(status.includes(Buffer.from('approved', 'ascii')));
  return ownerSessionId;
}

async function admitSchema(context, pair, descriptor, schema) {
  const { socketPaths, registrationId } = context;
  const runtimeSession = await openRuntimeSession({
    socketPath: socketPaths.runtime,
    registrationId,
    runtimeId: 'runtime-owner-ipc',
    generation: 1,
    distributionSha256: createHash('sha256').update('owner IPC runtime').digest(),
    pair,
  });
  const admitted = decode(await request(
    socketPaths.runtime,
    runtimeSubmitSchema(runtimeSession, descriptor, schema),
  )).get(4);
  assert.equal(stringValue(decode(admitted), 1), registrationId);
}

async function exerciseSettingsMutation(context, dataDir, staleSessionId) {
  const { socketPaths, registrationId } = context;
  const settingsSession = await openOwnerSession(socketPaths.owner, dataDir);
  const settings = decode(await request(
    socketPaths.owner,
    ownerUpdateSettings(
      registrationId, 0, settingsSnapshot(registrationId), settingsSession,
    ),
  )).get(6);
  assert.equal(stringValue(decode(settings), 3), 'pending_validation');
  assert.equal(errorCode(await request(
    socketPaths.owner,
    ownerTransition(registrationId, 'suspended', staleSessionId),
  )), 'operation_denied');
}

test('owner control IPC requires a current owner-device proof for mutations', async () => {
  const root = await mkdtemp(join(tmpdir(), 'hermes-kernel-owner-control-'));
  const dataDir = join(root, 'data');
  let server;
  let ownerSocket;
  try {
    const schema = schemaArtifact();
    const descriptor = moduleDescriptor(schema);
    const enrolled = kernelCommand(
      dataDir, 'initial-owner-enroll', '--owner-id', 'owner-ipc',
      '--device-id', 'device-ipc',
    );
    assert.equal(enrolled.status, 0, enrolled.stderr);
    const started = await startKernel(dataDir);
    server = started.server;
    ownerSocket = started.socketPaths.owner;
    const registrationId = await registerModule(
      started.socketPaths.registration, descriptor,
    );
    assert.equal((await stat(ownerSocket)).mode & 0o777, 0o600);
    await proveOwnerSessionEnforcement(ownerSocket, registrationId);
    const staleSession = await exerciseOwnerMutations(
      { ...started, registrationId }, dataDir, descriptor, schema,
    );
    await stopKernel(server);
    await assert.rejects(stat(ownerSocket));
    const restarted = await startKernel(dataDir);
    server = restarted.server;
    ownerSocket = restarted.socketPaths.owner;
    await exerciseSettingsMutation(
      { ...restarted, registrationId }, dataDir, staleSession,
    );
  } finally {
    if (server) await stopKernel(server);
    if (ownerSocket) await assert.rejects(stat(ownerSocket));
    await rm(root, { recursive: true, force: true });
  }
});
