# ADR-0319: Owner-authorized legacy provider account recovery

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: planned. Решение должно быть реализовано и доказано
  live recovery evidence до перевода `legacy_provider_account_recovery_v1` в
  `implemented`.
- Связанные решения: ADR-0200, ADR-0201, ADR-0204, ADR-0205, ADR-0213,
  ADR-0215, ADR-0222, ADR-0223, ADR-0240, ADR-0267, ADR-0278, ADR-0292,
  ADR-0293, ADR-0294, ADR-0295, ADR-0300, ADR-0303, ADR-0309, ADR-0310

## Контекст

Владелец явно разрешил однократное восстановление provider accounts из
архивного pre-clean-room хранилища. В проверенном источнике есть ровно три
пригодные активные конфигурации:

- один Gmail account;
- один iCloud Mail account;
- один Telegram user account.

Две дополнительные Gmail-записи уже помечены удалёнными и не имеют credential
references. Они не являются кандидатами восстановления.

Архивный PostgreSQL, Host Vault и TDLib state являются untrusted recovery
source, а не active owner stores и не compatibility runtime. Простое копирование
SQL, Vault records или provider databases нарушило бы clean-room boundaries:

- Mail и Telegram имеют разные lifecycle и credential contracts;
- Communications не владеет provider accounts, sessions или settings;
- Kernel не интерпретирует provider payload и не становится import service;
- frontend не должен получать secret bytes, legacy paths или TDLib database;
- старый OAuth token не доказывает совместимость с новым Gmail binding;
- старая TDLib database не является доказательством новой QR authorization.

Нужно восстановить accounts через действующие provider-owned contracts, не
возвращая legacy facade и не создавая generic `AccountService`.

## Решение

### Recovery является host application workflow

`legacy_provider_account_recovery_v1` реализуется как локальный first-party
host recovery workflow. Это не domain, integration, Kernel component или
managed module.

Workflow:

```text
read-only legacy source clone
  -> provider-specific recovery plan
  -> current Core Gateway owner contracts
  -> current owner Vault provisioning host
  -> provider-specific lifecycle/query contracts
  -> provider runtime observations
  -> Communications through durable events
```

Host workflow может читать только explicit legacy source roots, заданные
owner-ом при запуске. Он не пишет legacy source и не получает произвольный
target filesystem path. Все target mutations выполняются только через current
public owner contracts и current runtime fences.

Recovery tool не входит в signed managed-module inventory и не запускается
`make dev` автоматически. Он является отдельной maintenance unit и требует:

- loopback-only target;
- current owner session и fresh proof для target mutations;
- stopped legacy source либо read-only copy;
- explicit source fingerprint;
- explicit dry-run перед apply;
- точное подтверждение количества и provider kinds;
- idempotency key и resumable sanitized receipt.

### Provider-specific units

Один host orchestrator использует три независимые функциональные units:

```text
legacy account catalog reader
  structural PostgreSQL read-only discovery and source fingerprint

legacy Host Vault reader
  local Keychain-backed decrypt, bounded secret bytes, zeroization

Mail recovery composition
  typed Mail portability, Settings apply, exact credential provisioning/bind

Telegram recovery composition
  typed account identity, API credential provisioning and real TDLib QR start
```

Mail recovery code не импортирует Telegram contracts или state. Telegram
recovery code не импортирует Mail contracts. Reader adapters не импортируют
target integration implementations. Количество строк не является критерием
SRP; единицей разделения является причина изменения.

### Source admission

V1 принимает только:

- account state `active`;
- provider kind `gmail`, `icloud` или `telegram_user`;
- существующий exact secret reference для требуемого purpose;
- source records, у которых account/secret references согласованы;
- ровно один logical record на candidate account;
- bounded UTF-8 configuration values;
- source snapshot с неизменившимся fingerprint между dry-run и apply.

`telegram_bot`, deleted records, duplicate accounts, missing secret references,
unknown provider kinds, malformed encrypted records и changed source fail
closed. Recovery никогда не удаляет и не изменяет legacy files.

Tool выводит только provider kind, candidate count, sanitized state и source
fingerprint. Email, username, token, password, API hash, Vault record ID,
Keychain material, TDLib database key, filesystem path и provider-private
payload не попадают в stdout, logs, receipts или UI.

### Mail recovery

Mail account configuration восстанавливается через реализованный typed
`mail_account_portability_v1`:

```text
validate typed non-secret account export
  -> create desired Mail Settings
  -> configuration-only successor
  -> query current binding revisions
  -> exact owner Vault provisioning
  -> Mail credential bind
  -> credential successor
  -> query provider-path readiness
```

iCloud V1 допускает перенос legacy app password только если Host Vault reader
успешно расшифровал exact `imap_password` record и current Mail descriptor
допускает exact configuration-instance provisioning. Secret bytes передаются
только sealed local host transport и zeroize-ятся после ответа.

