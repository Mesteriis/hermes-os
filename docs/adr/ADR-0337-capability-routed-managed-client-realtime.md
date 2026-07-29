# ADR-0337: Capability-routed managed client realtime

Статус: Принято

Дата: 2026-07-29

Состояние реализации: generic managed publication wire, descriptor/Control
Store admission, Kernel fence route, shared Gateway source и первый owner
adapter `communication_delivery_intent` реализованы. Owner adapter атомарно
сохраняет monotonic transition sequence рядом с state mutation, до `ready`
восстанавливает bounded replay window и после запуска публикует новые записи.
Managed development admission и live Gateway/SSE conformance ещё не доказаны,
поэтому phase gate остаётся `planned`.

Уточняет:

- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0251: opening client_gateway_v1](ADR-0251-client-gateway-v1-opening-for-owner-contracts.md);
- [ADR-0330: communication delivery intent](ADR-0330-provider-neutral-communication-delivery-intent-workflow.md).

## Контекст

`client_gateway_v1` реализует transport-level SSE, replay gap и revoke
semantics, но его `InMemoryBrowserRealtimeSource` не имеет production
publisher. Ни один independently managed module не может объявить client-safe
event, пройти capability admission и передать его Gateway. Поэтому status query
не закрывает terminal realtime, а polling не считается реализацией
`client_realtime`.

Прямой доступ Gateway к owner SQL, импорт owner API в Kernel/Gateway, передача
внутреннего `DurableEnvelopeV1` клиенту или отдельный SSE endpoint каждого
module запрещены.

## Решение

Вводится owner-neutral managed publication path:

```text
owner-local state transition + realtime outbox
  -> managed control request
  -> Kernel capability/runtime/grant fence
  -> shared Gateway realtime source
  -> one authenticated multiplexed SSE stream
```

### Descriptor и public event contract

`ModuleDescriptorV1` получает отдельный provided surface kind
`client_realtime`. Surface содержит exact `ContractReferenceV1` client-safe
event payload. Он не является `client_rpc`, durable publisher или implicit
permission на любой owner event.

Capability admission сохраняет exact:

- registration и capability;
- contract owner/name/major/revision/schema digest;
- current runtime generation и grant epoch через существующие bindings.

Новый surface не имеет отдельного HTTP path: все owner events мультиплексируются
в существующий `GET /api/realtime/v1/events`.

### Managed publication

Current managed runtime отправляет typed bounded request с:

- exact client-safe contract reference;
- logical human owner id;
- `ClientRealtimeEventV1`.

Kernel принимает publication только от current managed registration, если
exact capability granted и descriptor предоставляет exact
`client_realtime` contract. Kernel валидирует только generic frame bounds и
fences; owner payload остаётся opaque. Module id, registration, generation,
grant epoch, NATS metadata и provider cursor клиенту не выдаются.

Gateway realtime source динамически допускает logical owner только после
успешной Kernel authorization. Повтор exact cursor + exact bytes идемпотентен;
тот же cursor с другими bytes fail closed. Revoke закрывает owner stream и
следующая publication требует current grant/runtime binding.

### Durable replay принадлежит owner

Gateway history остаётся bounded delivery cache, а не canonical storage.
Каждый publisher атомарно сохраняет client-safe transition рядом со своей
state mutation и выдаёт monotonic owner-local cursor. Runtime:

1. до `ready` публикует bounded durable replay window из owner-local ledger;
2. после каждой mutation дренирует новые transition records;
3. продвигает process-local checkpoint только после положительного Kernel
   response;
4. безопасно повторяет publication после ambiguous failure, а после restart
   заново публикует bounded durable window.

После Kernel/Gateway restart managed runtime заново наполняет delivery cache из
durable ledger. Если requested cursor старше восстановленного bounded window,
Gateway отправляет explicit replay gap; silent reset запрещён.

Для `communication_delivery_intent` client event содержит только `intent_id`,
state, state revision, occurred time и sanitized rejection enum. Body,
provider/account identifiers, opaque provider cursors, credentials и internal
event envelope отсутствуют.

### Отказоустойчивость

- недоступный Gateway не откатывает уже принятую business mutation;
- unpublished transition остаётся retryable;
- invalid contract/frame или stale runtime/grant отклоняются без publication;
- duplicate delivery дедуплицируется по stable cursor/event id;
- публикация не является canonical business event и не заменяет provider
  result/outbox contracts ADR-0332;
- отсутствие admitted publisher оставляет SSE fail closed.

## Units и SRP

```text
runtime protocol
  typed publication wire and generic validation

Control Store / descriptor admission
  exact client_realtime surface catalog

Kernel realtime route
  capability and runtime fence authorization

Gateway realtime source
  bounded replay cache, live fan-out and gap semantics

owner persistence
  durable monotonic transition ledger

owner runtime adapter
  client-safe payload mapping and publication retry
```

Gateway не получает owner semantics, Kernel не становится event store, а
workflow persistence не получает transport/session responsibilities.

## Phase gate `capability_routed_managed_client_realtime_v1`

Gate считается реализованным только когда доказаны:

1. exact descriptor surface и Control Store persistence;
2. managed request/response validation и hard payload bounds;
3. capability, registration, runtime generation и grant epoch fencing;
4. owner isolation, revoke и zero/ambiguous route failure;
5. durable owner replay after Kernel/module restart;
6. duplicate/cursor conflict, bounded gap и live fan-out;
7. absence of private content/provider metadata in client frame;
8. one Gateway SSE stream without owner-specific Kernel/Gateway imports;
9. architecture, SRP, Cargo, Clippy and managed live conformance.

`communication_delivery_intent_v1` остаётся `planned`, пока его exact event
contract, durable ledger adapter и live Gateway proof не пройдут этот gate.

## Отклонённые варианты

### Polling GetStatus

Отклонено: query остаётся recovery/read path и не даёт ordered terminal
notification или reconnect replay.

### Gateway читает owner transition table

Отклонено: создаёт cross-owner SQL и переносит business schema в platform.

### Пробросить DurableEnvelopeV1 в SSE

Отклонено: внутренние fences, subject, provider metadata и payload не являются
client contract.

### Отдельный SSE endpoint workflow

Отклонено: client подключился бы к module runtime и получил второй transport,
обходящий Core Gateway session/capability policy.
