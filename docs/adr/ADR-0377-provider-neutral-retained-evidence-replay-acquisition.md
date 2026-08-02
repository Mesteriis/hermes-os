# ADR-0377: Provider-neutral retained-evidence replay acquisition

Статус: Принято

Дата: 2026-08-02

Состояние реализации: planned. Решение уточняет ещё не admitted
`attachment_preview_retained_evidence_replay_v1` gate до browser cutover.
Существующий managed backend contour доказал exact-byte replay, но live browser
не может завершить исторический Preview: frontend не имеет generated replay
client, а public Start request требует producer registration, runtime/grant
fences и owner-local outbox message IDs, которых provider-neutral client не
владеет и не должен получать.

Зависит от:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0373](ADR-0373-bounded-attachment-preview-workflow.md);
- [ADR-0376](ADR-0376-owner-authorized-retained-durable-evidence-replay.md).

## Контекст

ADR-0376 правильно закрепил producer-local exact-byte authority, append-only
audit и отдельный replay workflow. Его первый managed request использовал exact
producer selections, переданные conformance harness:

```text
producer registration
+ runtime generation
+ grant epoch
+ original durable message IDs
```

Эти поля полезны как managed negative evidence, но не образуют допустимый
first-party browser contract. Communications UI знает только canonical
`attachment_anchor_id`. Он не читает Mail storage, Kernel registry или Event
Hub topology и не должен раскрывать пользователю integration registration либо
internal durable message identity.

Live browser доказал конкретный gap: generated Preview Start получил
`accepted`, один shared SSE остался подключён, но terminal state не наступил,
потому что frontend не мог легально начать replay. Добавлять polling,
handwritten REST, SQL lookup или frontend-доступ к Kernel registry запрещено.

## Решение

Публичный replay Start до admission заменяется provider-neutral revision:

```text
authenticated owner/device
+ operation_id
+ attachment_anchor_id
```

Client не передаёт owner claims, provider identity, registration, subject,
runtime/grant fence, contract reference или message IDs. Owner/device выводятся
Core Gateway из `ModuleClientRequestV1`, как и для остальных owner runtimes.

После Start отдельный replay workflow публикует два typed durable commands:

```text
attachment_preview_evidence_replay workflow
├─ event → Communications replay adapter
└─ event → Mail replay adapter
```

Каждый command содержит operation/actor/anchor, но не готовую selection.
Получив command через собственный current Event Hub route, producer:

1. проверяет logical owner и authenticated actor binding;
2. owner-locally находит exact replay-index row по `attachment_anchor_id`;
3. сверяет indexed contract/schema/hash с original outbox bytes;
4. атомарно фиксирует append-only audit с выбранным exact `message_id`;
5. публикует original bytes без decode/re-encode и без изменения source outbox;
6. возвращает causal typed result с bounded message IDs и sanitized outcome.

Таким образом owner authorizes одну bounded use-case operation по canonical
anchor, а exact selection остаётся в authority producer, который способен её
проверить. Отсутствие index row, stale route, owner mismatch, hash/contract
drift или outage дают terminal sanitized result. Другой owner не читает этот
index и не выбирает provider behavior.

Существующий client-supplied selection contract не является admitted public
compatibility surface. До gate admission он заменяется атомарно вместе с API
schema digest, workflow core/persistence/runtime, двумя producer contracts,
managed fixtures и generated frontend client. Двойной V1/V2 facade не
сохраняется.

## Kernel и Core agreement

Новых Kernel business semantics нет:

- desktop/browser продолжает обращаться только к Core Gateway;
- Gateway маршрутизирует exact generated client payload по descriptor и не
  интерпретирует anchor или replay outcome;
- Event Hub выдаёт обычные exact publish/consume permits и не читает payload;
- Kernel registry, Control Store и module inventory не становятся replay
  selector;
- stale/revoked runtime отсекается существующим registration/generation/grant
  route fencing;
- workflow и producers остаются отдельными managed OS-процессами;
- Communications domain не импортирует Mail integration, а Mail не импортирует
  Communications implementation.

Integration общается не с domain implementation и не через domain facade. Mail
получает только свой exact durable command через Core Event Hub и возвращает
свой exact durable result. Communications получает отдельный Communications-
owned command тем же способом.

## Frontend flow

Application composition запускает Preview и replay как одну bounded browser
operation:

```text
select canonical safe attachment
↓
generated Preview Start
↓
generated retained-evidence replay Start
↓
one existing replayable SSE hub
↓
Preview awaiting evidence → rendering → terminal
↓
generated IssueRead + exact client_blob
```

Replay Start idempotent по operation ID. Повтор для already-consumed evidence
является producer-local no-op. Frontend не создаёт interval/timeout polling,
не получает private bytes через query/SSE и не показывает provider internals.
Если replay capability не admitted, UI сохраняет reference-compatible skeleton
и bounded unavailable state вместо вечного spinner.

## Build units и SRP

Functional ownership остаётся раздельным:

- replay API — только generated client contract;
- replay core — только deterministic two-producer coordination;
- replay persistence — operation/result/command replay, без foreign SQL;
- replay runtime — Core Gateway/Event Hub composition;
- Communications replay persistence — только Communications index/audit/outbox;
- Mail replay persistence — только Mail index/audit/outbox;
- два producer contracts — разные schemas/routes;
- frontend generated adapter — transport mapping;
- frontend controller — Preview/replay/SSE lifecycle;
- presentation — только bounded state/artifact rendering.

Количество строк не определяет границу. Причина изменения и authority
определяют единицу сборки.

## Phase gate

Решение считается реализованным только после:

1. provider-neutral public request без producer/fence/message-ID fields;
2. two exact owner-specific anchor replay commands/results;
3. producer-local deterministic index selection и append-only selected-ID audit;
4. exact original byte/hash/contract verification before publish;
5. owner, route generation/grant, duplicate and operation-conflict fencing;
6. missing index, wrong owner, stale route, partial producer and outage/restart
   managed evidence;
7. canonical generator создаёт frontend schema/client, generated files не
   редактируются вручную;
8. frontend не содержит polling, handwritten business REST или provider
   selection logic;
9. live historical safe attachment достигает terminal Preview через shared SSE
   и читается только через `client_blob`;
10. architecture, SRP, Cargo, unit, managed, frontend и full pre-push gates.

До выполнения gate `attachment_preview_retained_evidence_replay_v1` и parent
`attachment_preview_v1` остаются `planned`.

## Отклонённые варианты

### Читать producer selection из Kernel registry во frontend

Отклонено: registry не знает owner-local outbox identity, а client получает
internal topology и начинает выбирать provider route.

### Добавить workflow SQL join к Communications и Mail

Отклонено: workflow превращается в cross-owner storage facade.

### Передать selection через Communications query

Отклонено: domain начинает раскрывать integration identity и authorizes чужой
outbox.

### Ждать Preview через polling

Отклонено: terminal lifecycle уже принадлежит одному replayable SSE stream.

### Оставить managed-only public request

Отклонено: conformance harness не является first-party browser contract и не
закрывает пользовательский flow.
