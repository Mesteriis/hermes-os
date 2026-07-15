# ADR-0209: Kernel Event Hub и контроль подписок

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md).

Связано с:

- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md).

## Контекст

NATS JetStream обеспечивает durable delivery, replay и consumer state, но сам
по себе не является каталогом контрактов Hermes и не проверяет согласованность
distribution manifest, module manifests, capabilities и фактически созданных
streams/consumers.

Без единого control plane невозможно надёжно ответить:

- какие event contracts существуют и какой модуль ими владеет;
- кто имеет право публиковать и потреблять каждый contract;
- какие подписки объявлены, созданы и действительно готовы;
- где растут lag, redelivery и dead letters;
- является ли отсутствие consumer ошибкой, отключённой optional capability или
  ожидаемым состоянием lifecycle.

При этом пропуск каждого event через Kernel превратит его в bottleneck и общую
failure domain. Поэтому контроль topology и доставка payload должны оставаться
разделёнными.

## Решение

### Роль Event Hub

**Event Hub** — обязательная подсистема Kernel, являющаяся control plane над
event topology Hermes. NATS JetStream остаётся data plane.

```text
Module outbox relay ───────────────→ NATS JetStream ───────────────→ Consumer
                                       ↑
Kernel Event Hub ── catalog / ACL / stream-consumer reconciliation / health
```

Event Hub не проксирует normal event traffic и не требуется на hot path между
publisher и consumer. Он может быть временно недоступен без потери уже
сохранённых outbox, JetStream messages и consumer state.

### Event catalog

Event Hub строит versioned catalog только из подписанного distribution
manifest и проверенных module manifests. Для каждого contract catalog содержит
минимум:

- стабильные `owner`, `contract_name` и `contract_version`;
- message kind;
- разрешённых publishers и subscribers;
- required и optional subscription semantics;
- delivery, retry, dead-letter и retention profile;
- partition-key contract;
- максимальный размер envelope;
- privacy classification и разрешённые reference types;
- schema identity/hash;
- compatibility policy.

Два владельца одного contract, неизвестная version, несовместимый schema hash,
незаявленный publisher/subscriber или расширяющий wildcard fail closed до
готовности затронутого runtime.

Catalog является техническим rebuildable state. Он не хранит domain entities,
provider payload или business policy.

### Декларативные подписки

Module runtime не создаёт произвольную production subscription самостоятельно.
Manifest объявляет contract, durable consumer identity template, обязательность,
partition policy и resource budget. Event Hub проверяет declaration и
согласует её с выданными NATS permissions.

Subscription использует состояния:

```text
declared
  → authorized
  → provisioning
  → ready
  → lagging / degraded
  → draining
  → stopped

любое несовместимое состояние
  → blocked
```

Required subscription входит в readiness модуля. Optional subscription может
перевести только связанную capability в `degraded`. Отсутствующая, лишняя или
несовместимая subscription никогда не исправляется выдачей широкого wildcard.

### Reconciliation с NATS

После готовности NATS Event Hub:

1. проверяет JetStream identity и совместимость topology;
2. сверяет streams с contract catalog;
3. создаёт или обновляет только объявленные bounded consumers;
4. сверяет publish/subscribe ACL с module capabilities;
5. проверяет ack, delivery, backlog и resource limits;
6. публикует sanitized readiness и delivery health;
7. повторяет reconciliation после restart NATS или module runtime.

Event Hub не удаляет stream, consumer или retained messages только потому, что
manifest временно отсутствует или runtime не запустился. Destructive topology
change требует отдельного migration/authorization contract.

### Наблюдаемое состояние

Для каждой subscription Event Hub отслеживает только технические metadata:

- desired и observed state;
- runtime instance и durable consumer identity;
- last delivery/ack time;
- pending, ack-pending, lag и redelivery counters;
- retry/dead-letter state;
- последнее sanitized failure class;
- resource-budget pressure;
- readiness и drain progress.

Event Hub не читает и не сохраняет event payload. Он не создаёт второй
canonical event log. Durable payload history остаётся в owner outbox и NATS
согласно retention policy ADR-0201. Техническая история delivery передаётся в
Telemetry Hub без private content.

### Event envelope и tracing

