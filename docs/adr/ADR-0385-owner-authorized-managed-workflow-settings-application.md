# ADR-0385: Owner-authorized managed workflow Settings application

Статус: Принято

Дата: 2026-08-03

Состояние реализации: planned. Решение должно быть реализовано до browser
Start/Get/shared-SSE gate `mail_contacts_sync_v1`; наличие ADR само по себе не
доказывает workflow reconfiguration или browser completion.

Уточняет:

- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0222](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0296](ADR-0296-owner-module-settings-through-core-gateway.md);
- [ADR-0380](ADR-0380-managed-workflow-configuration-instances-and-settings-bootstrap.md).

## Контекст

Kernel Settings Registry уже принимает owner-authorized desired Settings для
любого registered module и умеет запускать workflow с выбранным current
configuration instance. Публичный owner-proof apply contract, однако, существует
только как `ApplyOwnerManagedIntegrationSettingsV1` и запускает integration
configuration, integration state root и optional host bridge.

`mail_contacts_sync` является workflow. Использование integration apply для его
настроек смешало бы unit kind, runtime configuration и lifecycle authority.
Обновить desired revision без отдельного apply также недостаточно: effective
revision не меняется, а browser не получает доказательства, что successor
workflow действительно запущен с exact snapshot.

## Решение

### Отдельный public operation

`OwnerModuleSettingsService` получает отдельные additive Protobuf variants:

```text
ApplyOwnerManagedWorkflowSettingsV1
  registration_id
  storage_capability_id
  configuration_instance_id
  expected_desired_revision

ApplyOwnerManagedWorkflowSettingsReceiptV1
  registration_id
  configuration_instance_id
  effective_revision
  runtime_generation
  apply_state
```

Operation использует существующий two-phase `Prepare`/`Commit`, exact operation
ID, short-lived challenge, fresh device signature, owner/session binding,
Control Store generation и grant-epoch fencing. Integration operation остаётся
отдельным wire variant; generic `apply_module` с caller-selected kind не
вводится.

### Kernel authority и lifecycle

До mutation Kernel проверяет, что registration:

- approved и принадлежит authenticated logical owner;
- имеет exact effective grants и active Storage binding;
- descriptor-bound module kind равен `workflow`;
- имеет current schema и существующий configuration target;
- содержит exact desired revision в `pending_validation`;
- не запрашивает host bridge или integration state.

Apply выполняет один fenced successor transition:

1. валидирует desired snapshot против descriptor-bound Settings schema;
2. фиксирует `pending_apply`/`applying`;
3. fences predecessor runtime и Storage binding;
4. создаёт fresh runtime instance, generation, role epoch и credential lease;
5. строит `ManagedWorkflowRuntimeConfigurationV1` с выбранным target и bounded
   ordered catalog всех current configuration instances;
6. stages exact signed workflow executable/resources и settings bytes через
   private inherited control channel;
7. ждёт runtime-ready acknowledgement;
8. только после readiness повышает effective revision и возвращает receipt.

Launch failure переводит target в `blocked_config` с bounded sanitized reason и
не возвращает старый runtime к жизни автоматически. Retry требует нового
owner-proof operation и fresh successor identity.

### Границы

- Settings Registry владеет revisions и apply state, но не интерпретирует
  account ID, direction, interval или provider policy;
- workflow runtime декодирует и валидирует собственную typed schema;
- Mail integration остаётся единственным владельцем provider adapters,
  credentials и operational account truth;
- Contacts domain не получает Settings workflow или Mail;
- Core Gateway только аутентифицирует и переносит typed bytes;
- app frontend может композиционно связать Mail account projection и workflow
  Settings API, но Mail integration frontend не импортирует workflow code;
- workflow apply не является integration, domain или assembly unit.

## Browser gate

`mail_contacts_sync_v1` может быть открыт только когда реальный loopback browser:

1. создаёт/обновляет workflow configuration target через fresh owner proof;
2. получает workflow apply receipt с current effective revision;
3. открывает единственный shared replayable SSE до `Start`;
4. вызывает generated `Start` и затем generated `Get` через Core Gateway;
5. получает соответствующий typed status frame через общий SSE без polling;
6. не обращается напрямую к Mail, Contacts, NATS, PostgreSQL или module socket.

Static generated clients, mocked EventSource, integration-only apply и skeleton
UI не являются browser evidence.

## Единицы сборки и SRP

- gateway Protobuf package владеет только owner-facing transport shape;
- Kernel owner-settings handler владеет proof-bound operation dispatch;
- workflow lifecycle composer владеет только managed workflow launch;
- common Settings application helper владеет revision validation, successor
  storage fencing и readiness confirmation без выбора module kind;
- app composition владеет экраном, workflow frontend unit — Start/Get/SSE и
  typed settings semantics.

## Отклонённые варианты

### Запускать workflow через integration apply

Это передало бы workflow integration configuration/state/host-bridge semantics
и сделало бы declared module kind недостоверным.

### Менять desired Settings и считать reload завершённым

Desired revision не доказывает effective revision, fresh runtime identity или
runtime readiness.

### Добавить kind в generic apply request

Caller мог бы выбрать неверный launch path. Kind определяется только verified
registration/descriptor и отдельным exact operation variant.
