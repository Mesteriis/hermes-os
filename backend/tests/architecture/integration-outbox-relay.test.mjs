import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const backendRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

test('every Communications integration relay publishes exact durable envelopes', () => {
  for (const owner of ['mail', 'telegram', 'zulip', 'whatsapp']) {
    const source = readFileSync(
      join(backendRoot, 'src', `${owner}-runtime`, 'src', 'communications_outbox.rs'),
      'utf8',
    );

    assert.match(source, /publish_exact\(permit, record\.exact_bytes\(\)\)/);
    assert.match(source, /mark_communications_outbox_published\(record\.message_id\(\), published_at_unix_seconds\)/);
  }
});
