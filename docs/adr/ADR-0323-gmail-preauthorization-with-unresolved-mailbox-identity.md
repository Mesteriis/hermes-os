# ADR-0323: Gmail pre-authorization без выдуманной mailbox identity

Статус: Принято
Дата: 2026-07-28
Состояние реализации: implemented. Mail Settings принимает exact Gmail
pre-authorization target без выдуманного `from_address`, runtime сохраняет
delivery fencing, а live recovery запускает current OAuth и не импортирует
legacy token.

Уточняет:

- [ADR-0204: bundled integration plugins](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0294: Mail portability](ADR-0294-mail-account-portability-and-resumable-owner-import.md);
- [ADR-0319: provider-specific account setup](ADR-0319-provider-specific-account-setup-lifecycle-and-mail-multi-account.md);
- [ADR-0320: Mail multi-account targets](ADR-0320-mail-multi-account-configuration-instances-and-runtime-multiplexing.md);
- [ADR-0321: legacy provider recovery](ADR-0321-legacy-provider-recovery-bundle-and-native-secret-custody.md).

## Контекст

Legacy Gmail OAuth разрешал не указывать email до provider authorization. В
этом случае durable `external_account_id` становился opaque account ID. Он
сохраняет source identity, но не является допустимым RFC mailbox.

Current Gmail setup требует:

```text
target settings
  -> Mail runtime
  -> current OAuth
```

OAuth start выполняет Mail integration runtime, поэтому target должен
запуститься до получения provider credential. Подстановка
`opaque-id@gmail.com`, адреса из display label или другого синтетического
mailbox создала бы ложную provider identity.

## Решение

Mail integration вводит exact pre-authorization state для Gmail.

### Configuration

- `mail.gmail.user_id = "me"` является documented provider operational alias,
  а не business identity;
- `mail.gmail.from_address` может отсутствовать только для Gmail target с
  exact `user_id = "me"`;
- обычный `Add Gmail account` по-прежнему требует owner-entered address;
- legacy recovery не доверяет non-mailbox `external_account_id` и создаёт
  pre-authorization settings без `from_address`;
- IMAP/SMTP address requirements не меняются.

Отсутствующий address представлен как `Option`, а не empty-string sentinel.

### Runtime behavior

Pre-authorization target может:

- запустить current Gmail OAuth;
- получить и хранить current access/refresh credentials через existing Owner
  Vault boundary;
- после authorization использовать provider alias `me` для Gmail sync.

Он не может:

- отправлять mail до configured valid `from_address`;
- заявлять delivery readiness `ready`;
- экспортироваться как complete portable Gmail account;
- публиковать opaque legacy identity как mailbox или canonical truth.

Account status сохраняет provider-specific distinction:

```text
OAuth credentials absent
  sync = credential_required
  delivery = not_configured

OAuth credentials current, address absent
  sync = ready
  delivery = not_configured
```

Поздний owner Settings update с valid mailbox и supervised apply открывает
delivery path. Provider profile discovery и автоматическая address
reconciliation требуют отдельного contract; данный gate их не выдумывает.

## Units of assembly

```text
legacy recovery app workflow
  chooses pre-authorization settings for legacy Gmail

Mail Settings decoder
  owns optional Gmail from-address semantics

Mail runtime
  owns OAuth, sync readiness and delivery fencing

Kernel / Gateway
  transport typed owner settings and Mail client RPC only

Communications domain
  no Gmail configuration or provider identity dependency
```

## Failure and privacy

- non-`me` Gmail user ID without mailbox is invalid settings;
- invalid present mailbox is invalid settings;
- delivery without mailbox returns a sanitized admission failure before
  provider execution or durable send acceptance;
- recovery UI and receipts do not render raw legacy account identity;
- legacy OAuth token/client secret are still not imported;
- Kernel, Communications and generic frontend platform code do not interpret
  Gmail address semantics.

## Gate `gmail_pre_authorization_identity_v1`

Gate закрывается только при наличии:

1. optional Gmail `from_address` contract with exact `user_id = "me"` fence;
2. Settings decode tests for accepted pre-authorization and rejected invalid
   combinations;
3. Mail status tests proving sync/delivery readiness distinction;
4. delivery negative proving no enqueue/provider call without mailbox;
5. recovery workflow test proving no synthetic address is written;
6. complete Gmail portability rejection while mailbox is unresolved;
7. architecture and secret-negative tests;
8. live legacy Gmail target reaching current `reauthorization_required`.