Event Hub проверяет declared envelope contract и topology, но не перехватывает
каждый envelope для runtime validation. Producer и consumer adapters обязаны
проверять header, version и size на своей границе.

Envelope переносит `message_id`, `causation_id`, `correlation_id`, `trace_id` и
bounded versioned trace context. Произвольный baggage, user identifiers и
private content в trace context запрещены.

Event Hub связывает delivery telemetry только по техническим identifiers и не
копирует payload в logs, metrics, traces, health или diagnostics.

### Отказ и восстановление

При недоступности NATS Event Hub:

- сохраняет desired catalog в памяти из проверенных manifests;
- помечает observed topology как unavailable;
- не создаёт альтернативный in-memory transport;
- не подтверждает required subscriptions как ready;
- продолжает отдавать sanitized declared/last-observed diagnostics;
- выполняет reconciliation после восстановления NATS.

Crash Event Hub является отказом Kernel subsystem и требует bounded restart
Kernel внешним watchdog. Уже сохранённые messages и consumers не очищаются.
Несовместимая topology переводит затронутые capabilities в `blocked`, а не
запускает destructive repair.

### Client и operator surface

Через Core Gateway доступны только authorized sanitized views:

- catalog contracts без schema payload;
- publisher/subscriber topology;
- readiness и lifecycle state;
- lag, retry и DLQ counters;
- blocker class и recovery action category.

Raw event payload, private subjects, credentials и arbitrary NATS admin API
клиентам не выдаются. Paired Android не получает расширенные operator
capabilities автоматически.

## Запрещено

- пропускать normal event payload через Kernel как обязательный proxy;
- интерпретировать event payload или принимать business decisions;
- создавать subject, stream или consumer вне manifest/catalog;
- выдавать module wildcard `hermes.>`;
- хранить второй canonical event log внутри Event Hub;
- автоматически удалять orphaned stream/consumer/message;
- автоматически replay-ить DLQ или `unknown_outcome`;
- логировать payload, private identifiers, secrets или blob content;
- использовать Telemetry Hub или local memory как fallback data plane.

## Отклонённые варианты

### Event Hub как собственный message broker

Отклонено: дублирует JetStream, создаёт второй delivery protocol и новую
failure domain.

### Kernel proxy для каждого event

Отклонено: делает Kernel bottleneck и связывает доступность всех модулей с его
data-path latency.

### Динамические подписки без manifest

Отклонено: невозможно доказать authorization, resource budget и compatibility
до начала обработки.

### Полная история event payload в Event Hub

Отклонено: дублирует canonical/outbox/JetStream state и расширяет privacy
surface. Для диагностики достаточно bounded technical telemetry.

## Последствия

Положительные:

- все event contracts и subscriptions имеют единый проверяемый каталог;
- Kernel видит delivery health, не становясь data-plane proxy;
- restart NATS и модулей приводит к детерминированной reconciliation;
- lag, retry и DLQ становятся частью readiness и diagnostics;
- payload и business semantics остаются у владельцев contracts.

Отрицательные:

- manifests должны подробно описывать event topology и budgets;
- Event Hub требует NATS administration adapter и reconciliation tests;
- topology migrations становятся явными операциями;
- Kernel получает ещё одну критическую control-plane подсистему.

## Проверка решения

Architecture guard уже требует `event_hub` в составе единственного
`hermes-kernel` и запрещает отдельный Event Hub package. Это только статическая
предпосылка: Event Hub runtime и перечисленные ниже executable scenarios ещё
не реализованы.

До признания решения реализованным должны существовать tests:

- conflicting contract owners и schema versions fail closed;
- undeclared publisher/subscriber не получает NATS permission;
- missing required subscription блокирует readiness только её capability;
- optional subscription failure создаёт scoped degraded state;
- restart NATS восстанавливает declared consumers без удаления messages;
- лишний consumer не удаляется автоматически;
- lag, redelivery и DLQ видны без чтения payload;
- Event Hub отсутствует на normal publisher-to-consumer data path;
- NATS outage не создаёт in-memory fallback transport;
- diagnostics не содержат event payload, private content или secrets;
- Event Hub не создаёт canonical event history;
- destructive topology change требует отдельной authorization/migration path.

До появления executable evidence поле `Состояние реализации` остаётся
`Не реализовано`.
