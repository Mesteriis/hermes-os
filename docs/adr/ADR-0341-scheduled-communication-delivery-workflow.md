# ADR-0341: Scheduled communication delivery workflow

Статус: Принято

Дата: 2026-07-29

Состояние реализации: частично реализовано. Gate
`communication_delayed_delivery_v1` остаётся `planned`. `scheduler_v1`
реализован с live restart/revoke и hot-reconciliation evidence, а
module-originated schedule-control transport принят Kernel и Scheduler.
Реализованы отдельные `api`, `core` и persistence units delayed-delivery с
generated Schedule/Cancel/Status/realtime contract, hard request/body/time
bounds, cancellation-race policy и owner-local Storage bundle. Persistence
сохраняет только bounded Blob custody receipt, Scheduler inbox/outbox correlation
и execution fences; plaintext body в workflow SQL запрещён. Managed runtime
теперь материализует private body в encrypted Blob custody до создания
operation. Создание operation
и Ensure outbox атомарно, Cancel revision-fenced и атомарен со своим outbox,
Scheduler result дедуплицируется по inbox ID/hash до mutation. Encrypted Blob
custody receipt используется затем для one-use due execution. Persistence также предоставляет
bounded exact Scheduler command/receipt outbox relay с hash-bound idempotent
publication, owner-scoped Status с authoritative created/updated timestamps и
атомарный replayable client-realtime transition log. Due command уже atomically проходит
`scheduled|cancel_requested -> due -> dispatching`, сохраняет exact
run/schedule/lease fence и acceptance receipt outbox. Delivery-intent acceptance
может завершить operation и записать terminal Scheduler result только при живом
exact lease. Отдельная execution unit реализует owner-local due orchestration
через compile-isolated ports: one-use body read с проверкой custody size/digest,
stable delivery-intent request, fenced accepted/failed transition, terminal
Scheduler receipt и durable cleanup-pending outcome. Отдельная event-adapter
unit уже строит exact Scheduler command envelope с runtime/grant fences и
проверяет correlated Scheduler result до persistence mutation. Она также
строго допускает только due `ScheduledJobCommandV1` своего exact JobKind,
связывает command metadata, scheduled time и RuntimeLease и строит стабильные
acceptance/terminal Scheduler receipts без синтетических command-объектов.
Отдельная runtime-adapters unit реализует bounded Blob write,
receipt-bound Blob read,
terminal-reason-bound custody release и exact delivery-intent `request_rpc`
через один sequential managed-control port. Managed runtime реализует inherited
control authentication, owner-local Storage binding, method-exact
Schedule/Cancel/Status routing и cursor-based client realtime publication в
единый Gateway SSE stream. Schedule и Cancel используют разные exact contracts
по ADR-0345. Runtime также получает fenced Event Hub credential, публикует exact
Scheduler command/receipt outbox и принимает correlated schedule results только
если causation ссылается на owner-local сохранённый command. Due-command
execution подключён к отдельным execution/runtime/store adapter units: exact
Scheduler command декодируется до claim, Retryable не подтверждает JetStream
delivery, а accepted/rejected подтверждает его только после durable terminal
receipt. Отдельная assembly unit материализует runtime binary, exact descriptor,
settings schema и Storage bundle как unsigned release fragment без runtime
поведения. Development release compiler принимает этот exact fragment в общий
подписываемый distribution input. Live end-to-end contour ещё не реализован.
Этот ADR не открывает workflow gate сам по себе.

Уточняет:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0214](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0330](ADR-0330-provider-neutral-communication-delivery-intent-workflow.md);
- [ADR-0333](ADR-0333-delivery-intent-target-bound-blob-materialization.md);
- [ADR-0339](ADR-0339-capability-routed-module-request-rpc.md);
- [ADR-0343](ADR-0343-capability-routed-blob-custody-release.md).

## Контекст

Clean-room reconstruction требует отдельный workflow для scheduled delivery,
acceptance и cancellation. Это не таймер внутри Mail/Telegram/WhatsApp/Zulip,
не поле `deliver_at` в Communications и не business scheduler в Kernel.

Responsibilities уже разделены:

- Communications владеет canonical evidence;
- provider integrations владеют compose и provider execution;
- `communication_delivery_intent` владеет немедленным provider-neutral
  acceptance одного outbound intent;
- Scheduler владеет time policy, due points, run leases и technical dispatch;
- новый workflow должен владеть только lifecycle отложенной доставки.

Существующий owner-control `UpsertSchedulerSchedule` не является module API.
Workflow не получает owner session, не вызывает Gateway и не импортирует
Kernel implementation. Для independently restartable owners нужен durable
module-to-Scheduler contract через event spine.

## Решение

