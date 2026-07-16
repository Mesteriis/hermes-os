# ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано; production packages, schema и runtime
Job Platform ещё не созданы

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR уточняет запрет ADR-0206 на business schedulers внутри Kernel:
планирование является отдельной platform capability, а код задания всегда
остаётся внутри module-владельца. ADR не разблокирует запрещённые ADR-0208
domains или product projections; Graph rebuild остаётся только будущим
примером job kind до отдельного решения.

Scheduler и Job Plane не входят в `kernel_recovery_only_v1`. Их packages,
storage, NATS consumers и runtime activation разрешаются только отдельным
`scheduler_v1` gate ADR-0225 после Clock, Storage, NATS, Vault, Telemetry,
module control plane и managed-launch trust.

## Контекст

Hermes должен выполнять несколько разных видов фоновой работы:

- периодический опрос внешних систем, например получение почты;
- задания, вызванные domain или integration event;
- ручные и отложенные операции;
- длительные AI jobs с progress, checkpoint и cancellation;
- будущие rebuild и maintenance operations.

Эта работа должна переживать restart Scheduler, module runtime и NATS,
поддерживать bounded concurrency/retry и не переносить business code в Kernel.
Расписания и operational policy должны меняться без остановки системы, но
динамическая загрузка произвольного executable code, scripts или Rust libraries
из database запрещена.

Один центральный worker с реализациями разных owners нарушил бы
compile isolation, failure isolation и ownership. Использование только NATS
также недостаточно: broker доставляет сообщения, но не является canonical
источником schedule configuration, module checkpoints и business result.

## Область применения

ADR обязателен для любой технической фоновой работы независимо от owner:

- integrations: provider polling, reconnect, backfill, attachment download и
  outbound delivery;
- domains: reminders, retention, validation и owner-specific maintenance;
- AI: analysis, extraction, classification, summarization и embeddings;
- workflows: delayed step, timeout, compensation и resumable coordination;
- platform: outbox delivery, bounded cleanup, backup verification и technical
  reconciliation;
- будущие engines и projections после их явной разблокировки.

Ни один owner не получает собственный обходной scheduler, detached timer или
необъявленную background queue. Различается только owner handler и его
параметры; registration, scheduling, delivery и lifecycle следуют этому ADR.

## Решение

Вводится platform capability **Hermes Job Platform** из двух частей:

1. отдельный independently restartable Scheduler runtime;
2. owner-local Job Executor внутри каждого module runtime, которому нужна
   фоновая работа.

Kernel supervises Scheduler как managed module runtime и перед каждым launch
проверяет его exact-byte binding по ADR-0219, затем identity и capabilities.
Kernel не хранит schedules, не создаёт job rows и не интерпретирует job payload.
Event Hub управляет NATS catalog, consumers, permissions и delivery health, но
не вычисляет время запуска и не исполняет jobs.

Базовый поток:

```text
time trigger      domain/integration event      manual client command
     ↓                        ↓                           ↓
 Scheduler              owner/workflow                owner API
     └───────────────────────┴───────────────────────────┘
                             ↓
                  producer PostgreSQL outbox
                             ↓
                     NATS JetStream command
                             ↓
                    target module inbox
                             ↓
                    owner-local Job Executor
                             ↓
          owner state/checkpoint + result/event outbox
```

Scheduler является producer только для time-based trigger. Event-triggered
работа создаётся consumer/workflow владельца, а manual work — owner-specific
command handler. Все три пути используют один Job Command Envelope, но не
обязаны проходить через Scheduler как центральный proxy.

## Термины

- `JobKind` — versioned тип технической фоновой работы, объявленный module,
  например `mail.fetch.v1`, `ai.analyze-evidence.v1` или
  `documents.extract-text.v1`;
- `JobSchedule` — изменяемая time policy для одного JobKind и scope;
- `JobRunId` — стабильный ID конкретного запуска;
- `JobCommand` — durable требование владельцу принять запуск;
- `JobExecution` — owner-local состояние фактического выполнения;
- `JobCheckpoint` — owner-local durable позиция продолжения длительной работы;
- `JobProgress` — bounded sanitized progress metadata;
- `JobResult` — terminal результат запуска;
- `JobLease` — ограниченное по времени право исполнять конкретный run;
- `ScheduleRevision` — версия persisted schedule configuration.

