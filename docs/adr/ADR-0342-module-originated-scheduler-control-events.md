# ADR-0342: Module-originated Scheduler control events

Статус: Принято

Дата: 2026-07-29

Состояние реализации: protocol foundation реализован. Exact Protobuf
command/result, structural validation и negative conformance существуют в
`hermes-scheduler-protocol`. Durable Scheduler inbox/outbox, JetStream adapters,
runtime bindings и managed live contour ещё не реализованы; platform gate
`scheduler_module_schedule_control_v1` остаётся закрыт.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0214](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0341](ADR-0341-scheduled-communication-delivery-workflow.md).

## Контекст

`scheduler_v1` реализует owner-control schedule mutation и managed Scheduler
lifecycle. Owner-control требует authenticated Owner session и предназначен
для явного operator/client управления. Independently managed module runtime не
имеет Owner session и не должен вызывать Gateway или private Kernel API.

Owner-local workflow при этом должен иметь возможность durably:

- создать one-shot schedule для собственного approved JobKind;
- отменить его до durable acceptance due run;
- получить exact result после Scheduler persistence transaction.

Direct Scheduler socket, cross-owner SQL, Kernel `control_rpc` и client-side
координация нарушают существующие boundaries.

## Решение

Вводятся две exact durable surfaces:

```text
scheduler.schedule.command.v1
  SchedulerScheduleControlCommandV1
    EnsureOneShotScheduleV1
    CancelOneShotScheduleV1

scheduler.schedule.result.v1
  SchedulerScheduleControlResultV1
```

Обе передаются только как typed payload `DurableEnvelopeV1`. Producer сначала
сохраняет exact envelope bytes в owner outbox. Scheduler consumer
дедуплицирует command, применяет schedule mutation и сохраняет exact result в
своей transaction до JetStream ACK.

Это расширение существующих Scheduler build units:

```text
hermes-scheduler-protocol
hermes-scheduler
hermes-scheduler-persistence
hermes-scheduler-jetstream
hermes-scheduler-runtime
```

Новый generic workflow/runtime owner не создаётся. Protocol отвечает только за
wire/validation, persistence — inbox/result outbox, implementation — mapping
на canonical `ScheduleSpecV1`, JetStream — authorized transport, runtime —
fenced workers.

## Authority

Scheduler принимает Ensure только если:

1. outer source runtime/grant fence current;
2. source capability разрешает exact schedule-control command;
3. requested JobKind принадлежит тому же module registration/capability;
4. JobKind/revision/schema exact current approved Scheduler catalog;
5. schedule scope и concurrency key opaque and bounded;
6. policy является one-shot `at`, bounded retry и `forbid` overlap;
7. command ID и schedule revision idempotent/current.

Module не может schedule чужой JobKind. Scheduler не декодирует scope и не
читает owner storage. Kernel/Event Hub проверяют topology/grants, но не
интерпретируют payload.

## Contract

`SchedulerScheduleControlCommandV1` несёт exact non-zero 16-byte
`operation_id` и одну operation.

Ensure несёт:

- exact non-zero 16-byte `schedule_id`;
- positive `schedule_revision`;
- exact JobKind owner/name/major;
- positive contract revision и non-zero SHA-256 schema digest;
- opaque bounded scope и concurrency key;
- positive UTC due time;
- bounded execution deadline;
- bounded retry attempts/backoff.

Cancel несёт:

- exact non-zero 16-byte `schedule_id`;
- positive expected schedule revision.

Result всегда коррелирует operation/schedule/revision и имеет outcome:

```text
ensured | cancelled | too_late | rejected
```

Только `rejected` содержит bounded sanitized error code. Private content,
provider/account identity, raw errors, runtime coordinates и executable paths
запрещены.

## Cancellation race

Scheduler является единственной authority:

- schedule без durably accepted due run может стать cancelled;
- existing accepted/running/terminal run возвращает `too_late`;
- duplicate Cancel возвращает тот же durable result;
- stale revision возвращает `rejected/stale_revision`;
- result не удаляет Scheduler history или owner workflow evidence.

Cancel не является provider undo и не отзывает уже принятый owner command.

## Phase gate `scheduler_module_schedule_control_v1`

Gate открывается только вместе с:

1. exact generated command/result and validation;
2. descriptor/catalog capability and Event Hub routes;
3. Scheduler inbox deduplication and result outbox transaction;
4. one-shot Ensure mapping with current catalog verification;
5. cancellation race against accepted/running runs;
6. JetStream consumer/result publisher with commit-before-ACK;
7. runtime/grant/generation/lease fences;
8. duplicate, stale, foreign JobKind, malformed, outage and revoke negatives;
9. live managed producer → Scheduler → result contour;
10. architecture, SRP, Cargo, Clippy and full test gates.

Protocol foundation alone не открывает gate.

## Отклонённые варианты

### Module вызывает owner-control

Owner session не является module authority.

### `request_rpc` напрямую в Scheduler runtime

Schedule mutation должна переживать sender/Scheduler/NATS restart и поэтому
требует transactional outbox/inbox, а не ambiguous immediate RPC.

### Generic arbitrary schedule policy

Первый module-originated contract ограничен one-shot use case. Cron,
fixed-interval и maintenance schedules остаются owner-control/default-template
paths до отдельного требования.

### Job payload содержит private workflow input

Scheduler получает только opaque scope. Private input остаётся owner-local.
