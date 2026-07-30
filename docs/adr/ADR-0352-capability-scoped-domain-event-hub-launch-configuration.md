# ADR-0352: Capability-scoped domain Event Hub launch configuration

Статус: Принято

Дата: 2026-07-30

Состояние реализации: implemented. Runtime protocol validation,
owner-control Kernel composition и eventless Review live conformance
реализованы.

Уточняет:

- [ADR-0201: Core/module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0215: open module registration and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0337: capability-routed managed client realtime](ADR-0337-capability-routed-managed-client-realtime.md);
- [ADR-0351: Review Communications attention](ADR-0351-review-communications-attention-owner-admission.md).

## Контекст

`ManagedDomainRuntimeConfigurationV1` исторически требовал NATS endpoint и
credential revision для каждого domain process. Это связывало даже domain,
который предоставляет только client RPC/realtime и Storage capability, с Event
Hub topology. Review attention является первым таким owner: его durable
realtime replay публикуется в общий client SSE transport через correlated
Kernel control IPC и не является domain-to-domain event.

Фиктивный endpoint, выдача credential без capability или добавление
неиспользуемого event grant нарушили бы exact admission.

## Решение

Event Hub launch configuration является согласованной парой:

```text
eventless: endpoint == "" and credential_revision == 0
event-backed: valid nats:// endpoint and credential_revision > 0
```

Смешанная пара fail-closed.

Kernel строит event-backed pair только если хотя бы одна capability текущего
approved GrantSet имеет сохранённый typed event route request. Если ни одна
approved capability не запрашивает event route, Kernel передаёт eventless pair
и не читает Event Hub topology.

Runtime configuration не выдаёт authority. Publish/subscribe authority
по-прежнему определяется descriptor request, owner-approved GrantSet, current
runtime/grant generations и Event Hub credential delivery. Eventless domain не
может получить credential через пустую конфигурацию.

## Границы

- Domain не выбирает наличие Event Hub через settings или environment.
- Client realtime не становится NATS event и идёт только через общий
  replayable SSE transport.
- Наличие Event Hub topology не расширяет capability domain автоматически.
- Отсутствие Event Hub topology не блокирует eventless domain.
- Existing event-backed domain configuration остаётся совместимой.

## Units и SRP

- runtime protocol валидирует только целостность pair;
- Control Store хранит descriptor requests и approved grants;
- Kernel owner-control composition вычисляет required topology;
- domain runtime использует только выданные ему transport contracts;
- Gateway/SSE не интерпретирует business payload.

## Проверяемый gate

1. eventless и event-backed configurations проходят validation;
2. partial pair отклоняется;
3. Kernel выводит необходимость Event Hub только из approved capability
   route requests;
4. event-backed Communications продолжает запускаться;
5. eventless Review запускается без Event Hub topology и публикует client
   realtime через shared SSE.
