import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);
const ADR_PATH = new URL(
  'docs/adr/ADR-0288-managed-successor-quiesce-and-storage-fence-order.md',
  PROJECT_ROOT,
);
const SUCCESSOR_PATH = new URL('src/kernel/src/platform/storage/successor.rs', BACKEND_ROOT);
const SUPERVISOR_PATH = new URL(
  'src/kernel/src/runtime/lifecycle/supervisor.rs',
  BACKEND_ROOT,
);
const SUPERVISION_TEST_PATH = new URL(
  'tests/support/kernel-recovery/src/tests/managed_runtime_supervision/mod.rs',
  BACKEND_ROOT,
);
const TELEGRAM_LIVE_PATH = new URL(
  'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/telegram_managed_flow.rs',
  BACKEND_ROOT,
);

test('managed successor quiesces the predecessor before physical fencing and reservation', async () => {
  const [adr, successor, supervisor, supervisionTest, telegramLive] = await Promise.all([
    readFile(ADR_PATH, 'utf8'),
    readFile(SUCCESSOR_PATH, 'utf8'),
    readFile(SUPERVISOR_PATH, 'utf8'),
    readFile(SUPERVISION_TEST_PATH, 'utf8'),
    readFile(TELEGRAM_LIVE_PATH, 'utf8'),
  ]);

  assert.match(adr, /Состояние реализации: Реализовано/);
  assert.match(
    successor,
    /request_stop_if_active[\s\S]*fence_reserved_binding[\s\S]*stop_if_active/,
  );
  assert.match(
    successor,
    /revoke_predecessor\(supervisor,[\s\S]*managed_launch::reserve\(supervisor,/,
  );
  assert.match(supervisor, /request_stop_if_active[\s\S]*stop_requested\.store\(true/);
  assert.match(
    supervisionTest,
    /managed_runtime_supervisor_quiesce_is_idempotent_until_join/,
  );
  assert.match(
    telegramLive,
    /managed_telegram_core_operational_projection_is_restart_safe/,
  );
  assert.doesNotMatch(successor, /telegram|whatsapp|zulip|mail/i);
});
