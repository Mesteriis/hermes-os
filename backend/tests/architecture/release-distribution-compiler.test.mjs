import assert from 'node:assert/strict';
import { generateKeyPairSync, verify } from 'node:crypto';
import {
  chmodSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import {
  compileReleaseDistribution,
  generateReleaseSigningKey,
  loadReleaseSigningKey,
  writeReleaseArtifact,
} from '../../scripts/lib/release-distribution-compiler.mjs';

function decodeVarint(bytes, offset) {
  let value = 0n;
  let shift = 0n;
  let cursor = offset;
  while (cursor < bytes.length) {
    const byte = bytes[cursor];
    cursor += 1;
    value |= BigInt(byte & 0x7f) << shift;
    if ((byte & 0x80) === 0) return [value, cursor];
    shift += 7n;
  }
  throw new Error('truncated protobuf varint');
}

function decodeFields(bytes) {
  const fields = new Map();
  let offset = 0;
  while (offset < bytes.length) {
    const [tag, afterTag] = decodeVarint(bytes, offset);
    const field = Number(tag >> 3n);
    const wireType = Number(tag & 0x07n);
    offset = afterTag;
    if (wireType === 0) {
      const [value, afterValue] = decodeVarint(bytes, offset);
      fields.set(field, [...(fields.get(field) ?? []), value]);
      offset = afterValue;
      continue;
    }
    assert.equal(wireType, 2);
    const [length, afterLength] = decodeVarint(bytes, offset);
    const end = afterLength + Number(length);
    fields.set(field, [...(fields.get(field) ?? []), bytes.subarray(afterLength, end)]);
    offset = end;
  }
  return fields;
}

function fieldString(fields, number) {
  return fields.get(number)?.[0]?.toString('utf8');
}

function canonicalTemporaryDirectory(prefix) {
  return mkdtempSync(join(realpathSync(tmpdir()), prefix));
}

test('compiles a signed P-256 distribution manifest and matching trust root', async () => {
  const root = canonicalTemporaryDirectory('hermes-release-compiler-');
  try {
    const runtime = join(root, 'runtime');
    const descriptor = join(root, 'descriptor.pb');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(descriptor, 'descriptor bytes', { mode: 0o600 });
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(privateKeyPath, keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    const artifacts = await compileReleaseDistribution({
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'hermes-desktop',
      release_version: '1.0.0',
      build_id: 'build-1',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      additional_verification_keys: [],
      artifacts: [{
        artifact_kind: 'module_runtime',
        artifact_id: 'runtime.mail',
        relative_path: 'bin/mail',
        source_path: runtime,
        required: true,
        descriptor: {
          relative_path: 'contracts/mail.pb',
          source_path: descriptor,
        },
        settings_schema: null,
      }],
    }, loadReleaseSigningKey(privateKeyPath));

    const signed = decodeFields(artifacts.signedManifest);
    const rawManifest = signed.get(2)?.[0];
    assert.equal(fieldString(signed, 1), 'release-2026');
    assert.equal(signed.get(3)?.[0].length, 64);
    assert.ok(verify(
      'sha256',
      rawManifest,
      { key: keyPair.publicKey, dsaEncoding: 'ieee-p1363' },
      signed.get(3)[0],
    ));
    const manifest = decodeFields(rawManifest);
    assert.equal(fieldString(manifest, 3), 'hermes-desktop');
    assert.equal(manifest.get(8)?.length, 1);
    const artifact = decodeFields(manifest.get(8)[0]);
    assert.equal(fieldString(artifact, 2), 'runtime.mail');
    assert.equal(artifact.get(5)[0].length, 32);
    assert.equal(artifact.get(6)[0].length, 32);

    const trustRoot = decodeFields(artifacts.trustRoot);
    assert.equal(trustRoot.get(1)[0], 1n);
    const trustKey = decodeFields(trustRoot.get(3)[0]);
    assert.equal(fieldString(trustKey, 1), 'release-2026');
    assert.equal(trustKey.get(2)[0].length, 65);
    assert.equal(trustKey.get(2)[0][0], 4);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('rejects unordered artifacts and an exposed release signing key', async () => {
  const root = canonicalTemporaryDirectory('hermes-release-compiler-invalid-');
  try {
    const runtime = join(root, 'runtime');
    const descriptor = join(root, 'descriptor.pb');
    const privateKeyPath = join(root, 'release-key.pem');
    writeFileSync(runtime, 'runtime bytes');
    writeFileSync(descriptor, 'descriptor bytes');
    const keyPair = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(privateKeyPath, keyPair.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    const input = {
      verification_key_id: 'release-2026',
      trust_root_revision: 1,
      revision: 1,
      distribution_id: 'hermes-desktop',
      release_version: '1.0.0',
      build_id: 'build-1',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      additional_verification_keys: [],
      artifacts: [
        {
          artifact_kind: 'module_runtime', artifact_id: 'runtime.z', relative_path: 'bin/z',
          source_path: runtime, required: true,
          descriptor: { relative_path: 'contracts/z.pb', source_path: descriptor }, settings_schema: null,
        },
        {
          artifact_kind: 'module_runtime', artifact_id: 'runtime.a', relative_path: 'bin/a',
          source_path: runtime, required: true,
          descriptor: { relative_path: 'contracts/a.pb', source_path: descriptor }, settings_schema: null,
        },
      ],
    };
    await assert.rejects(
      compileReleaseDistribution(input, loadReleaseSigningKey(privateKeyPath)),
      /artifact is invalid/,
    );
    chmodSync(privateKeyPath, 0o644);
    assert.throws(() => loadReleaseSigningKey(privateKeyPath), /group or other access/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('adds sorted public P-256 rotation keys to the release trust root', async () => {
  const root = canonicalTemporaryDirectory('hermes-release-rotation-');
  try {
    const runtime = join(root, 'runtime');
    const descriptor = join(root, 'descriptor.pb');
    const activeKeyPath = join(root, 'active-release-key.pem');
    const nextKeyPath = join(root, 'next-release-key.pem');
    writeFileSync(runtime, 'runtime bytes', { mode: 0o700 });
    writeFileSync(descriptor, 'descriptor bytes', { mode: 0o600 });
    const active = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    const next = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    writeFileSync(activeKeyPath, active.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    writeFileSync(nextKeyPath, next.publicKey.export({ type: 'spki', format: 'pem' }), {
      mode: 0o644,
    });
    const artifacts = await compileReleaseDistribution({
      verification_key_id: 'release-2026',
      trust_root_revision: 2,
      revision: 1,
      distribution_id: 'hermes-desktop',
      release_version: '1.0.0',
      build_id: 'build-rotation',
      target_triple: 'aarch64-apple-darwin',
      generation: 1,
      additional_verification_keys: [{
        key_id: 'release-2027',
        public_key_path: nextKeyPath,
      }],
      artifacts: [{
        artifact_kind: 'module_runtime',
        artifact_id: 'runtime.mail',
        relative_path: 'bin/mail',
        source_path: runtime,
        required: true,
        descriptor: { relative_path: 'contracts/mail.pb', source_path: descriptor },
        settings_schema: null,
      }],
    }, loadReleaseSigningKey(activeKeyPath));
    const trustRoot = decodeFields(artifacts.trustRoot);
    assert.equal(trustRoot.get(2)[0], 2n);
    assert.deepEqual(
      trustRoot.get(3).map((key) => fieldString(decodeFields(key), 1)),
      ['release-2026', 'release-2027'],
    );
    assert.ok(verify(
      'sha256',
      decodeFields(artifacts.signedManifest).get(2)[0],
      { key: active.publicKey, dsaEncoding: 'ieee-p1363' },
      decodeFields(artifacts.signedManifest).get(3)[0],
    ));
    await assert.rejects(
      compileReleaseDistribution({
        verification_key_id: 'release-2026',
        trust_root_revision: 2,
        revision: 1,
        distribution_id: 'hermes-desktop',
        release_version: '1.0.0',
        build_id: 'build-reject-private-key',
        target_triple: 'aarch64-apple-darwin',
        generation: 1,
        additional_verification_keys: [{
          key_id: 'release-2027',
          public_key_path: activeKeyPath,
        }],
        artifacts: [{
          artifact_kind: 'module_runtime',
          artifact_id: 'runtime.mail',
          relative_path: 'bin/mail',
          source_path: runtime,
          required: true,
          descriptor: { relative_path: 'contracts/mail.pb', source_path: descriptor },
          settings_schema: null,
        }],
      }, loadReleaseSigningKey(activeKeyPath)),
      /only public key material/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('never overwrites a release artifact output', () => {
  const root = canonicalTemporaryDirectory('hermes-release-output-');
  try {
    const output = join(root, 'trust-root.pb');
    writeReleaseArtifact(output, Buffer.from('first'));
    assert.throws(() => writeReleaseArtifact(output, Buffer.from('second')));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('generates an owner-private P-256 release key without overwriting an existing file', () => {
  const root = canonicalTemporaryDirectory('hermes-release-key-');
  try {
    const output = join(root, 'release-key.pem');
    generateReleaseSigningKey(output);
    assert.equal(loadReleaseSigningKey(output).asymmetricKeyType, 'ec');
    assert.equal(statSync(output).mode & 0o077, 0);
    assert.throws(() => generateReleaseSigningKey(output));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
