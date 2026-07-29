# ADR-0325: Bounded asynchronous Mail sync execution

- Статус: принято
- Дата: 2026-07-29
- Состояние реализации: не реализовано. Решение принято до изменения
  provider/runtime/client contracts; gate остаётся закрытым до полного
  admission evidence из этого ADR.
- Связанные решения: ADR-0204, ADR-0205, ADR-0213, ADR-0214, ADR-0220,
  ADR-0239, ADR-0298, ADR-0299, ADR-0320

## Контекст

Live clean-room contour с восстановленным iCloud account доказал, что текущий
`mail.sync.v1` доходит до реального IMAP provider, но не является
работоспособным production contract:

- client RPC синхронно ждёт весь provider sync и блокирует Mail runtime loop;
- один `UID FETCH` запрашивает до всей настроенной sync window с
  `BODY.PEEK[]`, поэтому 1000 сообщений не укладываются в 10-секундный
  protocol-step deadline;
- timeout повторяет весь запрос до 255 раз, хотя один и тот же размер запроса
  остаётся заведомо неподходящим;
- заявленный ADR-0239 five-minute whole-sync deadline не применяется;
- client получает transport failure вместо быстрого durable acceptance и
  последующего terminal status;
- materialization начинается только после завершения всего adapter fetch, что
  увеличивает память, повторную работу и blast radius одного сбоя.

Уменьшить frontend timeout, убрать deadline или показать fake success нельзя.
Это скроет provider failure и сохранит blocking contract.

## Решение

### Ownership и согласование с Core

Изменение принадлежит Mail integration. Оно не добавляет Mail semantics в
Kernel, Gateway, Scheduler или Communications:

```text
first-party client
  -> Core Gateway opaque exact mail.sync.v1 request
  -> Mail runtime durable acceptance
  -> Mail-owned provider worker
  -> Mail persistence + exact outbox bytes
  -> NATS
  -> Communications ingress
```

Core Gateway по-прежнему проверяет session, route, grants, runtime generation,
deadline транспорта и переносит opaque Protobuf bytes. Его contract не ждёт
provider completion и не интерпретирует Mail result. Kernel не управляет
внутренней Mail queue и не получает credentials. Scheduler может в будущем
создать trigger, но не исполняет provider code.

Communications получает только ранее принятые provider-neutral observations
через durable events. Mail не вызывает Communications domain/runtime и не
читает его storage.

### Client command и terminal evidence

`mail.sync.v1/Sync` сохраняет exact operation ID и Mail-owned run до provider
I/O, после чего быстро возвращает:

```text
SyncInboxAcceptedV1 {
  operation_id
}
```

Acceptance означает только durable admission. Оно не означает, что provider
ответил, сообщения materialized или Communications обработал observations.

Terminal truth остаётся в отдельном `mail.sync.health.query.v1`:

- `GetRun(operation_id)` возвращает `RUNNING | SUCCEEDED | FAILED |
  INTERRUPTED`;
- `GetStatus(connection_id)` возвращает latest run и provider-path health;
- client наблюдает terminal state query/replayable owner realtime, но не держит
  command RPC открытым.

Повтор exact operation ID возвращает persisted acceptance или terminal run и
не создаёт второй provider execution. Другой connection с тем же operation ID
fail closed.

### Mail-owned execution state

Mail persistence является authority для accepted/running/terminal run.
Отдельная in-memory queue не является durable truth.

Runtime:

1. атомарно принимает run до provider I/O;
2. выбирает не более одного current run на configuration instance;
3. подготавливает provider operation с exact account/settings/runtime fence;
4. исполняет provider I/O вне client-control loop;
5. применяет завершённые pages в Mail-owned transaction;
6. фиксирует terminal run и sanitized failure code;
7. relays сохранённые exact observation bytes обычным outbox worker.

Restart помечает незавершённую работу predecessor generation как
`INTERRUPTED`. Successor не переиспользует process-bound credential material и
не продолжает старый worker. Повторный запуск требует нового operation ID либо
явного owner retry contract; automatic infinite resurrection запрещена.

