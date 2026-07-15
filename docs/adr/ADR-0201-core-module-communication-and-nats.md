# ADR-0201: Взаимодействие ядра и модулей через IPC и NATS

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md).

Связано с:

- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md).

## Контекст

Модули Hermes работают в отдельных процессах. Система должна поддерживать:

- управление lifecycle независимо от data-plane инфраструктуры;
- быстрые typed query и request/reply operations;
- durable commands, events и provider observations;
- replay после остановки модуля или NATS;
- backpressure и bounded retry;
- отсутствие прямых module-to-module соединений;
- realtime integrations Mail, Telegram и Zulip без потери сообщений при
  локальном restart.

Один transport не удовлетворяет всем требованиям. Управление процессом не
может зависеть от NATS, а durable delivery не должна зависеть только от
состояния IPC-соединения.

## Решение

Взаимодействие разделяется на независимые control plane и data plane.

### Control plane

Control plane использует versioned Protobuf RPC поверх локального Unix socket.
Socket доступен только владельцу процесса и имеет mode `0600`; каталог runtime
имеет mode `0700`.

Минимальный protocol:

```text
Hello
Describe
Start
Quiesce
Drain
Stop
Health
ReloadConfiguration
RenewCapability
RevokeCapability
GetRuntimeState
```

Control plane обязан работать при недоступности NATS. Через него supervisor
должен иметь возможность диагностировать и остановить модуль даже при отказе
data plane.

Первичная аутентификация child process использует одноразовый bootstrap secret
через inherited pipe/file descriptor. Он не передаётся через argv, environment
или logs. После handshake применяется module identity и capability set из
manifest.

### Синхронный request/reply

Typed query и операции, которым необходим немедленный предметный результат,
используют versioned Protobuf request/reply через capability router ядра и
локальный IPC.

Правила:

- каждый запрос имеет request ID, deadline и cancellation;
- runtime unavailability возвращается как typed `ModuleUnavailable`;
- write request имеет idempotency key;
- автоматический retry write request разрешён только если contract явно
  объявлен idempotent;
- один contract выбирает один delivery mode и не отправляется одновременно по
  RPC и NATS;
- ядро маршрутизирует payload, но не интерпретирует business fields.

### Durable data plane

NATS JetStream принимается обязательным data-plane transport с первого
production walking skeleton.

JetStream используется для:

- durable cross-module commands;
- domain events;
- provider observations;
- asynchronous results и acknowledgements;
- workflow triggers;
- replay и controlled redelivery.

Core NATS без JetStream не используется для durable business messages.

PostgreSQL остаётся canonical source of truth для business state, event log,
outbox и consumer inbox. JetStream является delivery/fan-out transport, а не
единственным хранилищем события.

### Transactional outbox

Модуль фиксирует business mutation и outbox message в одной локальной
PostgreSQL-транзакции:

```text
BEGIN
  mutate module-owned state
  append outbox envelope
COMMIT
```

Outbox relay публикует сообщение в JetStream с `Nats-Msg-Id = message_id` и
запоминает publish acknowledgement. Если NATS недоступен, outbox остаётся
durable и повторяется после восстановления.

Потеря publish acknowledgement может привести к повторной публикации. Поэтому
application semantics остаётся **at least once**, даже если JetStream
поддерживает собственные механизмы дедупликации и double acknowledgement.
Hermes не заявляет end-to-end exactly-once.

### Inbox и acknowledgement

Durable consumer использует pull mode и explicit acknowledgement.

Consumer подтверждает сообщение в NATS только после атомарного завершения:

```text
BEGIN
  verify inbox deduplication
  apply local mutation
  store emitted outbox messages
  mark inbox message processed
COMMIT
ACK JetStream message
```

Повторная доставка того же `message_id` возвращает сохранённый processing
result или no-op и затем подтверждается. Дедупликация не зависит только от
ограниченного deduplication window JetStream.

### Типы сообщений

- `query` — синхронный read-only request/reply, не сохраняется в JetStream;
- `request` — синхронная операция с немедленным typed result;
- `command` — durable требование выполнить изменение;
- `event` — immutable факт, уже зафиксированный владельцем state;
- `observation` — факт внешнего наблюдения со stable provenance и cursor;
- `result` — завершение durable command;
- `ack` — подтверждение canonical persistence или terminal handling.

Событие нельзя использовать как скрытую команду. Command не может утверждать,
что изменение уже произошло.

### Envelope

Общий envelope содержит минимум:

- `message_id`;
- `message_kind`;
- `contract_name` и `contract_version`;
- `source_module`;
- `target_capability`, когда применимо;
- `partition_key`;
- `created_at` и `deadline`, когда применимо;
- `causation_id` и `correlation_id`;
- `idempotency_key`, когда применимо;
- `attempt`;
- `lease_epoch`, когда сообщение связано с fenced runtime;
- `actor` или system source;
- `trace_id` и bounded versioned trace context;
- `content_type`;
- versioned payload.

Private content, account names, email addresses, chat titles, message text и
секреты не помещаются в subject, headers, logs или health metadata.
Произвольный trace baggage и user-provided identifiers в trace context также
запрещены.

Полные message bodies, raw documents, media bytes, provider session material и
secret payloads не передаются через JetStream. Владелец сначала сохраняет их в
разрешённом module storage или blob service, после чего сообщение переносит
bounded metadata и opaque `BlobRef`/`EvidenceRef`. Для каждого contract задан
максимальный payload size; превышение fail closed и не превращается в large
NATS message.

