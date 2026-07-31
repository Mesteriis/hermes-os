# ADR-0355: Capability-scoped Integration Event Hub launch configuration

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано. Managed Integration runtime protocol
принимает Event Hub configuration только как exact present pair
`endpoint + credential_revision` либо exact absent pair `empty + 0`.
Half-configured пары отклоняются. Ollama managed conformance доказывает
eventless Integration launch без фиктивного NATS grant. Eventful Mail,
Telegram, WhatsApp и Zulip contours сохраняют существующую credential-bound
конфигурацию.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0352](ADR-0352-capability-scoped-domain-event-hub-launch-configuration.md);
- [ADR-0353](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md).

## Контекст

ADR-0352 сделал Event Hub configuration capability-scoped для Domain runtime,
но Integration runtime protocol продолжал безусловно требовать NATS endpoint и
ненулевую credential revision.

Ollama integration реализует bounded synchronous `request_rpc`, owner-local
storage и loopback HTTP. В её descriptor нет event route capability. Выдача
фиктивного endpoint/revision:

- заявляла бы несуществующий grant;
- смешивала бы readiness с NATS;
- скрывала бы отсутствие event capability;
- создавала бы конфигурацию, которую Kernel не способен честно авторизовать.

## Решение

Managed Integration configuration использует exact pair:

```text
event route capability granted
  -> non-empty validated endpoint + credential_revision > 0

no event route capability
  -> empty endpoint + credential_revision = 0
```

Любая half-configured комбинация fail closed. Отсутствующий Event Hub не
ослабляет storage, settings, runtime artifact, configuration instance,
registration, generation или grant-epoch validation.

Integration runtime readiness означает готовность только descriptor-granted
capabilities. Eventless integration не получает credential, не соединяется с
NATS и не публикует события. Добавление event route в будущую revision требует
нового exact descriptor, approval, topology reconciliation и credential-bound
successor launch.

## Units и SRP

- runtime protocol валидирует структурную present/absent пару;
- Kernel формирует configuration по effective granted capabilities;
- Event Hub выдаёт credential только для reconciled event routes;
- integration реализует только собственные declared capabilities.

Ollama integration не получает пустой event adapter или fake outbox ради
прохождения общего validator.

## Phase gate

Gate реализован только при наличии:

1. protocol positive test для `empty + 0`;
2. negative tests для обеих half-configured пар;
3. сохранённых eventful integration tests;
4. live signed eventless Integration admission;
5. architecture, Cargo, Clippy и test gates.

## Отклонённые варианты

### Передать фиктивный loopback NATS endpoint

Фабрикует authority и делает health недостоверным.

### Добавить Ollama пустую event capability

Создаёт несуществующую responsibility и нарушает SRP.

### Сделать Event Hub обязательным для всех integrations

Связывает synchronous local provider ports с event infrastructure без
контрактной необходимости.
