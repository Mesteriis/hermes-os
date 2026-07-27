# ADR-0294: Mail account credential lifecycle and portability

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Phase 1 `mail_account_credential_binding_v1`
реализована. Retire/delete и portability остаются Planned; umbrella
`mail_account_lifecycle_v1` остаётся закрыт до выполнения всех phase gates
ниже.

Уточняет:

- [ADR-0204: integration/provider boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0236: integration owners and configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0263: Mail settings and Storage admission](ADR-0263-mail-integration-settings-and-storage-admission.md);
- [ADR-0278: Gmail OAuth](ADR-0278-mail-gmail-oauth-setup-and-refresh-gate.md);
- [ADR-0292: managed settings apply](ADR-0292-managed-integration-settings-apply-and-credential-binding.md);
- [ADR-0293: scoped Vault retirement](ADR-0293-scoped-vault-credential-retirement-and-deletion.md).

## Контекст

Mail runtime уже имеет exact sync, delivery и Gmail OAuth capabilities, но
account lifecycle остаётся неполным:

- IMAP/SMTP credential revisions хранятся в Kernel Settings snapshot;
- managed runtime не может стать ready до provisioned IMAP password;
- нет Mail-owned sanitized binding/status query;
- logout/delete не quiesce-ят provider I/O и не retire/delete-ят Vault
  credentials;
- legacy import/export semantics не классифицированы между Settings, Vault,
  Mail и app composition.

Credential revision является Vault binding metadata, а не non-secret
configuration. ADR-0263 в этой части заменяется ADR-0292 и этим решением:
Mail endpoint/account configuration остаётся Settings, credential binding
переходит в Mail-owned persistence.

## Решение

### Owner boundaries

Один Mail configuration instance составляется из независимых authority:

```text
Kernel Settings Registry
  connection ID, IMAP/Gmail/SMTP endpoints, account identifiers,
  sync policy and OAuth public configuration

Vault
  IMAP password, SMTP password, Gmail access token and refresh credential

Mail persistence
  purpose-specific credential revision binding, lifecycle state,
  Gmail OAuth attempt/binding, provider cursors and operational state

first-party app
  import/export/logout/delete user flow composition
```

Communications не хранит Mail settings, credentials, folders, provider state
или lifecycle. Kernel/Gateway не декодируют Mail account commands.
Все non-secret Mail Settings owner-editable с fresh owner proof; обычный
sanitized account query не дублирует endpoint, username/email или CA material.

### Exact credential purposes

Mail использует только закрытый enum:

```text
imap_password
smtp_password
gmail_access_token
gmail_refresh_credential
```

Client не передаёт arbitrary purpose, secret reference, record ID, Vault
location, password/token или provider payload.

### Phase 1: credential binding

`mail_account_credential_binding_v1` добавляет два независимых generated
contracts:

```text
mail.account.credential.bind.v1
  /hermes.mail.account.v1.MailAccountCredentialBindingService/Bind

mail.account.query.v1
  /hermes.mail.account.v1.MailAccountQueryService/Get
```

Bind принимает:

```text
connection_id
purpose = imap_password | smtp_password
expected_binding_revision
credential_revision
```

Gmail revisions не bind-ятся client command: их создаёт уже принятый typed
OAuth workflow ADR-0278.

Mail Storage хранит purpose-specific CAS binding:

```text
connection_id
configuration_instance_id
purpose
credential_revision
binding_revision
state = pending_restart | active | retired | deleted
applied_runtime_generation?
```

Bind немедленно quiesce-ит соответствующий provider path текущего runtime.
Credential применяется только новым managed generation через generic
ADR-0292 Settings successor. Runtime resolve-ит exact bound revision,
atomically отмечает binding active и только затем открывает provider I/O.

Mail runtime может быть ready в configuration-only state:

- Storage, lifecycle/query and Gmail OAuth setup routes доступны;
- IMAP sync отключён без active IMAP binding;
- SMTP delivery отключена без active SMTP binding;
- Gmail sync/delivery отключены без active OAuth binding;
- Communications observations не создаются без provider execution.

### Phase 2: retire and delete

`mail_account_retire_delete_v1` вводит отдельные durable command/status
contracts. Mail сначала durably quiesce-ит account, затем для каждой bound
purpose вызывает ADR-0293 exact `retire` или `delete`.

Multiple purpose mutation хранит per-purpose progress. Потерянный Vault
response reconciles explicit retry through idempotent exact-state action; нет
silent automatic retry. Terminal states:

```text
completed
rejected
outcome_unknown
```

Retire/delete не удаляют Communications evidence и не обращаются к
Communications storage. Delete создаёт Mail account tombstone; physical
provider-side deletion выполняется только если отдельный provider contract
честно поддерживает такую semantics.

### Phase 3: portability

`mail_account_portability_v1` является first-party app composition, а не новым
domain или generic provider facade.