Термин `Task` не используется для технического job, потому что `Tasks` является
отдельным business domain Hermes.

## Где находится исполняемый код

Код задания принадлежит module-владельцу и компилируется в его runtime.
Универсальная физическая форма следует owner topology ADR-0211/ADR-0212:

```text
backend/src/<owner-kind>/<owner>/<core-or-implementation>/jobs/<job>.rs
    owner-specific job algorithm and policy

backend/src/<owner-kind>/<owner>/<adapter>/
    external protocol or technical adapter, when required

backend/src/<owner-kind>/<owner>/persistence/
    cursor, checkpoint and execution persistence

backend/src/<owner-kind>/<owner>/runtime/
    handler registration, lifecycle, cancellation and concurrency
```

Для integration `owner-kind` равен `integrations`, для domain — `domains`, для
workflow — `workflows`, для engine — `engines`, для platform owner —
`platform` или `services` согласно ADR-0211. Package не создаётся только ради
папки `jobs`: handler остаётся внутри cohesive owner core/implementation.

Scheduler не импортирует packages конкретного owner, не содержит глобальный
`match` по domains/providers/job kinds и не знает Rust function name. В
PostgreSQL и NATS запрещено хранить executable code, SQL fragments, module
paths, dynamic library paths или scripts.

Примеры применяют одну и ту же границу:

| Owner | JobKind | Где находится код | Чем владеет executor |
|---|---|---|---|
| Mail integration | `mail.fetch.v1` | Mail core + IMAP adapter | provider cursor и полученные records |
| AI domain | `ai.analyze-evidence.v1` | AI implementation + model adapter | analysis execution и AI result candidate |
| Documents domain | `documents.extract-text.v1` | Documents implementation + parser adapter | extraction checkpoint и document-owned result |
| Calendar domain | `calendar.deliver-reminder.v1` | Calendar implementation | reminder execution и Calendar event outcome |
| Workflow | owner-specific delayed step | конкретный workflow implementation | saga step/checkpoint/compensation |
| Platform service | owner-specific maintenance kind | соответствующий platform owner | только собственное technical state |

Graph/search/context rebuild jobs не создаются, пока product projections
заблокированы ADR-0208.

`ModuleDescriptorV1` capability объявляет descriptor JobKind:

- stable owner и job kind ID;
- contract и payload version;
- допустимые trigger kinds;
- default schedule template, если он существует;
- concurrency/overlap capabilities;
- cancellation/checkpoint capabilities;
- resource class и bounded limits;
- minimum compatible runtime protocol.

Descriptor описывает capability, а не передаёт реализацию.

## Package boundaries

Общий technical contract получает самостоятельный platform protocol, потому
что его потребляют Scheduler и несколько независимых module runtimes:

```text
backend/src/platform/scheduler/protocol/       hermes-scheduler-protocol
backend/src/platform/scheduler/implementation/ hermes-scheduler
backend/src/platform/scheduler/persistence/    hermes-scheduler-persistence
backend/src/platform/scheduler/runtime/        hermes-scheduler-runtime
```

`hermes-scheduler-protocol` содержит только JobKind descriptor, job payload
messages, schedule/execution lifecycle states и typed errors. Он не определяет
второй outer envelope: любой job command/result использует
`DurableEnvelopeV1` из `hermes-events-protocol` ADR-0220. Scheduler protocol не
содержит SQLx, NATS client, provider SDK, domain types или runtime bootstrap.

Owner-local handler остаётся в существующих `domain`, `integration`,
`workflow` или `engine` packages владельца. Отдельный общий
`hermes-worker-runtime`, registry всех handlers или Celery-like application
package запрещён.

## Владение persisted state

Исполняемый код и persisted state разделяются по ответственности.

### Scheduler владеет

- revisioned JobSchedule;
- enabled/disabled/tombstone state;
- `next_due_at`, `last_fired_at` и bounded misfire metadata;
- time-triggered JobRunId и dispatch record;
- schedule lease/fencing state;
- sanitized acceptance/terminal control status, полученный из owner result;
- schedule change и dispatch outbox records под scheduler identity.

Это technical control state, а не business result или product projection.

