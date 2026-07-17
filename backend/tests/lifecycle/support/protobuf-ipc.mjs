import assert from 'node:assert/strict';
import { createConnection } from 'node:net';

export function varint(value) {
  let current = BigInt(value);
  const bytes = [];
  while (current >= 0x80n) {
    bytes.push(Number(current & 0x7fn) | 0x80);
    current >>= 7n;
  }
  return Buffer.from([...bytes, Number(current)]);
}

export function field(number, bytes) {
  return Buffer.concat([
    varint((number << 3) | 2),
    varint(bytes.length),
    bytes,
  ]);
}

export function text(number, value) {
  return field(number, Buffer.from(value, 'ascii'));
}

export function uint64(number, value) {
  return Buffer.concat([varint(number << 3), varint(value)]);
}

export function unframe(bytes) {
  const [length, index] = readVarint(bytes, 0);
  assert.equal(bytes.length - index, Number(length), 'response must contain one complete frame');
  return bytes.subarray(index);
}

export function decode(bytes) {
  const fields = new Map();
  let index = 0;
  while (index < bytes.length) {
    const [tag, afterTag] = readVarint(bytes, index);
    index = afterTag;
    const number = Number(tag >> 3n);
    const wire = Number(tag & 7n);
    if (wire === 0) {
      const [value, next] = readVarint(bytes, index);
      fields.set(number, Number(value));
      index = next;
    } else if (wire === 2) {
      const [length, next] = readVarint(bytes, index);
      index = next;
      fields.set(number, bytes.subarray(index, index + Number(length)));
      index += Number(length);
    } else {
      throw new Error(`unexpected protobuf wire type ${wire}`);
    }
  }
  return fields;
}

export function stringValue(fields, number) {
  return fields.get(number)?.toString('ascii');
}

export function errorCode(response) {
  return stringValue(decode(response), 15);
}

export function request(socketPath, message) {
  return new Promise((resolve, reject) => {
    const socket = createConnection(socketPath);
    const chunks = [];
    socket.once('error', reject);
    socket.on('data', (chunk) => chunks.push(chunk));
    socket.once('end', () => resolve(unframe(Buffer.concat(chunks))));
    socket.once('connect', () => socket.end(Buffer.concat([varint(message.length), message])));
  });
}

function readVarint(bytes, start) {
  let value = 0n;
  let shift = 0n;
  let index = start;
  while (index < bytes.length) {
    const next = BigInt(bytes[index++]);
    value |= (next & 0x7fn) << shift;
    if ((next & 0x80n) === 0n) return [value, index];
    shift += 7n;
  }
  throw new Error('truncated protobuf varint');
}