Export объединяет:

- effective non-secret Mail Settings snapshot;
- sanitized Mail account status and connector profile;
- optional provider resource mapping metadata;
- schema/contract versions.

Export никогда не содержит credential bytes, Vault record IDs, wrapping keys,
OAuth codes/verifiers, provider message content или sync cursors.

Import выполняет explicit sequence:

```text
validate typed export
  -> create/update Mail Settings desired revision
  -> sealed owner Vault provisioning
  -> Mail credential bind or Gmail OAuth setup
  -> generic managed Settings apply
  -> query Mail readiness
```

App не пишет Mail/Kernel/Vault stores и не создаёт hidden global transaction.
Partial state остаётся видимым и resumable через exact receipts.

### Account query

Реализованный Phase 1 sanitized query возвращает только:

- connection ID and connector profile;
- effective settings revision;
- aggregate account readiness;
- per-purpose binding state and revisions;
- applied runtime generation;
- sync/delivery readiness reason codes.

Phase 2 совместимо добавляет current pending lifecycle operation receipt и
terminal lifecycle state.

Endpoint host, username/email и CA material не возвращаются обычным status
query. Typed export требует отдельной fresh-owner-proof operation.

### Units of assembly

```text
hermes-mail-api
  generated account contracts and wire mapping

hermes-mail-persistence
  credential binding CAS, lifecycle journal and account tombstone

hermes-mail-runtime
  provider quiesce, Vault orchestration and readiness

hermes-mail-imap / gmail / smtp
  provider protocol adapters only

Kernel settings apply
  provider-neutral successor replacement

app portability composition
  first-party client workflow only

hermes-mail-assembly
  immutable release artifacts only
```

Runtime не становится assembly, integration не становится domain, app
composition не получает owner storage.

## Phase gates

### `mail_account_credential_binding_v1`

1. Settings schema без credential revision/reference;
2. exact Bind and Query generated contracts;
3. purpose-specific owner-local CAS binding;
4. configuration-only runtime;
5. bind quiesce and successor-only activation;
6. stale binding/settings/runtime/grant/storage/Vault negatives;
7. sanitized status without secret carriers;
8. live IMAP/SMTP rotation and no-provider-I/O evidence;
9. architecture/SRP/Cargo/Clippy/workspace gates.

### `mail_account_retire_delete_v1`

1. exact durable retire/delete/status contracts;
2. per-purpose durable progress and explicit retry;
3. provider quiesce before first Vault mutation;
4. ADR-0293 tombstone evidence for every bound purpose;
5. Gmail access/refresh credentials handled separately;
6. restart/revoke/stale revision negatives;
7. no Communications deletion or direct store access;
8. sanitized terminal state and privacy negatives.

### `mail_account_portability_v1`

1. typed versioned non-secret export;
2. fresh owner proof;
3. sealed provisioning dependency;
4. resumable multi-receipt import;
5. no secret/session/content/cursor carriers;
6. desktop generated client and integration-owned UI.

### `mail_account_lifecycle_v1`

Umbrella открывается только после всех трёх gates выше и существующего
`mail_gmail_oauth_v1`.

## Evidence реализованной Phase 1

- `hermes-mail-api` поставляет exact generated Bind/Query contracts без
  secret bytes, Vault record IDs и arbitrary purposes;
- Mail Settings schema major 2 содержит только owner-editable non-secret
  configuration и не содержит credential revisions;
- Mail Storage bundle revision 7 хранит purpose-specific CAS binding;
- runtime стартует configuration-only, quiesce-ит изменённый path после Bind
  и активирует exact Vault revision только в successor generation;
- `mail_account_credential_flow` live-conformance проверяет IMAP/SMTP rotation,
  отсутствие provider I/O в `pending_restart`, generic Settings Apply,
  activation revision 2 и stale-generation fencing;
- executable architecture gate:
  `tests/architecture/mail-account-credential-binding.test.mjs`.

## Отклонённые варианты

### Credential revisions в Settings

Отклонено: нарушает ADR-0222/0292 и смешивает configuration с Vault binding.

### Общий IntegrationAccount domain

Отклонено: скрывает разные provider lifecycle semantics и создаёт новый
cross-integration facade.

### Communications Mail account API

Отклонено: provider authorization, folders, sync и delivery принадлежат Mail.

### Runtime hot-swap credential без successor

Отклонено: старые in-flight provider requests не получают однозначного
generation fence.

### Export credentials в файл

Отклонено: portability переносит только non-secret configuration; secret
transfer требует отдельной Vault backup/recovery ceremony.

## Последствия

- Mail Settings становятся non-secret;
- IMAP/SMTP/Gmail lifecycle остаётся у Mail integration;
- logout/delete используют реальный Vault revocation primitive;
- import/export не создают общий Channels facade;
- полный Mail lifecycle требует нескольких независимых commits/gates, а не
  одного всесильного runtime handler.