### Module-владелец владеет

- inbox deduplication конкретного command;
- JobExecution state, execution lease, heartbeat и attempt history;
- JobCheckpoint и resumable cursor;
- provider/domain-specific input validation;
- фактический business или operational result;
- result/event outbox;
- собственные accounts, provider cursors, messages, documents и другие owned
  records.

Scheduler для любого owner хранит только факт, что конкретный JobKind для
opaque scope должен быть вызван по schedule. Target module хранит собственные
cursor/checkpoint, execution outcome и owned result. Например, Mail хранит IMAP
cursor и provider records, AI — analysis checkpoint и candidate result,
Documents — parser checkpoint и document-owned extraction state. Credentials,
private input и provider session material остаются в соответствующих protected
owner/Vault boundaries и никогда не попадают в Scheduler state.

### Job configuration не является module settings

`JobSchedule`, enabled/tombstone, due time, retry, overlap, misfire, lease и
run state остаются canonical PostgreSQL state Scheduler. Kernel Settings
Registry ADR-0222 не хранит, не копирует и не применяет эти records.

Module settings могут влиять на owner-local behavior executor, например bounded
batch size, но не заменяют `JobSchedule` и не создают второй timing source of
truth. Общий client screen может визуально показать Scheduler section рядом с
module settings; queries и mutations всё равно идут в разные owner contracts и
не обещают cross-owner transaction.

### Shared platform state

Outbox/inbox/event tables остаются shared technical tables ADR-0202 с
role-aware RLS. Scheduler и module runtimes видят только строки собственной
identity. NATS JetStream хранит delivery message до acknowledgement/replay, но
не заменяет PostgreSQL source of truth.

Ни Scheduler, ни Gateway не читают owner tables для построения общего job
экрана. Client query получает schedule control state у Scheduler и подробное
execution/business state у owner через contracts; cross-owner SQL запрещён.

## Registration и startup reconciliation

При старте используется следующий protocol:

1. Module runtime выполняет `Hello`/`Describe` и передаёт exact bounded
   `ModuleDescriptorV1` с JobKind descriptors.
2. Kernel применяет registration state, runtime identity, protocol
   compatibility и effective GrantSet ADR-0215.
3. Проверенный descriptor становится доступен Scheduler через platform
   capability/catalog protocol; Kernel не интерпретирует его business fields и
   не пишет schedule tables.
4. Scheduler сверяет active JobKind catalog со своим persisted state.
5. Только после успешной сверки time-triggered capability получает readiness.

Reconciliation fail closed:

- JobKind pending/suspended/revoked module или без effective grant не
  регистрируется;
- несовместимая contract version блокирует schedule до provider call или owner
  mutation;
- schedule без активного handler становится `blocked_missing_handler`;
- удалённый JobKind не удаляет history и schedule автоматически;
- повторная регистрация того же `ModuleDescriptorV1` после restart не создаёт
  duplicate schedule или run;
- Kernel restart не изменяет persisted schedule configuration.

### Default schedules

Default schedule является versioned template из `ModuleDescriptorV1`, а не
неизменяемым hard-coded timer внутри runtime.

Template является owner-declared complete initial policy: он содержит
trigger/time policy и все обязательные поля валидного schedule из раздела
`Scheduling policies`, включая default `overlap_policy`, `misfire_policy`,
concurrency key/maximum parallelism, timeout/deadline и bounded retry policy,
а также jitter и timezone/DST policy, когда они применимы. После первого
создания эти значения становятся revisioned canonical `JobSchedule` Scheduler
и могут изменяться только через его typed commands. Live IDs/revisions,
enabled/tombstone и due state, leases, runs и user overrides не входят в
default template и никогда не записываются обратно в `ModuleDescriptorV1`.

- Global default создаётся Scheduler только при первом появлении identity, если
  persisted schedule и tombstone отсутствуют.
- Scope-specific default создаётся только после owner command
  `EnsureSchedule`, когда scope уже существует: account для integration,
  document/evidence scope для owner job или workflow instance для delayed step.
- Existing schedule никогда не перезаписывается default template при restart.
- Изменение default в новой версии module не меняет существующие schedules
  молча; нужна explicit schedule migration или `ResetToDefault` command.
