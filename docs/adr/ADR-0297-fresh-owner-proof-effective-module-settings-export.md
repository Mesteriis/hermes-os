# ADR-0297: Fresh-owner-proof effective module Settings export

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Planned.

Уточняет:

- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0218: owner device identity](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0232: browser client identity](ADR-0232-browser-client-device-identity-and-same-origin-session.md);
- [ADR-0294: Mail account portability](ADR-0294-mail-account-credential-lifecycle-and-portability.md);
- [ADR-0296: owner module Settings through Core Gateway](ADR-0296-owner-module-settings-through-core-gateway.md).

## Контекст

ADR-0294 требует fresh owner proof перед экспортом non-secret Mail account
configuration. Обычный `ClientBootstrap` уже отдаёт authenticated sanitized
Settings projection, но cookie/session сам по себе не удовлетворяет этому
требованию.

Mail integration не может проверять browser device identity: authority и
device public key принадлежат Kernel. Core Gateway не может добавить
Mail-specific export RPC, потому что тогда Core начнёт импортировать
integration contract и интерпретировать provider configuration. Tauri bridge,
private owner-control socket и handwritten REST также запрещены.

## Решение

### Provider-neutral export operation

Существующий generated `OwnerModuleSettingsService` расширяется третьей
operation внутри той же Prepare/Commit ceremony:

```text
export_effective {
  registration_id
  expected_effective_revision
}
```

После fresh owner proof `Commit` возвращает:

```text
registration_id
schema_major
schema_revision
effective_revision
repeated typed visible values
```

Используется тот же закрытый public Settings value oneof из ADR-0296. Contract
не импортирует private runtime descriptor и не содержит provider identity,
arbitrary map, raw snapshot bytes, schema bytes, display metadata, credential
revision, secret reference или storage location.

### Kernel authority

`Prepare` и `Commit` используют существующие:

- authenticated same-origin non-LAN admission;
- active owner-device P-256 proof;
- exact request, session, owner, device, Control Store generation, identity
  epoch, target grant epoch и nonce binding;
- bounded volatile single-use challenge state.

После proof Kernel повторно проверяет approved registration и effective
GrantSet. Export разрешён только если:

1. Settings schema binding существует;
2. desired revision равна effective revision;
3. apply state равен `current`;
4. request revision равна effective revision;
5. schema artifact совпадает с admitted hash;
6. canonical snapshot target/revision и values проходят schema validation.

Kernel экспортирует только definitions с client visibility `editable` или
`read_only`. Hidden/system values, schema bytes и provider-specific
интерпретация не выходят в Gateway.

### Mail portability composition

Mail integration UI использует generated platform export как один input для
своего typed versioned `MailAccountExportV1`. First-party app дополнительно
запрашивает sanitized `MailAccountStatusV1` и собирает только:

- exact Mail Settings schema/revision;
- typed effective non-secret Mail values;
- connector profile и sanitized readiness;
- export contract major.

Импорт валидирует closed Mail export type и выполняет последовательность
ADR-0294:

```text
Settings update receipt
  -> sealed Vault provisioning receipt
  -> Mail binding/OAuth receipt
  -> Settings apply receipt
  -> Mail readiness
```

Каждый receipt хранится в UI workflow state отдельно. Ошибка не откатывает
предыдущие authority и не скрывает partial progress; продолжение начинается
только после fresh bootstrap/query reconciliation.

### Units and dependency direction

```text
Gateway Settings contract/router
  provider-neutral public wire and admission

Kernel Settings owner application
  proof, schema/hash/current-state validation and visible-value projection

Mail portability contract
  generated integration-owned typed export model only

Mail integration UI
  export/import orchestration and visible partial receipts

Vault / Mail runtime / Settings Registry
  independent authorities
```

Gateway и Kernel не импортируют Mail. Mail не импортирует Kernel
implementation и не читает Settings storage. App composition не становится
integration runtime или assembly.

## Phase gate

### `owner_module_settings_export_v1`

1. generated provider-neutral `export_effective` Prepare/Commit operation;
2. exact active-device fresh proof and single-use bounded challenge;
3. current desired/effective/apply-state and request-revision checks;
4. admitted schema hash and canonical snapshot validation;
5. visible typed values only, without raw/schema/secret carriers;
6. stale session/device/identity/control/grant/schema/revision negatives;
7. authenticated same-origin admission and LAN denial;
8. generated desktop client adapter;
9. architecture, SRP, Cargo, Clippy and workspace gates.

`mail_account_portability_v1` остаётся отдельным integration/app gate и не
открывается одним наличием generic export operation.

## Отклонённые варианты

### Использовать ClientBootstrap без fresh proof

Отклонено: authenticated session не является explicit export authorization.

### Mail-specific export в Core Gateway

Отклонено: Core начал бы импортировать integration contract и provider
semantics.

### Передать browser public key в Mail runtime

Отклонено: integration стала бы вторым identity authority и получила бы
Kernel-owned device state.

### Экспорт raw Settings snapshot или schema

Отклонено: это раскрывает internal descriptor surface, создаёт parser
duplication и позволяет hidden settings покинуть authority.

## Последствия

- fresh-proof export становится reusable platform operation без generic
  Settings read-all API;
- Mail portability получает честный generated input без Core/Mail coupling;
- обычный bootstrap остаётся sanitized navigation/read projection;
- наличие ADR не доказывает implementation или не открывает phase gate.
