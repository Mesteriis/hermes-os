# ADR-0376: Owner-authorized replay of retained durable evidence

Статус: Принято

Дата: 2026-08-02

Состояние реализации: protocol and Communications owner-local persistence
foundations implemented, gate planned.
Диагностический browser gate Preview подтвердил, что generated Start/Get
проходят через Core Gateway и один shared SSE stream, но source events старше
bounded JetStream retention уже отсутствуют в broker. Отдельный workflow-owned
build unit `hermes-retained-evidence-replay-protocol` реализует bounded exact
message selection, producer registration, owner-device actor hash,
runtime/grant fences и sanitized terminal result без subject/query/payload
surface. Communications build unit
`hermes-communications-retained-evidence-replay-persistence` добавляет exact
index поверх собственного `communications_domain_outbox`, проверяет сохранённые
bytes/message/hash/contract и ведёт append-only replay audit как storage bundle
revision 17. Mail owner-local selection/audit adapter, workflow runtime и live
conformance ещё не реализованы. Никакая SQL-правка publish state не считается
реализацией этого решения.

## Контекст

JetStream является bounded transport, а не canonical archive. Новый consumer,
admitted после `max_age`, не может восстановить историческое evidence только из
broker. В Preview это проявилось на уже импортированных вложениях: canonical
projection и producer-owned durable evidence существуют, но Preview inbox пуст,
поэтому workflow остаётся в accepted state и не имеет права запрашивать bytes.

Автоматически увеличивать retention, читать чужие PostgreSQL tables из Preview,
сбрасывать `published_at` или строить Kernel replay facade нельзя. Эти варианты
нарушают owner authority, bounded resource policy либо exact-byte provenance.
ADR-0201 и ADR-0220 уже требуют explicit operator/owner operation для replay.

## Решение

Ввести platform-neutral replay protocol и отдельные owner-local adapters.
Replay не является domain, integration или Kernel capability:

- owner/device начинает typed operation через Core Gateway;
- operation выбирает exact producer registration, contract reference и bounded
  set canonical `message_id`; arbitrary subject, SQL predicate и read-all scope
  запрещены;
- Kernel только проверяет обычный route/grant/runtime fence и не читает payload;
- producer adapter сверяет owner-local outbox indexes, envelope SHA-256 и exact
  canonical bytes, затем публикует тот же byte buffer и тот же `message_id`;
- publish получает отдельный owner-local replay attempt/audit record. Canonical
  outbox row и исходный publish acknowledgement не переписываются;
- consumer применяет существующий inbox ID/hash contract. Уже обработанный факт
  становится no-op, новый consumer фиксирует его впервые;
- replay не меняет business truth и не создаёт provider command;
- если exact original bytes отсутствуют или contract требует transformed
  payload, replay запрещён. Migration создаёт новый typed envelope с новым
  `message_id` и causation на original message в отдельном ADR;
- secrets, provider session state, private body/blob bytes и raw payload не
  возвращаются в client response, logs, health, telemetry или audit summary.

Один общий replay service с доступом ко всем owner outbox запрещён. Каждый
producer сохраняет SRP: selection/authorization, exact-byte verification и
publish-attempt persistence являются отдельными owner-local build units либо
явно разделёнными components внутри уже admitted producer runtime. Integration
не становится domain, domain не импортирует integration, а replay между ними не
вводит direct call.

## Preview recovery slice

Для исторического Preview требуется отдельный
`attachment_preview_retained_evidence_replay_v1` gate:

1. Communications owner выбирает exact safety-event message для запрошенного
   attachment anchor без выдачи provider identity;
2. producer integration, владеющий exact scan-candidate outbox bytes, выбирает
   соответствующее observation по собственному owner-local index;
3. owner подтверждает оба bounded replay attempts одной use-case operation;
4. оба producer adapters публикуют только original exact bytes;
5. Preview получает facts обычными durable consumers, выполняет существующий
   order-independent join и продолжает custody/render/SSE flow;
6. отсутствие одного producer, original bytes либо owner proof даёт terminal
   sanitized unavailable result, а не бесконечный spinner и не fallback к
   чужому storage.

Координация exact producer operations принадлежит отдельному workflow. Он
хранит только operation/correlation state, не читает owner storage и не
импортирует domain/integration implementations. Его target commands и results
идут через durable events; client polling не вводится.

## Phase gate

Gate становится implemented только после:

1. versioned replay operation/result contracts и отдельной workflow assembly;
2. owner/device, registration, runtime generation и grant epoch fencing;
3. bounded exact message selection без arbitrary subject/query;
4. byte/hash/index verification и publish без decode/re-encode;
5. append-only replay audit и idempotent attempt replay;
6. new-consumer recovery и already-consumed no-op evidence;
7. expired retention, missing bytes, stale fence, wrong owner, partial producer,
   NATS outage/restart и privacy-negative conformance;
8. live Preview terminal SSE/client_blob browser proof без polling;
9. architecture, SRP, Cargo, frontend и full pre-push gates.

До выполнения gate ADR-0373 и inventory `attachment_preview_v1` остаются
`planned`.

## Отклонённые варианты

### Увеличить JetStream retention до бесконечности

Отклонено: broker не становится canonical archive и теряет bounded disk policy.

### Сбросить `published_at` в producer outbox

Отклонено: mutation скрывает replay attempt, меняет delivery evidence и требует
direct owner-storage intervention.

### Дать Preview доступ к Communications, integration или Attachment Security SQL

Отклонено: нарушает owner isolation и превращает workflow в facade.

### Реализовать replay в Kernel/Event Hub

Отклонено: Kernel не знает business selection и не получает generic payload или
owner-outbox authority.