- Disable сохраняет schedule и прекращает будущие runs.
- Delete создаёт durable tombstone, поэтому startup reconciliation не
  воскрешает пользовательски удалённое расписание.
- ResetToDefault является отдельным authorized command, увеличивает revision и
  явно удаляет tombstone.

Таким образом module может гарантировать наличие разумного initial schedule,
но не отбирает у пользователя последующее управление.

## Job command payload

Job dispatch использует обычный `DurableEnvelopeV1.command`. Общие
`message_id`, logical `command_id`, source/target, partition, deadline,
idempotency, causation/correlation, trace и source fence принадлежат outer
envelope ADR-0220 и не дублируются Scheduler schema.

Typed job payload содержит только Scheduler/job semantics:

- `job_run_id`;
- `job_kind` и `job_contract_version`;
- opaque `scope_id`, когда применимо;
- `schedule_id` и `schedule_revision` для time-triggered run;
- `trigger_kind` и `scheduled_for`;
- execution lease scope/epoch;
- bounded owner input или opaque owner-controlled reference.

Subject содержит только stable owner/contract tokens. Account IDs, private
identifiers и payload не помещаются в subject, logs или health.

## Durable acceptance и выполнение

Module consumer не удерживает JetStream ACK на всё время длительного job.

```text
BEGIN
  deduplicate command in owner-visible inbox
  create or return existing JobExecution
  persist initial execution/checkpoint state
  append DURABLE_ACCEPTANCE Ack-envelope when required
COMMIT
JetStream ACK

owner-local executor claims durable JobExecution
  → running / heartbeat / checkpoint
  → succeeded | failed | cancelled | expired | unknown_outcome
  → terminal result + optional progress/domain event outbox
```

JetStream ACK означает только broker acknowledgement после durable owner inbox
acceptance. При необходимости отдельный `AckMetadataV1` сообщает
`DURABLE_ACCEPTANCE`; это не один и тот же protocol. Terminal outcome
доставляется только `result`, progress — отдельным event. Crash после broker
ACK не теряет работу: owner-local executor снова находит persisted non-terminal
execution.

Длительная работа обязана поддерживать bounded execution lease. Stale worker
не может checkpoint или завершить run после lease epoch change.

## Scheduling policies

Поддерживаются только явно versioned policies:

- one-shot `at`;
- `cron` с timezone и определённым DST behavior;
- `fixed_interval` от planned fire time;
- `fixed_delay` от terminal completion;
- manual trigger;
- delayed/deferred command;
- event-triggered command, создаваемый owner consumer или workflow.

Для каждого schedule обязательны:

- overlap policy: `forbid`, `queue`, `coalesce_latest` или explicitly bounded
  `allow`;
- misfire policy: `skip`, `fire_once` или `catch_up_bounded`;
- concurrency key и maximum parallelism;
- timeout/deadline;
- bounded retry policy;
- optional bounded jitter;
- timezone/DST policy для calendar schedules.

Unbounded catch-up, concurrency, queue и retry запрещены.

Policy выбирается owner contract, а не глобальным default для всех jobs:

- provider polling обычно использует `fixed_delay`, scope partition,
  `forbid` overlap и bounded jitter;
- AI analysis обычно использует event/manual trigger, idempotency по input и
  model/prompt revision, bounded concurrency по resource class;
- document processing использует checkpoint и `coalesce_latest`, если новая
  revision документа делает старую queued работу устаревшей;
- reminder использует one-shot `at` и explicit timezone/DST semantics;
- workflow timeout/compensation использует one-shot/deferred command и saga
  correlation.

Это примеры policy, а не особые архитектурные ветки. Каждый JobKind явно
объявляет поддерживаемые policies и bounded limits.

## Retry layers

Три разных механизма не смешиваются:

1. JetStream redelivery повторяет доставку до durable owner acceptance.
2. Owner execution retry повторяет только typed transient operation после
   acceptance.
3. Schedule misfire policy решает, что делать с пропущенным временем.

Каждый слой bounded и наблюдаем. Validation, authorization, incompatible
version и stale lease являются terminal. После неоднозначного внешнего
non-idempotent action результат становится `unknown_outcome`; automatic retry
запрещён.