Gmail non-secret account configuration переносится всегда после validation.
Legacy OAuth payload переносится только если отдельный exact compatibility
validator доказывает полный current access/refresh credential shape и Mail-owned
binding operation атомарно принимает обе новые Vault revisions. До реализации
такого contract V1 не инъектит старый bearer token и возвращает
`reauthorization_required`; владелец завершает действующий Gmail OAuth flow.
Нельзя подменять OAuth success локальной записью или client assertion.

Partial success сохраняется как provider-specific resumable receipt. Готовность
iCloud не маскирует требуемую Gmail reauthorization.

### Telegram recovery

V1 восстанавливает только:

- user-account identity;
- display label;
- Telegram API ID;
- sealed Telegram API hash credential;
- non-secret settings, необходимые current QR lifecycle.

Legacy TDLib database, WAL, downloaded files и session key не копируются в
current integration state root. После configuration-only successor Telegram
runtime начинает существующий user-only flow ADR-0310:

```text
prepare account
  -> TDLib RequestQrCodeAuthentication
  -> short-lived real login link
  -> QR rendered locally
  -> owner scans with Telegram mobile client
  -> TDLib AuthorizationStateReady
```

Bot token запрещён. Fake QR, synthetic `tg://login` payload, browser-generated
authorization success и перенос opaque legacy authorization link запрещены.
Если API credential невозможно совместимо восстановить, account остаётся
`blocked_config` и QR не показывается до explicit correction.

Восстановление уже авторизованной TDLib session является отдельной будущей
recovery ceremony и не входит в этот ADR.

### Communications и historical provider data

Recovery workflow не пишет Communications tables, canonical message IDs,
conversations, search projections или Blob custody. Legacy provider message
cache, raw MIME и TDLib history не импортируются напрямую.

После provider authorization integration выполняет обычный sync и публикует
neutral observations через durable outbox/NATS. Communications принимает их
своим inbox и создаёт canonical evidence. Поэтому account recovery не создаёт
business truth и не обходится без event spine.

### Idempotency и receipts

Каждый candidate получает stable recovery key:

```text
source fingerprint
provider kind
legacy account opaque digest
target configuration instance
recovery revision
```

Secret bytes и raw identifiers в key не входят. Повторный apply:

- не создаёт duplicate target account;
- продолжает с последнего подтверждённого provider-specific шага;
- использует current CAS revisions;
- не повторяет Vault mutation после ambiguous outcome;
- требует explicit retry после `outcome_unknown`;
- не считает `accepted` terminal provider readiness.

Sanitized terminal states:

```text
completed
reauthorization_required
qr_authorization_required
blocked_source
blocked_config
outcome_unknown
```

## Phase gate `legacy_provider_account_recovery_v1`

1. Отдельный host recovery workflow и provider-specific units без нового
   business domain или managed runtime.
2. Read-only source clone, exact fingerprint и unchanged-source gate.
3. Structural discovery ровно двух active Mail и одного Telegram user account;
   deleted Gmail records исключены.
4. Legacy Host Vault decrypt только через local Keychain без secret output.
5. Typed Mail portability и exact iCloud credential provisioning/bind through
   current public contracts.
6. Gmail config recovery с real OAuth либо отдельным доказанным exact
   compatibility contract; generic token injection запрещён.
7. Telegram user config/API credential recovery и real TDLib QR; bot/session
   database import запрещены.
8. Никаких direct target SQL, Vault store writes, arbitrary target paths или
   Communications mutations.
9. Idempotent resumable receipts, changed-source/replay/stale-CAS/ambiguous
   outcome negatives.
10. Secret-negative stdout/log/error/browser evidence.
11. Live target evidence: два Mail accounts видимы с честной per-path
    readiness, Telegram user account показывает настоящий QR до scan.
12. Architecture, SRP, Cargo boundaries, formatting, lint, unit/integration и
    relevant frontend tests.

## Последствия

- Восстановление старых accounts не возвращает legacy runtime или facade.
- Integration остаётся owner-ом provider lifecycle; Communications получает
  только новые neutral observations.
- iCloud может стать operational без повторного ввода app password, если exact
  legacy credential валиден.
- Gmail может потребовать реальную повторную OAuth authorization.
- Telegram обязательно требует новый real QR scan; старая session database не
  переносится молча.
- Recovery workflow остаётся удаляемой maintenance unit после завершения
  migration и не расширяет обычную runtime authority.

## Отклонённые варианты

### Скопировать legacy PostgreSQL rows в current integration schemas

Отклонено: это обходит typed lifecycle, Settings, Vault, CAS, runtime fences и
provider-specific invariants.

### Скопировать legacy Vault database или Keychain references

Отклонено: current Vault records имеют другие owner, purpose, generation,
grant и tombstone semantics.

### Скопировать TDLib database и считать Telegram авторизованным

Отклонено: это скрытая session migration без отдельной recovery ceremony и
противоречит обязательному real QR onboarding V1.

### Импортировать accounts в Communications

Отклонено: provider account не является Communications domain entity.

### Один generic AccountService

Отклонено: Mail OAuth/password lifecycle и Telegram TDLib QR имеют разные
authority, state machines и failure modes.
