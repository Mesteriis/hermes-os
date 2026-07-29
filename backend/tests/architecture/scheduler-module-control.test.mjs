import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

test('module-originated Scheduler control keeps its gate planned while protocol and persistence foundations are exact', async () => {
  const [adr, inventorySource, proto, validation, runtimeContract, admission, mapping, migration, authorityMigration, persistence, jetstream, manifest] = await Promise.all([
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
        'src/platform/runtime_protocol/proto/hermes/runtime/v1/scheduler_runtime.proto',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/implementation/src/control/admission.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/implementation/src/control/one_shot.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/persistence/migrations/0008_scheduler_schedule_control.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/persistence/migrations/0009_scheduler_schedule_control_authority.sql',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/persistence/src/store/control/apply.rs',
        BACKEND_ROOT,
      ),
      'utf8',
    ),
    readFile(
      new URL(
        'src/platform/scheduler/jetstream/src/transport/schedule_control.rs',
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
  assert.match(adr, /protocol, pure admission mapping и persistence foundation/);
  assert.match(adr, /DurableEnvelopeV1/);
  assert.match(proto, /message SchedulerScheduleControlCommandV1/);
  assert.match(proto, /message SchedulerScheduleControlResultV1/);
  assert.match(proto, /EnsureOneShotScheduleV1 ensure_one_shot/);
  assert.match(proto, /CancelOneShotScheduleV1 cancel_one_shot/);
  assert.match(proto, /JobKindV1 job_kind = 3/);
  assert.match(validation, /MAX_ATTEMPTS: u32 = 32/);
  assert.match(runtimeContract, /SchedulerRuntimeScheduleControlBindingV1/);
  assert.match(runtimeContract, /SchedulerRuntimeScheduleControlGrantV1/);
  assert.match(admission, /source_runtime_generation/);
  assert.match(admission, /source_grant_epoch/);
  assert.match(mapping, /ScheduleTriggerV1::At/);
  assert.match(mapping, /OverlapPolicyV1::Forbid/);
  assert.match(mapping, /MisfirePolicyV1::FireOnce/);
  assert.match(migration, /scheduler_schedule_control_inbox/);
  assert.match(migration, /scheduler_schedule_control_results/);
  assert.match(authorityMigration, /scheduler_schedule_control_authorities/);
  assert.match(persistence, /command_envelope_sha256/);
  assert.match(persistence, /exact_envelope_bytes/);
  assert.match(persistence, /SchedulerScheduleControlDecisionV1::TooLate/);
  assert.match(persistence, /ForeignAuthority/);
  assert.match(jetstream, /SchedulerJetStreamScheduleControlPortV1/);
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
