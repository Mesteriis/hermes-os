import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('module-originated Scheduler control is a planned event-only platform gate with an exact protocol foundation', async () => {
  const [adr, inventorySource, proto, validation, manifest] = await Promise.all([
    readFile(
      new URL(
        'docs/adr/ADR-0342-module-originated-scheduler-control-events.md',
        PROJECT_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'architecture/communications-settings-reconstruction.json',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/protocol/proto/hermes/scheduler/v1/job_command.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/protocol/src/validation/schedule_control.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/protocol/Cargo.toml',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
  ]);
  const inventory = JSON.parse(inventorySource);
  const gate = inventory.slices.find(
    (slice) => slice.gate === 'scheduler_module_schedule_control_v1',
  );

  assert.deepEqual(gate, {
    gate: 'scheduler_module_schedule_control_v1',
    role: 'platform',
    owner: 'scheduler',
    state: 'planned',
    dependsOn: ['scheduler_v1', 'nats_data_plane_v1'],
  });
  assert.match(adr, /protocol foundation реализован/);
  assert.match(adr, /DurableEnvelopeV1/);
  assert.match(proto, /message SchedulerScheduleControlCommandV1/);
  assert.match(proto, /message SchedulerScheduleControlResultV1/);
  assert.match(proto, /EnsureOneShotScheduleV1 ensure_one_shot/);
  assert.match(proto, /CancelOneShotScheduleV1 cancel_one_shot/);
  assert.match(validation, /MAX_RETRY_ATTEMPTS: u32 = 32/);
  assert.match(manifest, /role = "platform"/);
  assert.match(manifest, /owner = "scheduler"/);
  assert.match(manifest, /surface = "contract"/);
  const scheduleControlProto = proto.slice(
    proto.indexOf('message EnsureOneShotScheduleV1'),
  );
  assert.doesNotMatch(
    `${scheduleControlProto}\n${validation}`,
    /mail|telegram|whatsapp|zulip|conversation|provider/i,
  );
});