Вводится отдельный workflow owner `communication_delayed_delivery` с единицами
сборки:

```text
hermes-communication-delayed-delivery-api
hermes-communication-delayed-delivery-core
hermes-communication-delayed-delivery-persistence
hermes-communication-delayed-delivery-execution
hermes-communication-delayed-delivery-event-adapters
hermes-communication-delayed-delivery-runtime-adapters
hermes-communication-delayed-delivery-store-adapters
hermes-communication-delayed-delivery-runtime
hermes-communication-delayed-delivery-assembly
```

API содержит generated Schedule/Cancel/Status/realtime contracts. Core
валидирует lifecycle и cancellation race. Persistence владеет workflow
operation, body custody reference, Scheduler correlation и owner-local inbox /
outbox. Execution unit владеет только fenced due orchestration через public
ports. Event-adapters unit владеет exact Scheduler `DurableEnvelopeV1`
construction/admission mapping и не выполняет transport I/O. Runtime обслуживает
client contract и managed lifecycle. Runtime-adapters unit владеет только
Kernel-routed Blob/request transport, не persistence и не lifecycle. Assembly
создаёт отдельный signed runtime/storage fragment. Store-adapters unit
реализует только execution persistence port и явно отображает owner-local
execution models в persistence models без SQL или transport logic.

Ни одна unit не импортирует Communications implementation, integration
runtime/persistence, Scheduler implementation/persistence или Kernel
implementation.

## Согласование с Kernel, Scheduler и delivery-intent

### Kernel и Core

Kernel:

- проверяет registration, exact capability grants, runtime generation и grant
  epoch;
- маршрутизирует opaque durable envelopes и exact `request_rpc`;
- не декодирует body, conversation identity или schedule semantics;
- не создаёт schedule и не выбирает provider.

Core Gateway:

- передаёт generated Schedule/Cancel/Status request в exact workflow runtime;
- возвращает immediate workflow receipt;
- доставляет client-safe invalidation через общий replayable SSE;
- не вызывает Scheduler или provider integration от имени клиента.

### Scheduler

Scheduler остаётся единственным platform time authority. Он получает только
exact durable schedule-control commands:

```text
scheduler.schedule.command.v1
  EnsureOneShot
  CancelOneShot

scheduler.schedule.result.v1
  ensured | cancelled | too_late | rejected
```

Module-originated schedule control проходит через producer outbox,
`DurableEnvelopeV1`, NATS JetStream и Scheduler inbox. Result возвращается
через Scheduler outbox тем же event spine. Это не `control_rpc`,
`client_rpc`, direct socket или cross-owner SQL.

One-shot due dispatch использует существующий `ScheduledJobCommandV1`:

```text
job_kind = communication.delayed_delivery.execute.v1
scope_id = delayed_operation_id
trigger_kind = scheduled
```

Command не содержит body, conversation/provider identity или Blob reference.
Scheduler хранит opaque scope, schedule/run identity, policy, due point и
technical result; workflow хранит business orchestration state.

### Delivery intent

После durable acceptance due command owner-local executor:

1. дедуплицирует Scheduler command в workflow inbox;
2. создаёт/возвращает existing execution по exact run/lease fence;
3. получает private body из workflow-owned Blob custody;
4. вызывает public `communication.delivery_intent.command` через exact
   capability-routed `request_rpc`;
5. сохраняет acceptance receipt;
6. публикует fenced Scheduler terminal result.

Один и тот же `delivery_operation_id` используется при ambiguous retry.
`accepted` delivery-intent не означает provider completion. Provider terminal
delivery остаётся в delivery-intent status/realtime и не становится Scheduler
state.

## Public contract

### Schedule

Client передаёт:

- `protocol_major = 1`;
- exact non-zero 16-byte `delayed_operation_id`;
- exact non-zero 16-byte `delivery_operation_id`;
- canonical `conversation_id`;
- optional canonical `reply_to_message_id`;
- private non-empty UTF-8 body не более 64 KiB;
- absolute UTC `deliver_at_unix_millis`.

`deliver_at` обязан быть не раньше чем через 5 секунд и не дальше чем через
366 дней от authenticated Kernel Clock reading. Client wall clock не является
authority. Один request ограничен 128 KiB.

Schedule сначала durably сохраняет workflow operation и Blob custody receipt,
затем публикует idempotent `EnsureOneShot`. Immediate receipt имеет состояния
`accepted` или `existing`; он не обещает, что Scheduler уже подтвердил
schedule.

### Cancel

Cancel принимает exact `delayed_operation_id` и expected workflow revision.
Он сохраняет `cancel_requested`, затем публикует idempotent
`CancelOneShot`.

Scheduler является authority cancellation race:

- `cancelled` допустим только до durable acceptance due dispatch;
- после accepted/running due run возвращается `too_late`;
- workflow не помечает operation cancelled до Scheduler result;
- cancellation после delivery-intent acceptance не является undo.

Повтор Cancel идемпотентен. Stale revision, terminal operation и foreign owner
fail closed.

### Status

Status возвращает только:

- delayed operation ID;
- sanitized state и monotonic revision;
- requested due time;
- delivery-intent ID после acceptance;
- typed bounded error code;
- client-safe timestamps.

Body, Blob reference/proof, provider/account identity, Scheduler runtime
coordinates и raw error text не возвращаются.

## Durable lifecycle

```text
accepted
  -> schedule_pending
  -> scheduled
  -> due
  -> dispatching
  -> delivery_accepted

schedule_pending | scheduled
  -> cancel_requested
  -> cancelled | scheduled

accepted | schedule_pending | scheduled | due | dispatching
  -> failed
```

`scheduled` означает только Scheduler acceptance. `delivery_accepted` означает
только delivery-intent acceptance. Provider delivered/failed не копируется в
delayed workflow.

Каждый transition требует current workflow revision. Job execution additionally
requires exact Scheduler run ID, schedule revision, lease epoch/expiry and
current runtime/grant fences. Stale worker не может materialize body, вызвать
delivery-intent, завершить run или удалить custody.

## Private body custody

Workflow PostgreSQL не хранит plaintext body. До due body находится в
workflow-owned encrypted Blob custody:

- Blob write uses exact owner/runtime/operation binding;
- persistence хранит только opaque reference, digest, size и custody proof;
- runtime получает one-use scoped read lease только для current fenced due
  execution;
- terminal cancellation, rejection or delivery-intent acceptance удаляет
  workflow custody через idempotent cleanup command;
- cleanup failure остаётся durable technical retry и не меняет business
  outcome;
- body не попадает в Scheduler payload, subjects, logs, errors, health,
  realtime или status.

Blob orphan cleanup является bounded platform maintenance, а не скрытым timer
workflow runtime.

## Units и SRP

```text
delayed-delivery API
  generated client request/status/realtime schemas

delayed-delivery core
  time bounds, lifecycle and cancellation race policy

delayed-delivery persistence
  operation, custody reference, inbox/outbox and execution fences

Scheduler durable adapter
  exact Ensure/Cancel command and result mapping

owner-local Job Executor
  due claim, Blob materialization and delivery-intent request

assembly
  descriptor, settings, Storage bundle and release fragment
```

Scheduler protocol owns generic schedule/run contracts. Delayed workflow owns
only `communication.delayed_delivery.execute.v1` semantics. Delivery-intent
owns outbound acceptance. Сходство полей не является причиной объединить units.

## Phase gate `communication_delayed_delivery_v1`

Gate открывается только вместе с:

1. implemented `scheduler_v1`, включая live successor restart/revoke и hot
   reconciliation;
2. exact module-originated Scheduler command/result contracts и grants;
3. девятью отдельными delayed-delivery packages и Cargo boundaries;
4. generated Schedule/Cancel/Status/realtime contracts и hard bounds;
5. owner-local Storage bundle, idempotent operation and state transitions;
6. encrypted Blob custody without plaintext workflow SQL;
7. exact one-shot schedule correlation and `ScheduledJobCommandV1` executor;
8. cancellation-race authority and `too_late` semantics;
9. exact delivery-intent `request_rpc` with stable operation ID;
10. managed client RPC and shared SSE invalidation;
11. restart, stale lease, revoke, NATS outage, Scheduler outage, Blob outage,
    duplicate, cancellation race and ambiguous request negatives;
12. live managed contour through real Scheduler and delivery-intent runtimes;
13. architecture, SRP, Cargo, Clippy and full test gates.

Gate остаётся `planned`, пока весь evidence не пройден. Skeleton UI может
показывать reference layout, но не fake scheduled records или fake completion.

## Отклонённые варианты

### Таймер внутри workflow runtime

Не переживает restart и создаёт второй источник времени/расписания.

### Вызов Kernel owner-control из workflow

Подменяет module authority owner session и связывает workflow с private Kernel
API.

### Client координирует workflow и Scheduler

Создаёт неатомарный cross-owner saga в UI и ломает mobile/headless clients.

### Schedule хранится в Communications

Смешивает canonical evidence domain с outbound orchestration.

### Provider-specific scheduled send

Не даёт единых cancellation/receipt semantics и смешивает provider capability
с provider-neutral workflow. Если provider имеет собственную scheduled-send
функцию, она остаётся отдельным integration capability.

### Body в Scheduler payload или schedule state

Передаёт private content platform owner и нарушает payload/logging boundary.
