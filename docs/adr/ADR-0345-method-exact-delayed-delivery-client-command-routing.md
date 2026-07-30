# ADR-0345: Method-exact delayed-delivery client command routing

Статус: Принято

Дата: 2026-07-30

Состояние реализации: реализовано в public API и runtime contract unit.
Executable admission и live Core Gateway routing остаются частью незавершённого
gate `communication_delayed_delivery_v1`.

Уточняет:

- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0341](ADR-0341-scheduled-communication-delivery-workflow.md).

## Контекст

Generated Connect service delayed delivery содержит два command RPC:
`Schedule` и `Cancel`. Внутренний `ModuleClientRequestV1` переносит exact
`ContractReferenceV1`, но не HTTP/Connect method path. Один общий command
contract поэтому не даёт runtime надёжного discriminator между двумя protobuf
payload. Угадывание команды по полям protobuf запрещено: неизвестные поля и
совпадающие field numbers не образуют fail-closed routing contract.

## Решение

Каждый command RPC получает отдельный public contract:

```text
Schedule -> communication.delayed_delivery.schedule@1
Cancel   -> communication.delayed_delivery.cancel@1
Status   -> communication.delayed_delivery.query@1
SSE      -> communication.delayed_delivery.status_changed@1
```

Core Gateway выбирает command contract по принятому generated Connect route до
создания `ModuleClientRequestV1`. Runtime сравнивает весь exact
`ContractReferenceV1` и декодирует только соответствующий payload. Неизвестный
contract отклоняется без попытки protobuf autodetection.

Realtime остаётся одним provider-neutral replayable SSE contract. Разделение
command contracts не создаёт второй stream и не вводит polling.

## Последствия

- Schedule и Cancel нельзя перепутать на module boundary.
- Client transport остаётся generated и method-exact.
- Runtime не получает HTTP details и не становится facade.
- Admission обязан предоставить оба command contracts как независимые
  capability routes.
