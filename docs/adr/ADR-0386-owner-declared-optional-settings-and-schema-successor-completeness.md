# ADR-0386: Owner-declared optional Settings и полнота schema successor

Статус: Принято
Дата: 2026-08-03
Состояние реализации: запланировано этим ADR; gate считается закрытым только
после protocol, Kernel, Mail и live recovery evidence из раздела «Проверка».

Уточняет:

- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0236: Integration owners, protocol adapters и configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0306: Repeatable development release refresh](ADR-0306-repeatable-development-release-refresh-and-successor-fencing.md);
- [ADR-0320: Mail multi-account configuration instances](ADR-0320-mail-multi-account-configuration-instances-and-runtime-multiplexing.md).

## Контекст

`SettingsSchemaV1` описывает тип, authority, scope, apply mode и default одного
поля, но прежний wire contract не различает безусловно обязательное значение и
значение, которое требуется только для выбранной provider-owned ветки.

Kernel поэтому определял полноту как равенство числа values числу definitions.
После добавления Mail address-book definitions это заблокировало все сохранённые
IMAP и Gmail configuration instances как `required_settings_missing`, хотя их
отсутствующие значения относятся к невыбранным transport/address-book веткам.
Provider snapshots и Vault bindings при этом сохранились.

Kernel не должен знать `imap`, `gmail`, `smtp`, `carddav` или другие provider
semantics. Integration runtime, в свою очередь, не должен владеть durable
Settings authority или обходить Kernel schema lifecycle.

## Решение

`SettingDefinitionV1` получает additive поле:

```proto
bool optional = 12;
```

Proto default `false` означает безусловно обязательное значение. Это сохраняет
fail-closed semantics и wire-совместимость всех существующих schemas: старые
artifacts и definitions без нового поля остаются required.

Kernel использует один platform-owned predicate полноты:

```text
snapshot complete = every definition where optional == false has a value
```

Он применяется одинаково при:

- materialization первого registration-scoped snapshot;
- создании configuration-instance target;
- projection schema successor во время managed release refresh.

`optional` не означает «не проверять». Snapshot по-прежнему проходит generic
structural/type validation. Integration runtime проверяет cross-field semantics
своего public schema: discriminator-selected значения могут быть обязательными,
а значения невыбранной ветки могут быть запрещены. Kernel не интерпретирует
provider identity или discriminator values.

Defaults materialize-ятся как раньше. Optional definition с default получает
явное значение; optional definition без default может отсутствовать. Required
definition без default оставляет target в `blocked_config` с sanitized reason
`required_settings_missing`.

Mail schema revision увеличивается внутри существующего major `2`. В ней:

- common account identity, inbound discriminator, SMTP discriminator и sync
  bounds остаются required;
- IMAP, Gmail, OAuth, SMTP endpoint и address-book branch values объявляются
  optional на platform completeness layer;
- Mail runtime остаётся единственной authority для точной проверки выбранной
  ветки.

Существующие desired snapshots проецируются без provider-specific conversion.
После projection полные targets переходят в `pending_validation` и проходят
обычный owner-authorized managed apply; secrets не копируются в Settings и
повторный legacy import не требуется.

## SRP и единицы сборки

- runtime protocol владеет declarative wire semantics;
- Kernel владеет generic completeness, durable revisions и apply state;
- integration владеет provider-specific conditional validation;
- assembly только связывает exact schema artifact с signed module release;
- Communications domain не импортирует Mail или Kernel implementation и не
  участвует в восстановлении provider configuration.

## Проверка

Gate требует:

1. protocol tests: required missing blocks, optional missing completes, default
   materialize-ится, sparse snapshot остаётся type-checked;
2. Kernel regression tests для initial target и schema successor;
3. Mail tests на exact required/optional inventory и обе transport branches;
4. architecture check, запрещающий provider-specific completeness в Kernel;
5. повторный `make dev` без reset Control Store/Vault/provider state;
6. read-only evidence, что два сохранённых Mail targets больше не имеют
   `required_settings_missing` из-за невыбранных branch fields;
7. browser evidence, что account selector получает восстановленный Mail catalog,
   а Gmail OAuth и Telegram QR остаются provider-owned workflows;
8. полный `make pre-push`.

## Последствия

- Additive schemas могут честно добавлять условные поля без блокировки unrelated
  configuration branches.
- Ошибка в owner schema всё ещё fail closed: новое поле required по умолчанию.
- Kernel не превращается в provider rules engine.
- Optionality является частью signed schema digest; изменение флага требует
  новой schema revision и explicit managed release refresh.