End-to-end semantics остаётся at least once. Stable JobRunId, message ID и
idempotency key обязательны; exactly-once не обещается.

## Hot reload расписания

Без restart разрешено менять:

- schedule expression, interval или delay;
- enabled state;
- future effective time;
- concurrency, overlap и misfire policy;
- timeout, retry, jitter и resource limits;
- bounded owner input/reference.

Изменение выполняется только через typed command с expected
`ScheduleRevision`:

```text
BEGIN
  compare current revision
  persist next revision
  append schedule-changed outbox event
COMMIT
refresh active scheduler state
```

Текущий run закреплён за revision, с которой он был создан. Изменение влияет на
будущие runs. Disable не отменяет in-flight execution; cancellation является
отдельной authorized command. Scheduler периодически сверяет in-memory due set
с PostgreSQL, поэтому NATS outage или потерянное notification не оставляют
configuration навсегда устаревшей.

Прямая правка schedule tables, session-dependent `LISTEN/NOTIFY` как
единственный reload path и NATS KV как canonical configuration запрещены.

## Обновление исполняемого кода

Rust code не hot-loadится в существующий process, а Kernel и Scheduler не
скачивают и не устанавливают executable. Изменение job handler сначала
пересобирает только packages владельца и его runtime, после чего следует
explicit update ADR-0219:

1. Scheduler прекращает выдачу новых executions старому runtime;
2. старый runtime выполняет bounded drain/checkpoint и останавливается;
3. host updater/OS атомарно устанавливает signed bundled release либо владелец
   отдельно подтверждает новый owner-pinned `ManagedLaunchBinding`;
4. Kernel проверяет exact installed bytes по `DistributionManifestV1` либо
   owner-pinned `ManagedLaunchBinding`, затем совместимость `ModuleDescriptorV1`
   и job contract versions;
5. запускает новую process generation;
6. повышает lease epoch и явно переключает capability.

Queued commands старой contract version должны быть либо совместимы с новой
version, либо явно migrated/cancelled до cutover. Silent payload reinterpretation
и automatic fallback/rollback на старый binary запрещены.

## Failure behavior

| Отказ | Поведение |
|---|---|
| Scheduler runtime | новые time triggers временно не создаются; queued и owner-local jobs продолжаются |
| Один module runtime | останавливаются только jobs этого owner; persisted executions возобновляются после restart |
| NATS | producer outbox сохраняется; accepted owner-local jobs продолжаются |
| PostgreSQL/PgBouncer | новые claims/commits блокируются; processes остаются управляемыми и bounded reconnect |
| Event Hub | topology reconciliation unavailable; Scheduler не создаёт in-memory fallback transport |
| Telemetry Collector | execution продолжается; diagnostics degraded без подмены canonical state |
| Incompatible handler | schedule blocked; запуск и provider call не выполняются |

Scheduler claim использует короткую PostgreSQL transaction, row lease и
fencing, совместимые с PgBouncer transaction pooling. Session advisory lock не
используется. Атомарно создаются JobRunId, dispatch record, outbox message и
новый `next_due_at`, чтобы concurrent Scheduler instances не создавали два
разных run для одного fire point.

## Security и privacy

- JobKind descriptor и effective module grant обязательны; arbitrary
  target/function name запрещены. Publisher signature не является условием
  external registration по ADR-0215, но Scheduler и любой managed Job Executor
  запускаются только с verified `ManagedLaunchBinding` ADR-0219. Self-declared
  descriptor не является executable integrity proof.
- Schedule command авторизуется по owner capability и scope.
- Secrets, provider sessions, message bodies, documents, prompts и media bytes
  не хранятся в Scheduler и не передаются в NATS.
- Большой или private input передаётся через opaque owner record, `BlobRef` или
  `EvidenceRef` с capability/expiry, когда это разрешено owner contract.
- Logs, metrics, traces и health содержат только job/run identity, duration,
  state, bounded error class и sanitized resource metrics.
- Progress не является каналом для private content.
- Scheduler не получает Vault capability для provider credentials.
- Settings Registry не получает `JobSchedule`, JobRun, lease, checkpoint,
  retry/misfire state или owner job payload.

## Проверка решения

Перед изменением `Состояние реализации` обязательны tests:

