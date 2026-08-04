# ADR-0310: Telegram user-only TDLib QR account identity

Статус: Принято
Дата: 2026-07-28
Состояние реализации: Implemented. Bot identity/token удалены из active Rust,
Protobuf и frontend contracts; Telegram client contract revision повышена до
6, client не может утверждать QR authorization, а backend требует exact API
hash/session-key bindings. Rust, generated frontend, unit, architecture и
loopback browser gates зелёные. Реальный QR scan не выполнялся без
пользовательских Telegram API ID/hash и не заявляется как evidence.
Authorization transport extension переводится с polling на общий replayable
Gateway SSE: code/unit evidence готово, повторный managed browser gate ещё не
заявлен.

Уточняет:

- [ADR-0204: integration/provider boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0236: integration configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0240: Telegram clean-room boundary](ADR-0240-telegram-clean-room-provider-boundary.md);
- [ADR-0292: managed integration settings apply](ADR-0292-managed-integration-settings-apply-and-credential-binding.md);
- [ADR-0295: Owner write-only Vault provisioning](ADR-0295-owner-write-only-vault-provisioning-through-core-gateway.md);
- [ADR-0303: provider-owned QR account linking](ADR-0303-provider-owned-qr-account-linking-and-transient-artifact-custody.md);
- [ADR-0309: loopback browser Owner Vault host](ADR-0309-loopback-browser-owner-vault-provisioning-host.md).

## Контекст

Clean-room Telegram runtime использует TDLib user authorization и уже
отклоняет bot account при bootstrap. Однако public Rust contract всё ещё
содержит `TelegramProviderKind::Bot` и credential purpose
`telegram_bot_token`, а client Protobuf переносит динамический
`provider_kind`.

Эта модель:

- обещает неподдерживаемый bot lifecycle;
- позволяет frontend или старому клиенту прислать identity, которую runtime
  всё равно отвергнет;
- смешивает выбор provider principal с Telegram user account setup;
- мешает единственному принятому onboarding flow: API ID/hash, TDLib session и
  provider-issued QR.

Bot API является отдельным Telegram protocol/product surface с другими
credential, lifecycle и capability semantics. Он не может быть скрытым
вариантом TDLib user account и не входит в текущий Communications migration.

## Решение

Active Telegram integration поддерживает только Telegram user account через
TDLib:

```text
owner enters Telegram API ID/hash
  -> Owner Vault stores API hash and session-store key
  -> managed Telegram settings become effective
  -> Telegram runtime starts TDLib user authorization
  -> requestQrCodeAuthentication
  -> transient provider tg://login link
  -> local QR rendering
  -> optional TDLib 2FA continuation
  -> TDLib ready
```

`provider_kind` удаляется из Telegram lifecycle client request/response.
Удалённые Protobuf field number и name резервируются. Telegram integration
identity уже задаётся exact owner contract и не является runtime-selectable
setting.

`TelegramProviderKind`, `Bot` и `telegram_bot_token` удаляются из active Rust
contract, wire decoder, bootstrap и frontend. Допустимы только:

- `telegram_api_hash` с secret class `PROVIDER_CREDENTIAL`;
- `telegram_session_encryption_key` с secret class `SESSION_STORE_KEY`.

API ID остаётся non-secret typed Telegram setting. API hash и session key не
попадают в Settings, logs или provider lifecycle payload.

`qr_authorized` остаётся только compatibility field первой версии lifecycle
command и всегда передаётся frontend как `false`. Оно не является owner/client
assertion: runtime считает account авторизованным только после TDLib
authorization state `ready`.

TDLib authorization state change публикуется как typed
`telegram.authorization.status_changed.v1` через единственный Gateway SSE.
Событие содержит только fixed public state и не переносит `tg://` link,
password hint или credential material. Получив сигнал, frontend выполняет один
generated authorization status recovery query; initial/manual recovery query
также разрешён. Periodic polling запрещён.

Реальный QR:

- создаётся только из transient `tg://login?token=...`, полученного от TDLib;
- не имеет fixture/demo/fallback значения;
- не открывается во внешней навигации;
- хранится только в памяти runtime и active Telegram view;
- удаляется после state transition, cancel или unmount.

Если Telegram Bot API понадобится позже, он требует отдельного ADR, contract,
credential purpose, runtime adapter, admission gate и frontend experience.
Он не расширяет этот TDLib user account contract.

## Compatibility

Ранее объявленный bot path не был admission-complete и не имел работающего
managed runtime. Старые payload с `telegram_bot` или `telegram_bot_token`
отклоняются как invalid payload; silent conversion в user account запрещён.

Удалённые Protobuf fields резервируются, чтобы их номера и имена не были
переиспользованы с другой семантикой.

## Units of assembly

```text
telegram-api                   user-only lifecycle and authorization contracts
telegram-core                  exact Vault purposes for TDLib user setup
telegram-runtime               TDLib user bootstrap and QR lifecycle
telegram frontend              user account form, shared SSE, QR render and 2FA
owner-vault development host   provider-neutral credential sealing only
Communications domain          no Telegram account or QR dependency
```

## Gate `telegram_tdlib_user_qr_identity_v1`

Gate закрывается только при наличии:

1. no bot provider kind or bot-token purpose in active Telegram contracts;
2. no client-selectable Telegram provider kind;
3. exact API hash and session-store-key provisioning classes;
4. account provisioning with `qr_authorized = false`;
5. real TDLib QR request and transient authorization status;
6. local validation/rendering only for exact `tg://login` links;
7. shared Gateway SSE signal, initial/manual recovery query, no polling,
   cancel, cleanup and 2FA continuation;
8. no fixture/demo QR or external navigation;
9. generated client, Rust, frontend, architecture and live loopback gates.

Gate не добавляет Telegram integration в Communications domain и не открывает
Telegram Bot API.
