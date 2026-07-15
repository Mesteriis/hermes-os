# Контракт взаимодействия компонентов

Статус: Принятая архитектура, реализация не начата
Дата: 2026-07-15

Источники active policy:

- [ADR-0200: Модульная модель и изоляция runtime](../adr/ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](../adr/ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](../adr/ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](../adr/ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](../adr/ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](../adr/ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](../adr/ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md).
- [ADR-0207: Канонический реестр бизнес-доменов Hermes](../adr/ADR-0207-canonical-business-domain-registry.md).
- [ADR-0208: Allowlist разработки доменов и запрет проекций](../adr/ADR-0208-domain-development-allowlist-and-projection-freeze.md).
- [ADR-0209: Kernel Event Hub и контроль подписок](../adr/ADR-0209-kernel-event-hub-and-subscription-control-plane.md).
- [ADR-0210: Telemetry Hub и локальная диагностика](../adr/ADR-0210-telemetry-hub-and-local-diagnostics.md).

Архивные ADR и legacy executable contract не являются policy новой системы.

## Компоненты

- Core runtime — supervisor, module registry, capability router и внешний API
  gateway без business logic. Его список обязанностей закрыт ADR-0206.
- Event Hub — Kernel control plane для event catalog, subscription
  reconciliation, NATS permissions и delivery health без чтения payload.
- Telemetry Hub — Kernel policy/control surface и отдельный managed Collector
  для logs, metrics, traces и crash/lifecycle diagnostics.
- Domain module — владелец одного bounded context и его durable truth.
- Integration plugin — владелец provider protocol, auth/session runtime,
  cursor, operational contract/projection и neutral evidence mapper, но не
  business truth.
- Provider experience surface — bundled frontend controller конкретного
  integration-плагина; это не domain и не удалённо загружаемый код.
- Desktop client — Vue/Tauri application, использующее local embedded Gateway
  profile.
- Android client — planned first-party application, использующее тот же public
  contract через local embedded или paired remote Gateway profile.
- Workflow module — координатор нескольких public contracts.
- Platform service — storage, events, vault, blobs, clock и scheduler.
- Product projection module — зарезервированная будущая роль; implementation
  полностью заблокирована ADR-0208.

Каждый independently restartable runtime является отдельным OS-процессом.

Supervisor является подсистемой Kernel. Он управляет managed PostgreSQL,
PgBouncer, NATS и module runtimes, но не является отдельным обязательным
Hermes-процессом. Kernel перезапускается Tauri или OS watchdog.

Kernel обязан достичь `recovery_only` без PostgreSQL, PgBouncer, NATS, vault и
module runtimes. Domain, workflow или integration failure переводит только
затронутые capabilities и Kernel в `degraded`; глобальный `fatal` зарезервирован
для потери доверия к самому control plane.

## Допустимые interaction kinds

- `local_call` — вызов внутри одного module implementation;
- `control_rpc` — lifecycle и health через Protobuf RPC по Unix socket;
- `query_rpc` — синхронный read-only запрос через capability router;
- `request_rpc` — синхронная typed операция с immediate result;
- `durable_command` — требование изменить state через PostgreSQL outbox и NATS
  JetStream;
- `event` — immutable факт владельца state через outbox и JetStream;
- `observation` — факт внешнего наблюдения с provenance и cursor;
- `result` — terminal или промежуточный результат durable command;
- `ack` — подтверждение canonical persistence или terminal handling;
- `projection` — rebuildable read/search model, построенная из owned facts;
- `client_rpc` — owner-specific ConnectRPC query/request/command через Gateway;
- `client_realtime` — replayable multiplexed SSE через Gateway;
- `client_blob` — bounded HTTP transfer по opaque BlobRef capability;
- `host_bridge` — только OS/bootstrap capability desktop или Android.
- `telemetry_signal` — bounded structured log, metric, trace или lifecycle
  record через private local Telemetry Collector channel.

Contract выбирает один delivery mode. Одна и та же операция не отправляется
одновременно как RPC и NATS command.

Interaction kind `projection` зарезервирован контрактом, но production
projection packages, runtimes, schemas и consumers запрещены ADR-0208 до
отдельной разблокировки.

`telemetry_signal` не является event, command или canonical audit record. Он не
проходит через NATS/PostgreSQL и не содержит business payload.

## Маршрутизация

Прямое module-to-module соединение запрещено:

```text
source module
    ↓ public envelope
core capability/event router
    ↓
target module or workflow
```

Ядро проверяет identity, capability, contract и protocol version, но не
интерпретирует business payload.

Cross-domain behavior принадлежит workflow:

```text
source domain event
    ↓
workflow
    ↓
target domain command
```