- first registration создаёт default schedule ровно один раз;
- complete default template создаёт valid schedule, а отсутствие обязательной
  policy, включая `misfire_policy`, блокирует reconciliation до persistence;
- module/Kernel restart не создаёт duplicate и не меняет revision;
- user-modified schedule переживает restart и module upgrade;
- disable/delete tombstone не воскрешается reconciliation;
- scope-specific schedule создаётся только после owner `EnsureSchedule`;
- missing/incompatible JobKind blocks before dispatch;
- concurrent Scheduler claims создают один JobRunId на fire point;
- crash до/после schedule commit и до/после NATS publish acknowledgement;
- duplicate command создаёт одну owner JobExecution;
- ACK после durable acceptance и crash после ACK возобновляет execution;
- stale execution lease/epoch не может checkpoint или завершить run;
- owner overlap/concurrency policy соблюдается для integration, domain, AI,
  workflow и platform job fixtures;
- hot schedule update применяется к future run и не меняет in-flight revision;
- settings catalog не содержит Scheduler records, а composed client screen не
  превращает settings mutation в schedule mutation;
- disable, cancel, timeout, bounded retry и `unknown_outcome`;
- NATS outage/replay и Scheduler restart/misfire;
- owner restart не влияет на jobs соседнего module;
- explicit verified code replacement сохраняет compatible queued work;
- managed Scheduler/Executor update не запускает bytes без valid binding и не
  выполняет automatic fallback/rollback;
- diagnostics и persisted technical state не содержат private content или
  secrets.

Integration tests используют PostgreSQL, PgBouncer и NATS JetStream через
testcontainers. Live provider accounts не используются.

## Последствия

### Положительные

- schedules меняются без restart и без executable code в database;
- integrations, domains, AI, workflows и platform owners сохраняют собственный
  code/storage/lifecycle;
- Scheduler failure не завершает module runtimes;
- NATS, outbox/inbox и owner checkpoints дают durable restart semantics;
- изменение одного handler пересобирает только owner packages;
- Kernel и Event Hub не превращаются в business orchestration monolith.

### Цена

- Scheduler получает собственные persistence, leases и reconciliation tests;
- каждый owner обязан реализовать durable execution state и idempotency;
- result/status для клиента требует owner query, а не cross-owner SQL;
- schedule/handler version compatibility становится обязательной частью
  module rollout.

## Отклонённые варианты

### Kernel создаёт и исполняет jobs

Отклонено: Kernel начал бы знать business schedule, provider/account state и
handlers, нарушая закрытый список обязанностей ADR-0206.

### Один центральный Celery-like worker со всеми handlers

Отклонено: связывает owners compile-time и runtime, увеличивает rebuild fan-out
и превращает падение одного handler в общий failure domain.

### Хранить executable code или scripts в PostgreSQL

Отклонено: обходит signed distribution/owner-pinned launch binding, ломает
reproducible builds, code review, explicit update/rollback model и security
boundary.

### Использовать NATS как единственное job storage

Отклонено: JetStream обеспечивает durable delivery и redelivery, но не заменяет
canonical schedule revisions, owner checkpoints и business state.

### Event Hub одновременно является Scheduler

Отклонено: event topology/subscription reconciliation и вычисление времени
запуска имеют разные owners, причины изменения и failure semantics.

### Temporal как обязательный initial runtime

Отклонено для первого clean-room implementation: добавляет отдельный critical
workflow control/persistence service рядом с уже обязательными PostgreSQL и
NATS. Решение можно пересмотреть, если появятся доказанные многодневные durable
workflow requirements, которые Job Platform не покрывает.

### Apalis как cross-module architecture contract

Отклонено: библиотека может быть исследована как owner-local implementation
detail, но не должна владеть Hermes envelopes, module boundaries или
inter-module delivery semantics.

## Ссылки

- [NATS JetStream consumers](https://docs.nats.io/nats-concepts/jetstream/consumers)
- [NATS JetStream](https://docs.nats.io/nats-concepts/jetstream)
- [Temporal schedules](https://docs.temporal.io/develop/go/workflows/schedules)
- [Temporal self-hosted deployment](https://docs.temporal.io/self-hosted-guide/deployment)
- [Apalis](https://github.com/apalis-dev/apalis)
