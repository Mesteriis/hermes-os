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
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

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
ValidateSettings
ApplySettings
RenewCapability
RevokeCapability
GetRuntimeState
```

Control plane обязан работать при недоступности NATS. Через него supervisor
должен иметь возможность диагностировать и остановить модуль даже при отказе
data plane.

Kernel создаёт private inherited pipe/file descriptor и одноразовый challenge
managed child только после проверки approved `ManagedLaunchBinding` и exact
executable bytes по ADR-0219. External runtime использует registration proof
ADR-0215; этот proof подтверждает registration identity, но не publisher или
целостность binary. Bootstrap material не передаётся через argv, environment
или logs. `Describe` передаёт exact bounded `ModuleDescriptorV1` bytes; Kernel
сохраняет их и digest без reserialization. Managed runtime не получает data
plane, пока descriptor и optional settings schema digests не совпали с
`DistributionManifestV1` либо owner-pinned binding. После handshake effective
GrantSet вычисляется отдельно для каждой capability как пересечение descriptor
request, owner approval и hard Kernel policy ADR-0221/0222.

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

NATS JetStream принимается обязательным transport с первого production
walking skeleton, который открывает durable module data plane. Recovery-only
Kernel slice ADR-0225 запускается раньше него без NATS и оставляет
`nats_data_plane_v1` закрытым до отдельного package/configuration/security и
conformance gate.

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

Точный binary wire contract определён ADR-0220. Все durable message families
используют `DurableEnvelopeV1` из `hermes-events-protocol`:

```text
common immutable header
  + exactly one kind-specific metadata variant
  + catalog-bound opaque owner Protobuf payload
```

Отдельных stringly `message_kind` и `content_type` нет. Envelope major/revision,
owner contract major/revision и schema SHA-256 изменяются независимо. Command,
event, observation, terminal result и durable acknowledgment являются пятью
вариантами `oneof`; dead letter не является шестым business kind.

Producer сериализует envelope один раз при записи outbox. Relay публикует exact
stored bytes без decode/re-encode. Consumer проверяет header, subject, catalog,
schema hash и source fence до декодирования owner payload и mutation.

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
hermes.command.v1.<owner>.<contract>.v<contract-major>
hermes.event.v1.<owner>.<contract>.v<contract-major>
hermes.observation.v1.<owner>.<contract>.v<contract-major>
hermes.result.v1.<owner>.<contract>.v<contract-major>
hermes.ack.v1.<owner>.<contract>.v<contract-major>
hermes.dead.v1.<owner>.<contract>.v<contract-major>
```

Первый `v1` — transport/envelope subject grammar major, последний token —
major owner contract. Revision и schema hash находятся в envelope/catalog.

`owner` является владельцем versioned contract/capability, а не обязательным
источником наблюдения. Provider-specific operational message может принадлежать
integration-плагину; observation, пересекающее границу context domain,
принадлежит neutral evidence contract. Исходный provider сохраняется в
`source.module_id` и typed provenance payload, но domain subscriptions и business
решения не ветвятся по provider.

`owner` и `contract` являются стабильными lowercase ASCII IDs. Account/entity
identity и partition key находятся в envelope, а не в subject. Произвольные
user-provided tokens в subjects запрещены.

Точные subjects принадлежат versioned contract и не создаются ad hoc в module
implementation.

### Streams и consumers

Начальная topology содержит отдельные bounded streams для commands, events,
observations, terminal results, durable acknowledgements и dead letters. Для
каждого stream обязательны:

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
исчерпания delivery budget consuming owner adapter сохраняет exact original
bytes в bounded owner-scoped quarantine и публикует отдельный sanitized
`DeadLetterRecordV1`. Original envelope не переписывается, Event Hub не читает
payload, а replay требует explicit operator/owner operation. `unknown_outcome`
не является DLQ reason и автоматически не replay-ится.

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
- Stale typed source fence ADR-0220 или owner-specific lease epoch отклоняется
  до mutation и подтверждается как terminal fenced result/Ack согласно
  contract.

### NATS permissions

Kernel Event Hub по ADR-0209 сверяет permissions и фактические subscriptions с
проверенным catalog и effective GrantSet, но не становится proxy для message
payload.

Каждый module runtime получает отдельную NATS identity с allowlist на publish и
subscribe subjects из effective GrantSet. `ModuleDescriptorV1` задаёт только
requested contract references и не содержит NATS subjects; Event Hub выводит
точную allowlist из catalog. Wildcard-доступ ко всему `hermes.>` запрещён для
module runtimes.

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

- durable delivery и replay обязательны с первого `nats_data_plane_v1`
  walking skeleton; recovery-only slice их не имитирует;
- control plane остаётся работоспособным при отказе broker;
- PostgreSQL transaction и NATS fan-out связаны через outbox;
- module contracts не зависят от конкретного NATS client;
- duplicate delivery является штатным и проверяемым сценарием.

Отрицательные:

- NATS становится обязательным локальным runtime component после открытия
  `nats_data_plane_v1`, но не является boot dependency recovery-only Kernel;
- необходимо сопровождать outbox relay, inbox deduplication и DLQ;
- нельзя считать успешную публикацию равной успешной обработке;
- ordering и retry policy должны проектироваться для каждого contract.

## Проверка решения

- protocol и payload version mismatch fail closed;
- unknown envelope major/revision и schema hash fail closed;
- duplicate publish и duplicate delivery не создают повторную mutation;
- тот же `message_id` с другими bytes даёт collision quarantine без mutation;
- outbox-to-NATS publication сохраняет byte-for-byte identity;
- crash до commit, после commit, до publish ack и до consumer ack;
- NATS outage сохраняет outbox и выполняет replay после reconnect;
- stale lease epoch rejected before mutation;
- consumer lag и `MaxAckPending` создают backpressure;
- retry exhaustion приводит к terminal dead-letter state;
- control plane выполняет `Drain` и `Stop` при остановленном NATS;
- managed child не получает inherited control channel и не запускается при
  invalid/missing exact-byte binding;
- external registration proof не принимается как executable integrity proof;
- subject permissions запрещают чужие publish/subscribe;
- subjects, headers, logs и health не содержат private content или secrets.
- durable Ack-envelope не смешивается с JetStream ACK.

## Ссылки

- [NATS JetStream](https://docs.nats.io/nats-concepts/jetstream)
- [JetStream consumers](https://docs.nats.io/nats-concepts/jetstream/consumers)
- [JetStream model and deduplication](https://docs.nats.io/using-nats/developer/develop_jetstream/model_deep_dive)
