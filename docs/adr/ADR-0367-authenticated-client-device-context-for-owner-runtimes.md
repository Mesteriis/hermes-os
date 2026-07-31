# ADR-0367: Authenticated client device context for owner runtimes

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано в owner-neutral Gateway ClientRpc и
client_blob delivery envelope. Review task-candidate является первым owner,
который использует этот контекст как human decision evidence; его managed
runtime и live conformance остаются частью ADR-0366.

Уточняет:

- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0218](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0232](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0366](ADR-0366-communication-task-candidate-extraction-and-reviewed-task-promotion.md).

## Контекст

Core Gateway уже аутентифицирует browser session как пару logical owner и
device principal. Generic `ModuleClientRequestV1` передавал owner runtime только
`logical_owner_id`, поэтому runtime не мог отличить человеческое решение
конкретного отзываемого device от произвольного business payload.

Передавать device ID внутри generated Review request нельзя: клиентский payload
не является authentication authority. Выводить actor из owner ID также нельзя:
owner и device имеют разные lifecycle, key и revoke semantics.

## Решение

`ModuleClientRequestV1` получает owner-neutral поле
`authenticated_device_id`. Gateway заполняет его только из уже проверенной
browser session одновременно с `logical_owner_id` и до opaque routing в Kernel.

```text
authenticated browser session
  -> Gateway logical_owner_id + authenticated_device_id
  -> opaque ModuleClientRequestV1
  -> exact admitted owner runtime
```

Kernel проверяет route/grant/runtime fences и переносит opaque envelope без
интерпретации business payload. Runtime обязан использовать device context
только там, где его public contract требует human actor evidence.

Structural validation требует, чтобы owner и device context либо оба были
валидными bounded identifiers, либо оба отсутствовали у non-client internal
test/host envelopes. Клиент не может задать или переопределить device context.

## Границы и SRP

- Gateway session владеет authentication и выбором actor context;
- runtime protocol владеет только typed transport field и validation;
- Kernel остаётся owner-neutral capability router;
- Review владеет decision semantics и owner-local evidence;
- Tasks получает только typed approved-candidate event, а не Gateway session;
- domain, integration, workflow и engine packages не импортируются друг в
  друга из-за этого решения.

Поле не создаёт generic audit facade, не выдаёт owner rights, не расширяет
GrantSet и не заменяет durable causation/provenance.

## Failure semantics

- owner без authenticated device или device без owner отклоняется;
- malformed identifier отклоняется до runtime dispatch;
- stale/revoked browser session не создаёт request;
- runtime не принимает actor из generated client payload;
- internal non-client envelope не может притвориться authenticated request,
  заполнив только одну половину context.

## Phase gate

Решение считается реализованным после protocol positive/negative tests,
Gateway session propagation tests, Kernel envelope tests, formatting, Clippy и
workspace checks. Business использование и live proof проверяются gate
соответствующего owner, первым — ADR-0366.

## Отклонённые варианты

### Добавить device ID в Review request

Отклонено: untrusted клиент смог бы заявить произвольного actor.

### Хешировать logical owner как device evidence

Отклонено: это fake identity без device key и revoke lifecycle.

### Создать Review-specific Gateway API

Отклонено: Gateway стал бы business facade и нарушил owner-neutral boundary.
