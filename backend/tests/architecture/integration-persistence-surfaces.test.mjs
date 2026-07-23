import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

test('integration persistence exports only durable owner-local storage', () => {
  for (const owner of ['mail', 'telegram', 'zulip', 'whatsapp']) {
    const source = readFileSync(join(backendRoot, 'src', `${owner}-persistence`, 'src', 'lib.rs'), 'utf8');

    assert.match(source, /DurablePersistence/);
    assert.doesNotMatch(source, /\b(?:HashMap|BTreeMap)\b/);
  }
});
