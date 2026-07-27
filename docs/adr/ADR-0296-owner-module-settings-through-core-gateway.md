# ADR-0296: Owner module Settings through Core Gateway

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Backend implemented; first-party client adapter pending.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0218: owner device identity](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0232: browser client identity](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0292: managed integration Settings apply](ADR-0292-managed-integration-settings-apply-and-credential-binding.md);
- [ADR-0294: Mail account portability](ADR-0294-mail-account-credential-lifecycle-and-portability.md).

Уточняется:

- [ADR-0297: fresh-owner-proof effective Settings export](ADR-0297-fresh-owner-proof-effective-module-settings-export.md).

## Контекст

Kernel уже владеет typed Settings Registry и provider-neutral managed
integration successor apply. Эти операции доступны только через private
owner-control Unix socket. First-party desktop client получает sanitized
Settings snapshot в bootstrap, но не имеет публичного generated пути для:

- записи следующей owner-editable desired revision;
- запуска generic managed integration Settings apply;
- получения exact sanitized receipt.

Frontend не может использовать private owner-control descriptor, Tauri
business bridge, handwritten REST или прямой Kernel socket. Integration UI не
может владеть Settings Registry и не должна превращать provider settings в
Communications command.

## Решение

### Public platform contract

Core Gateway публикует отдельный provider-neutral generated contract:

```text
OwnerModuleSettingsService
  Prepare
  Commit
```

`PrepareOwnerModuleSettingsRequestV1` содержит:

```text
operation_id = exact non-zero 16 bytes
operation =
  update_desired {
    registration_id
    expected_desired_revision
    repeated typed owner-editable values
  }
  | apply_managed_integration {
    registration_id
    storage_capability_id
    configuration_instance_id
    expected_desired_revision
    request_host_bridge
  }
```

Public settings values используют закрытый typed oneof:

```text
boolean
signed integer
unsigned integer
decimal string
string
duration millis
timestamp millis
enum
resource reference
```

Contract не импортирует private `owner_control.proto` или internal runtime
descriptor. Kernel преобразует public values в canonical `SettingsSnapshotV1`
только после fresh owner proof и затем применяет существующую schema
validation. Client не передаёт schema, display authority, apply mode,
credential revision, secret reference, runtime generation или provider
identity.

### Fresh owner proof

`Prepare` разрешён только authenticated same-origin browser session вне LAN
development mode. Kernel:

1. проверяет active owner device и принадлежность target registration;
2. связывает challenge с session, owner, device, exact request bytes,
   Control Store generation, identity epoch и random nonce;
3. сохраняет challenge только в bounded volatile state.

`Commit` принимает challenge ID и raw P-256 device signature. Challenge
single-use и short-lived. Перед mutation Kernel повторно проверяет principal,
Control Store generation, identity epoch, target registration и device state.
Cookie без fresh proof недостаточно.

### Mutation authority

Для `update_desired` Kernel:

1. строит canonical next revision из public typed values;
2. проверяет target, schema hash, value types, owner-editable visibility и CAS;
3. commit-ит desired revision в private Settings Registry;
4. возвращает только registration, desired revision и apply state.

Для `apply_managed_integration` Kernel использует уже принятый ADR-0292:

```text
validate exact desired revision
  -> durable successor reservation
  -> predecessor quiesce and fence
  -> managed integration launch
  -> readiness confirmation
  -> effective revision receipt
```

Public contract не декодирует provider configuration. Core Gateway переносит
только typed platform request, а integration runtime получает canonical
snapshot через existing managed launch configuration.

### Failure and reconciliation

- stale session/device/identity epoch: permission denied;
- changed Control Store generation or consumed challenge: conflict/not found;
- invalid schema/value/CAS: rejected without apply;
- failed successor launch: existing ADR-0292 blocked state remains authority;
- Gateway response loss: client читает fresh bootstrap and integration
  readiness before deciding whether to issue a new explicit operation;
- Kernel restart invalidates pending challenges but не откатывает committed
  desired/effective Settings state;
- automatic retry, hidden multi-owner transaction и success inference from
  accepted transport запрещены.

`operation_id` связывает UI receipt и challenge, но не является разрешением на
silent retry. Durable Settings revisions and apply state являются authority.

### Units and dependency direction

```text
Gateway Settings contract/router
  public wire, session/origin admission, bounded transport

Kernel owner-device proof
  shared active-device verification and P-256 challenge proof

Kernel Settings owner application
  public-to-canonical mapping, schema/CAS validation and ADR-0292 apply

integration-owned Settings UI
  provider form and workflow composition only

integration runtime
  canonical settings decoding and provider behavior

release assembly
  immutable artifacts only
```

Gateway не зависит от Mail/Telegram/WhatsApp/Zulip. Kernel Settings owner
application не импортирует integration packages. Integration UI использует
generated platform contract, но не private Kernel protocol. Assembly не
реализует mutation или provider semantics.

## Phase gate

### `owner_module_settings_gateway_v1`

1. public generated Prepare/Commit contract without internal descriptor import;
2. authenticated same-origin non-development admission;
3. exact operation-bound fresh active-device P-256 proof;
4. typed public-to-canonical Settings value mapping;
5. schema, owner-editable and desired-revision CAS negatives;
6. provider-neutral managed integration successor apply;
7. stale session/device/identity/control/runtime/grant/storage negatives;
8. sanitized receipts and negative-output privacy;
9. restart/response-loss reconciliation through canonical bootstrap state;
10. architecture, SRP, Cargo, Clippy and workspace gates.

## Отклонённые варианты

### Public owner-control socket

Отклонено: private Kernel control descriptor не является client API.

### Tauri Settings business commands

Отклонено: host bridge стал бы вторым authority и desktop-only API.

### Provider-specific Settings RPC в Gateway

Отклонено: Core начал бы импортировать integrations и интерпретировать
provider configuration.

### Credential revisions в Settings payload

Отклонено: credential binding принадлежит integration persistence и Vault
lifecycle, а не Settings Registry.

## Последствия

- desktop и будущий Android используют один generated client-neutral contract;
- Mail portability может честно собрать Settings update, sealed credential
  provisioning, Mail bind, generic apply и readiness query;
- provider Settings остаются integration-owned UI, но authority остаётся в
  Kernel Settings Registry;
- наличие ADR не открывает gate до реализации всех evidence выше.