### Subject convention

Subjects имеют versioned machine-readable grammar:

```text
hermes.command.v1.<owner>.<contract>
hermes.event.v1.<owner>.<contract>
hermes.observation.v1.<owner>.<contract>
hermes.result.v1.<owner>.<contract>
hermes.dead.v1.<owner>.<contract>
```

`owner` является владельцем versioned contract/capability, а не обязательным
источником наблюдения. Provider-specific operational message может принадлежать
integration-плагину; observation, пересекающее границу context domain,
принадлежит neutral evidence contract. Исходный provider сохраняется в
`source_module` и provenance envelope, но domain subscriptions и business
решения не ветвятся по provider.

`owner` и `contract` являются стабильными lowercase ASCII IDs. Account/entity
identity и partition key находятся в envelope, а не в subject. Произвольные
user-provided tokens в subjects запрещены.

Точные subjects принадлежат versioned contract и не создаются ad hoc в module
implementation.

### Streams и consumers

Начальная topology содержит отдельные bounded streams для commands, events,
observations, results и dead letters. Для каждого stream обязательны:

- максимальный размер;
- максимальный возраст;
- storage policy;
- replica count для выбранной local topology;
- publish timeout;
- alert threshold.

Для каждого durable consumer обязательны:

- explicit ack;
- bounded `MaxAckPending`;
- bounded `MaxDeliver`;
- backoff policy;
- processing deadline;
- lag и redelivery metrics;
- terminal dead-letter handling.

JetStream не считается автоматически реализующим application DLQ. После
исчерпания delivery budget core event runtime явно сохраняет sanitized failure
record и переносит envelope в dead-letter contract либо блокирует его для
operator review.

### Ordering

Глобальный порядок сообщений не обещается. Порядок гарантируется только внутри
явного `partition_key`, например aggregate ID или provider account ID.

Consumer не обрабатывает один partition параллельно. События разных partitions
могут обрабатываться одновременно.

### Retry и unknown outcome

- Retry policy принадлежит contract, а не вызывающему коду.
- Validation, authorization и protocol-version errors не повторяются.
- Transient infrastructure errors используют bounded backoff.
- После неоднозначного non-idempotent provider call результат становится
  `unknown_outcome`; автоматический повтор запрещён.
- Stale `lease_epoch` отклоняется до mutation и подтверждается как terminal
  fenced result.

### NATS permissions

Kernel Event Hub по ADR-0209 сверяет permissions и фактические subscriptions с
проверенным catalog/manifest, но не становится proxy для message payload.

Каждый module runtime получает отдельную NATS identity с allowlist на publish и
subscribe subjects из manifest. Wildcard-доступ ко всему `hermes.>` запрещён
для module runtimes.

Business/context domain не получает subscribe permission на provider-specific
operational contracts. Он может потреблять только neutral evidence или другие
явно разрешённые domain/workflow contracts.

Credential material передаётся через защищённый control plane и не сохраняется
модулем в business storage. NATS слушает только разрешённый local interface;
admin/monitoring endpoints не публикуются наружу.

### Недоступность NATS

При недоступности NATS:

- control plane и supervisor продолжают работать;
- синхронные local queries могут продолжать работать;
- новые durable messages остаются в PostgreSQL outbox;
- consumers переходят в degraded state;
- outbox имеет bounded backlog limits и blocker thresholds;
- сообщения не переключаются на другой незафиксированный transport.

## Запрещённые способы общения

- прямой socket от одного модуля к другому;
- shared in-memory event bus в production;
- чтение таблиц другого модуля как query API;
- запись в чужие tables как command;
- domain subscription на provider-specific operational contract;
- fire-and-forget Core NATS для durable facts;
- бесконечные retries;
- NATS payloads с blob bytes или secret material;
- полные private message/document bodies вместо bounded opaque reference;
- автоматическое изменение topology при ошибке.

## Последствия

Положительные:

- durable delivery и replay доступны с первого walking skeleton;
- control plane остаётся работоспособным при отказе broker;
- PostgreSQL transaction и NATS fan-out связаны через outbox;
- module contracts не зависят от конкретного NATS client;
- duplicate delivery является штатным и проверяемым сценарием.

Отрицательные:

- NATS становится обязательным локальным runtime component;
- необходимо сопровождать outbox relay, inbox deduplication и DLQ;
- нельзя считать успешную публикацию равной успешной обработке;
- ordering и retry policy должны проектироваться для каждого contract.

## Проверка решения

- protocol и payload version mismatch fail closed;
- duplicate publish и duplicate delivery не создают повторную mutation;
- crash до commit, после commit, до publish ack и до consumer ack;
- NATS outage сохраняет outbox и выполняет replay после reconnect;
- stale lease epoch rejected before mutation;
- consumer lag и `MaxAckPending` создают backpressure;
- retry exhaustion приводит к terminal dead-letter state;
- control plane выполняет `Drain` и `Stop` при остановленном NATS;
- subject permissions запрещают чужие publish/subscribe;
- subjects, headers, logs и health не содержат private content или secrets.

## Ссылки

- [NATS JetStream](https://docs.nats.io/nats-concepts/jetstream)
- [JetStream consumers](https://docs.nats.io/nats-concepts/jetstream/consumers)
- [JetStream model and deduplication](https://docs.nats.io/using-nats/developer/develop_jetstream/model_deep_dive)