### IMAP page и protocol bounds

Понятия разделяются:

- `mail.sync.window` — максимальное число сообщений в одном provider page;
- `mail.sync.windows` — максимальное число последовательных pages одного run;
- adapter fetch chunk — меньшая transport-oriented часть page.

Один IMAP `UID FETCH` содержит не более 25 UIDs. Chunk является protocol
adapter constant, а не public business setting. Один protocol step имеет
10-секундный deadline.

Один run:

- имеет hard total deadline 300 seconds;
- обрабатывает не больше `window * windows` messages;
- использует cursor/checkpoint для последовательных pages;
- после каждой успешно полученной page атомарно materializes provider rows и
  corresponding Communications outbox intents;
- не перечитывает уже committed page после transient retry внутри того же
  runtime generation.

Первый IMAP cutover сохраняет latest-first semantics существующего manual
backfill, но page/cursor calculation обязана быть deterministic и
test-covered. Provider cursor не входит в client contract.

### Retry classification

ADR-0239 limits `255 attempts` и `120 ms` заменяются этим решением.

- transient network/timeout failure: не более 3 attempts для текущего
  uncommitted protocol chunk;
- authentication, protocol rejection, invalid response и unsupported behavior:
  terminal без retry;
- retry никогда не продлевает total 300-second deadline;
- raw provider error остаётся developer diagnostic без credentials, message
  body или provider session data;
- public run получает только closed sanitized failure code.

Retry одного chunk не повторяет уже committed Mail materialization или outbox
intent. Idempotency Mail persistence остаётся последней защитой от redelivery.

### SRP и единицы сборки

- `hermes-mail-api` меняется с public command/status language;
- `hermes-mail-imap` меняется с IMAP paging, chunking, deadlines и provider
  error classification;
- `hermes-mail-persistence` меняется с durable run claim/checkpoint/terminal
  state;
- `hermes-mail-runtime` композирует provider worker и owner-local
  materialization, но не становится assembly;
- Mail release assembly только связывает уже admitted exact artifacts;
- frontend generated client, controller и presentation меняются отдельным
  Mail-owned slice после backend contract;
- Communications packages не меняются для provider execution и продолжают
  принимать только neutral event contract.

Integration не является domain. Runtime не является integration. Assembly не
реализует sync. Client composition не является provider authority.

## Admission evidence

Решение считается реализованным только атомарно с:

1. public wire regression, что `Sync` возвращает acceptance до provider
   completion, а terminal state читается только через sync-health contract;
2. persistence tests для duplicate/cross-account operation ID, single current
   run, predecessor interruption и terminal fencing;
3. deterministic IMAP tests для page ordering, maximum 25-UID fetch chunk,
   three-attempt transient retry, no-retry rejection и 300-second total
   deadline;
4. runtime test, что client/control loop продолжает обслуживать queries во
   время provider I/O;
5. PostgreSQL conformance, что каждая page materializes operational rows и
   exact Communications outbox intents atomically;
6. restart conformance без повторного использования credential lease;
7. generated frontend client и UI, которые показывают accepted/running/
   terminal state без fake completion;
8. live iCloud proof:
   `Sync accepted -> Mail projection -> neutral event -> Communications
   canonical evidence -> evidence export`;
9. architecture guards, что Mail/Communications/Core implementation imports не
   появились;
10. полный `make pre-push`.

## Последствия

Manual sync перестаёт быть длинным RPC и становится durable Mail operation.
Большой mailbox больше не превращается в один заведомо превышающий deadline
`UID FETCH`. Ошибка одного provider chunk имеет ограниченный retry blast
radius, а уже подтверждённые pages остаются materialized.

До завершения admission frontend не должен имитировать asynchronous success.
Старый synchronous response и параметры retry из ADR-0239 считаются
историческим состоянием реализации и заменяются только атомарным cutover этого
ADR.