## Граница integration-плагина

Provider различия сохраняются до operational frontend surface. Mail, Telegram,
WhatsApp и Zulip не обязаны изображать один универсальный набор операций.
Каждый bundled integration-плагин предоставляет собственный versioned
operational contract и может использовать общий presentation shell через
узкие provider-neutral props.

Контекстные домены не видят этот operational contract. Плагин сам публикует
отдельное neutral evidence observation:

```text
External provider
        ↓
Integration plugin
        ├─→ operational projection → provider experience
        └─→ neutral evidence → NATS → context/memory domains
```

Provider identity сохраняется в provenance, но не определяет domain behavior.
Core маршрутизирует оба contract family по manifest/capabilities и не выполняет
provider-to-domain mapping.

Integration plugins и frontend experiences поставляются в allowlisted
application bundle. Plugin store, runtime download и remote frontend code не
поддерживаются.

## Граница клиентских приложений

Desktop и Android общаются только с Core Gateway:

```text
Desktop / Android
        ↓ ConnectRPC + SSE + bounded HTTP
Core Gateway
        ↓ capability router
module runtimes
```

- ConnectRPC/Protobuf обслуживает typed queries, requests и commands.
- Один multiplexed SSE stream на active client session даёт replayable
  realtime и terminal command results.
- Обычный HTTP используется только для health/readiness, OAuth callbacks,
  blobs и SSE.
- Tauri/Android host bridge обслуживает только OS capabilities и bootstrap, а
  не business API.
- NATS, PostgreSQL, PgBouncer и module sockets клиенту не видны.

Desktop local profile использует private loopback HTTP. Paired Android profile
использует защищённый HTTP/2 baseline и preferred HTTP/3 over QUIC после
conformance проверки. HTTP/3 меняет network transport, но не Protobuf contracts
или application semantics. При блокировке UDP разрешён наблюдаемый fallback на
защищённый HTTP/2; 0-RTT запрещён.

Android background suspension не останавливает Kernel. На resume клиент
восстанавливает SSE по собственному cursor. Offline cache не является
canonical truth, а push notification не переносит domain state/private content.

## Control plane

Control plane не зависит от NATS и остаётся доступным при его отказе. Он
поддерживает handshake, manifest validation, start, quiesce, drain, stop,
health и capability renewal/revocation.

Module process не передаёт bootstrap secrets через argv, environment или logs.

## Infrastructure lifecycle

Supervisor subsystem не зависит от PostgreSQL или NATS и остаётся доступным в
recovery mode. Managed services могут перезапускаться только по
service-specific bounded policy. External services никогда не получают signal
от Kernel. Restart процесса не удаляет и не заменяет PostgreSQL, JetStream,
vault или provider session state.

## Data plane

- PostgreSQL является canonical source of truth.
- Mutation и outbox append выполняются одной локальной транзакцией owner
  module.
- NATS JetStream выполняет durable delivery, fan-out и replay.
- Consumer фиксирует inbox deduplication и local mutation до NATS ACK.
- End-to-end semantics — at least once; exactly-once не обещается.
- Ordering существует только внутри явного partition key.
- Retry bounded; `unknown_outcome` не повторяется автоматически.
- Private bodies, documents, media и secrets не проходят через NATS; сообщения
  используют bounded metadata и opaque blob/evidence references.

## Storage boundary

Module runtime использует собственную PostgreSQL role и ходит через PgBouncer.
Он не читает чужие business tables и не получает direct administrative
connection. Cross-module SQL и cross-module foreign keys запрещены.

Shared outbox/inbox/event tables являются platform state и защищаются grants и
RLS по module identity.

## Запрещено

- domain-to-domain implementation import;
- integration-to-domain implementation import;
- domain dependency/subscription на provider operational contract;
- shared presentation component с импортом provider generated types;
- client-to-module/NATS/PostgreSQL direct connection;
- business REST/JSON рядом с эквивалентным ConnectRPC method;
- business operation через Tauri/Android host bridge;
- HTTP/3-only remote API, raw QUIC RPC или 0-RTT requests;
- module-to-module socket;
- чтение или запись чужих tables как API;
- shared in-memory production event bus;
- durable fire-and-forget через Core NATS;
- secrets, private content или user-provided identifiers в NATS subjects и
  diagnostics;
- unbounded queues, retries или connection pools;
- automatic runtime/topology fallback;
- remote/plugin-store executable code loading.

## Состояние реализации

Этот документ описывает принятую clean-room архитектуру, а не существующий
runtime. Legacy scripts и код в `references/backend-legacy/` не являются
доказательством реализации. Статус изменяется только после появления новых
executable guards и process-level integration tests.
