# ADR-0380: Managed workflow configuration instances and Settings bootstrap

Статус: Принято

Дата: 2026-08-02

Состояние реализации: статический contract slice implemented. Owner-neutral
runtime protocol, Kernel launch path и Mail-Contacts runtime используют exact
staged configuration-instance catalog; architecture и pre-commit gates пройдены.
Managed successor/revoke, multi-instance live conformance и полный
`mail_contacts_sync_v1` gate остаются planned. Наличие typed workflow Settings
schema без staged effective snapshot не является runtime evidence.

Уточняет:

- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0214](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0222](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0302](ADR-0302-bundled-managed-settings-and-runtime-bootstrap.md);
- [ADR-0379](ADR-0379-mail-address-book-sync-and-contacts-command-boundary.md).

## Контекст

`ManagedWorkflowRuntimeConfigurationV1` передаёт identity, Storage, Event Hub и
release artifacts, но не передаёт effective Settings identity или snapshot.
Это было достаточно для workflow с пустой либо compile-time конфигурацией, но
не для Mail address-book sync: направление, interval и account binding являются
typed workflow Settings. Environment/default fallback, чтение Kernel SQLite или
Mail storage и перенос provider configuration в workflow нарушают authority.

Одна registration должна обслуживать несколько независимо настроенных Mail
accounts. Один process на account конфликтует с bounded process budget и создаёт
лишнюю runtime authority. Registration-scoped singleton snapshot, наоборот,
теряет configuration-instance identity.

## Решение

Workflow получает owner-neutral configuration-instance bootstrap, аналогичный
по lifecycle integration instances, но без provider state root, credentials или
integration semantics.

`ManagedWorkflowRuntimeConfigurationV1` аддитивно получает:

```text
selected configuration_instance_id
selected settings_revision
ordered ManagedWorkflowConfigurationInstanceV1[]
  configuration_instance_id
  exact SettingsSnapshotV1 bytes
```

Kernel выбирает current/effective target только после owner-authorized start,
загружает все bounded current targets одной registration, валидирует schema,
target/revision и stable ordering и передаёт их через private staged runtime
configuration. Snapshot не передаётся через argv, environment, Event Hub,
Gateway или Storage. Secrets, cursors, checkpoints и last-run state в Settings
запрещены.

Старые workflow без configuration-instance bootstrap сохраняют пустые новые
fields. Наличие непустого selected ID требует непустой exact catalog и
`settings_revision` выбранного snapshot. Частично заполненная форма отклоняется.

Managed workflow runtime мультиплексирует bounded configuration instances в
одном process, но каждая client command и Scheduler job обязаны выбрать exact
instance/account. Workflow не выбирает provider: Mail integration разрешает
provider по своему account contract. Storage остаётся workflow-owned, а
business tenancy остаётся logical human owner.

## Scheduler binding

Scheduler регистрирует только workflow-owned JobKind. `scope_id` несёт opaque
configuration-instance ID. Due consumer проверяет current staged catalog,
lease, schedule revision и account binding, атомарно создаёт workflow run и
receipt outbox, затем подтверждает delivery. Disabled instance отклоняет новый
scheduled run без влияния на manual Start.

## Единицы сборки

- runtime protocol владеет только transport shape и validation;
- Kernel Settings/launch владеет selection и private staging;
- workflow runtime владеет typed decode и account/run semantics;
- Scheduler владеет time, lease и retry;
- Mail integration владеет provider identity и network protocol;
- assembly только связывает exact artifacts и opaque identities.

Ни одна из этих единиц не становится facade другой.

## Gate

Решение считается реализованным только после protocol compatibility tests,
Kernel selection/staging negatives, managed successor/revoke evidence и
Mail-Contacts multi-instance Scheduler/client conformance. До этого
`mail_contacts_sync_v1` остаётся `planned`.
