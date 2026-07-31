# ADR-0358: Capability-scoped Engine Event Hub launch configuration

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано. Runtime protocol принимает только exact
present/absent Event Hub pair, Kernel выводит необходимость topology из
approved capability route requests, а signed managed AI inference conformance
доказывает eventless launch без фиктивного NATS grant. Event-backed Attachment
Security Engine сохраняет существующую конфигурацию.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0352](ADR-0352-capability-scoped-domain-event-hub-launch-configuration.md);
- [ADR-0353](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md);
- [ADR-0355](ADR-0355-capability-scoped-integration-event-hub-launch-configuration.md).

## Контекст

`ManagedEngineRuntimeConfigurationV1` и Kernel engine launch path
безусловно требуют Event Hub endpoint и credential revision. Это связывает
любой Engine process с NATS даже тогда, когда его approved capabilities
содержат только Storage, Blob и synchronous `request_rpc`.

AI inference engine ADR-0353 является первым таким Engine: workflow вызывает
его exact typed request port, а engine читает target-bound Blob и делегирует
provider request отдельной Ollama integration. В descriptor AI engine нет
durable event route. Фиктивный endpoint, credential или пустая event
capability фабриковали бы authority и нарушали бы exact admission и SRP.

## Решение

Managed Engine configuration использует согласованную capability-scoped пару:

```text
approved event route exists
  -> validated nats:// endpoint + credential_revision > 0

approved event route absent
  -> empty endpoint + credential_revision = 0
```

Любая half-configured пара отклоняется. Kernel определяет необходимость Event
Hub только по typed event route requests capabilities из текущего effective
GrantSet. Если таких routes нет, Kernel не читает Event Hub topology и не
выдаёт credential.

Runtime configuration остаётся transport configuration, а не источником
authority. Добавление event route в следующую engine revision требует нового
descriptor, owner approval, topology reconciliation и successor launch с
актуальными runtime/grant/credential generations.

## Границы

- Engine не выбирает Event Hub через settings, environment или provider
  response.
- Synchronous `request_rpc` не превращается в durable event.
- Blob custody, Storage binding, settings revision и provider dependency не
  ослабляются для eventless engine.
- Отсутствие Event Hub topology не блокирует engine без approved event route.
- Event-backed Attachment Security Engine сохраняет существующую
  credential-bound конфигурацию.
- AI inference engine не получает пустой event adapter, fake outbox или
  неиспользуемую capability.

## Units и SRP

- runtime protocol валидирует только exact present/absent pair;
- Control Store хранит descriptor route requests и effective GrantSet;
- Kernel owner-control composition выводит требуемую topology;
- Event Hub выдаёт credential только для reconciled approved routes;
- каждый Engine реализует только свои declared responsibilities.

Engine остаётся отдельной единицей сборки и runtime failure boundary. Это
решение не объединяет AI engine, Ollama integration, Communications domain или
reply workflow.

## Phase gate

Решение считается реализованным только при наличии:

1. protocol positive tests для event-backed и `empty + 0`;
2. negative tests для обеих half-configured пар;
3. Kernel descriptor/grant-driven configuration без unconditional topology
   lookup;
4. сохранённого event-backed Attachment Security launch;
5. signed eventless AI engine admission с настоящими Vault/Storage/Blob;
6. restart/idempotency/privacy conformance без фиктивного NATS grant;
7. architecture, Cargo, Clippy и test gates.

## Отклонённые варианты

### Добавить AI engine пустую event capability

Создаёт несуществующую ответственность и выдаёт лишнюю authority.

### Передать фиктивный NATS endpoint

Делает runtime configuration и health недостоверными.

### Считать любой Engine event-backed

Смешивает module kind с capability. Engine обозначает ответственность и
failure boundary, а не обязательный transport.
