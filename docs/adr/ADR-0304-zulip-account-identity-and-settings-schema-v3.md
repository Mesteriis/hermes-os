# ADR-0304: Zulip account identity and Settings schema v3

Статус: Принято
Дата: 2026-07-27
Состояние реализации: Реализовано. Zulip Settings schema major 3,
integration-owned account config, HTTP adapter, managed contour fixtures,
release assembly и frontend setup используют `account_email`. Bot-only active
field/UI отсутствуют; existing Vault credential binding сохранён.

Уточняет:

- [ADR-0204: integration/provider boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0222: Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0248: Zulip provider contract](ADR-0248-zulip-clean-room-provider-contract.md);
- [ADR-0291: Zulip account lifecycle](ADR-0291-zulip-account-history-query-and-replay-boundary.md);
- [ADR-0292: managed integration settings](ADR-0292-managed-integration-settings-apply-and-credential-binding.md).

## Контекст

Zulip API authentication использует realm URL, email identity и API key.
Identity может принадлежать разрешённой пользователю Zulip учётной записи или
боту. Текущий clean-room schema и UI называют любое подключение `bot_email` и
«Zulip bot», хотя runtime использует email как HTTP Basic identity и не имеет
bot-only validation или lifecycle.

Bot-only naming является ложным продуктовым ограничением. Простая замена UI
label оставила бы ошибку в typed settings и integration API.

## Решение

Zulip остаётся отдельной integration. Вводится Settings schema major 3:

```text
zulip.account_id
zulip.account_email
zulip.realm_url
```

Integration-owned `ZulipAccountConfig` использует `account_email`. HTTP adapter
передаёт его как Basic authentication username и сравнивает provider sender
identity без предположения, что account является bot.

`zulip.bot_email` не читается как silent alias. Переход major 2 → major 3 идёт
через normal supervised settings successor и explicit first-party setup.
Legacy field не переносится автоматически, потому что client должен
подтвердить exact account identity и credential authority.

API key остаётся только в Vault с exact `zulip.credentials.v1` lease. Email,
realm и local account ID не становятся Communications data или domain truth.

### Kernel agreement

Kernel:

- хранит typed desired/effective values по admitted schema hash;
- не интерпретирует Zulip email, realm или API key;
- не выбирает bot/user semantics;
- не импортирует Zulip packages;
- применяет schema major 3 только через generic Settings successor protocol.

Gateway переносит generic typed Settings mutation и Zulip-owned lifecycle/query
payload opaque. Communications не получает Zulip settings.

### Functional units

```text
zulip-api          account identity and validation
zulip-http         provider authentication adapter
zulip-runtime      schema v3 decode and lifecycle composition
zulip frontend     account setup and lifecycle UX
zulip assembly     immutable composition only
```

Assembly не владеет account semantics. App Settings только составляет panel.

## Gate `zulip_account_identity_v3`

Gate открывается только при наличии:

1. exact schema major 3 with `zulip.account_email`;
2. no active `zulip.bot_email` field or bot-only UI;
3. provider adapter using account email without bot assertion;
4. explicit no-alias validation for schema v2;
5. Vault API-key custody unchanged;
6. managed setup and runtime tests;
7. frontend type/unit/boundary evidence.

Gate не меняет Zulip provider permissions: фактический доступ по-прежнему
определяется выданным Zulip API key.
