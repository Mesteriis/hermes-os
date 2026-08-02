# ADR-0379: Mail address-book sync and Contacts command boundary

Статус: Принято

Дата: 2026-08-02

Состояние реализации: planned. Решение принято до production implementation.
Наличие legacy address-book service, Mail account UI или статических contracts
не открывает `contacts_mail_identity_command_v1` либо `mail_contacts_sync_v1`.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0207](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0208](ADR-0208-domain-development-allowlist-and-projection-freeze.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0222](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0236](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0294](ADR-0294-mail-account-credential-lifecycle-and-portability.md).

## Контекст

Inventory ADR-0282 требует `mail_contacts_sync_v1`, но active workspace не
содержит production Contacts owner packages. Legacy implementation выполняла
provider fetch/write, canonical person mutation, provider-link persistence,
scheduling и HTTP response внутри одного service и общего PostgreSQL pool.
Такой перенос сделал бы Mail integration владельцем Contacts truth и нарушил
бы event-only owner boundary.

Полезное observable behavior legacy остаётся обязательным:

- manual и scheduled sync для конкретного Mail account;
- paginated Google Contacts и read-only iCloud CardDAV import;
- stable provider-entry linkage и idempotent canonical upsert;
- optional bidirectional write только при явном desired setting и provider
  write scope;
- ETag fence для update и безопасная блокировка ambiguous remote write;
- bounded run counters, terminal result и restart-safe retry.

## Решение

### Owners и ответственность

Capability разделяется между тремя owners:

```text
Mail integration provider observation/result
  -> mail_contacts_sync workflow
  -> Contacts durable command
  -> Contacts owner-local mutation/result

Contacts changed event
  -> mail_contacts_sync workflow
  -> Mail provider write command
  -> Mail provider result
  -> workflow link/checkpoint state
```

Mail integration владеет Google People/CardDAV protocol, provider pagination,
OAuth/scopes, ETag и provider-specific errors. Contacts domain владеет person,
email/phone identity invariants и canonical provider provenance. Workflow
владеет направлением sync, correlation, checkpoints, retry и run report.

Ни один owner не импортирует implementation, persistence или runtime другого.
Kernel, Gateway и Event Hub не интерпретируют address-book payload и не
становятся sync facade. Communications в flow не участвует и не получает
Contacts state.

### Единицы сборки

Contacts получает минимальный production slice:

- `hermes-contacts-command-api` — exact durable upsert command и terminal
  result contracts;
- `hermes-contacts-core` — canonical identity normalization, merge/conflict
  invariants и deterministic IDs;
- `hermes-contacts-persistence` — Contacts-owned canonical state, inbox/outbox
  и provider provenance;
- `hermes-contacts-runtime` — managed command consumer/result publisher;
- `hermes-contacts-assembly` — descriptor, empty initial Settings schema,
  Storage bundle и unsigned release fragment.

Mail добавляет integration-owned contract unit
`hermes-mail-address-book-contract` с typed provider observations, provider
write commands и terminal results. Provider adapter остаётся в Mail runtime.

Workflow получает пять units:

- `hermes-mail-contacts-sync-api` — generated Start/Get/replayable realtime;
- `hermes-mail-contacts-sync-core` — pure direction, correlation and lifecycle;
- `hermes-mail-contacts-sync-persistence` — owner-local run, inbox/outbox,
  checkpoints and realtime replay;
- `hermes-mail-contacts-sync-runtime` — event orchestration and Scheduler job
  handler;
- `hermes-mail-contacts-sync-assembly` — descriptor, typed Settings schema,
  Storage bundle and unsigned release fragment.

Assembly не является domain, integration или workflow implementation и не
получает runtime signing authority.

### Contacts command

`UpsertContactFromProviderEntryV1` содержит только bounded typed fields:

- logical owner, source Mail account and provider kind;
- stable provider entry ID and optional ETag;
- display name, normalized email addresses and E.164-capable phone candidates;
- observed time, source revision and deterministic content digest.

Contacts валидирует syntax и limits, нормализует identities и применяет exact
precedence:

1. existing provider link;
2. unique normalized email;
3. unique normalized phone;
4. otherwise a new deterministic Contact.

Conflicting provider link or identity matching multiple Contacts fails closed;
automatic cross-contact merge is forbidden. Name-only entry is accepted only
when an existing provider link already identifies the Contact. Result reports
`created`, `updated`, `unchanged` or bounded rejection and never returns another
owner's private state.

Inbox validates exact message ID/hash before mutation. Canonical mutation,
provider provenance and terminal result outbox commit atomically. Retry of the
same command replays the same result; command identity reuse with a different
digest is rejected.

### Provider contracts и bidirectional write

Mail publishes one typed observation per provider entry and one explicit page
terminal event. Raw Google/CardDAV JSON/XML, tokens, scopes, cookies and error
detail never leave Mail. Workflow does not call provider SDK or read Mail SQL.

For local-to-provider flow Contacts publishes a bounded changed event with an
opaque contact reference and exact revision. Workflow requests an authorized
Contacts export snapshot through a distinct target-bound Blob handoff, then
emits exact Mail provider upsert command. Mail verifies current account,
credential lease, provider write scope, expected ETag, runtime generation and
grant epoch before network mutation. ICloud v1 remains read-only and rejects
remote write explicitly.

Accepted command is not provider completion. Provider result returns by a
separate durable event. Ambiguous timeout is `outcome_unknown`; workflow does
not blindly repeat it without provider reconciliation.

### Settings, Scheduler и client transport

Sync configuration belongs to the workflow typed Settings schema:

- enabled account IDs;
- direction `provider_to_contacts` or `bidirectional`;
- bounded interval;
- explicit remote-write enablement.

Credentials, OAuth scopes, cursors, checkpoints, last run and errors are not
Settings. Kernel Settings Registry stores desired/effective revisions and
supervises apply; Mail and Contacts do not merge workflow settings.

Scheduled execution uses the admitted Scheduler durable job contract. Manual
Start/Get use generated client contracts through Core Gateway. Terminal and
progress updates use the single replayable client SSE stream; periodic polling
and handwritten REST are forbidden.

### Privacy и failure isolation

Contact presentation fields and provider identifiers are business data and are
forbidden in subjects, health, logs and sanitized errors. Every owner uses its
own PostgreSQL role/schema and inbox/outbox. Restart, revoke, stale runtime
generation, stale grant epoch, stale source/contact revision and ETag conflict
fail closed without duplicate canonical or provider mutation.

Deleting or retiring a Mail account stops new sync work but does not delete
Contacts truth. Removing a provider link is a separate Contacts command and is
not inferred from one missing provider page.

## Phase gates

### `contacts_mail_identity_command_v1`

Opens only after:

1. five exact Contacts units and compile isolation;
2. typed command/result and provider provenance without generic maps;
3. owner-local Storage bundle and atomic inbox/mutation/outbox;
4. deterministic normalization/idempotency and ambiguity negatives;
5. signed managed runtime evidence through real Vault, Storage and NATS;
6. duplicate/conflict, restart, revoke, generation/grant and privacy gates.

### `mail_contacts_sync_v1`

Opens only after the Contacts gate plus:

1. Mail provider observation/write contracts and real Google/CardDAV adapters;
2. five workflow units with typed Settings and Scheduler binding;
3. manual and scheduled provider-to-Contacts E2E;
4. authorized Google bidirectional create/update with ETag fencing;
5. explicit iCloud read-only and missing-write-scope negatives;
6. provider outage/outcome-unknown recovery, pagination checkpoint/restart and
   duplicate event conformance;
7. generated Start/Get, shared SSE and account settings UI cutover;
8. architecture, Cargo, unit, disposable PostgreSQL, managed runtime, browser
   and full pre-push gates.

Static contracts, fixtures, seeded contacts, direct service calls, shared SQL,
frontend skeletons or a provider mock do not open either gate.

## Последствия

- Mail остаётся integration, Contacts — domain, sync — workflow.
- Canonical Contacts truth больше не живёт в Communications или Mail storage.
- Bidirectional write получает явную authority и terminal reconciliation.
- Полный capability требует дополнительного owner runtime, но каждый unit
  сохраняет одну функциональную причину изменения.

## Отклонённые варианты

### Перенести legacy service внутрь Mail runtime

Mail получил бы Contacts policy и cross-owner SQL.

### Дать Contacts provider SDK и OAuth credential

Domain стал бы integration adapter и потерял provider neutrality.

### Реализовать sync как Kernel/Scheduler callback

Owner-neutral platform начала бы интерпретировать business payload.

### Считать provider command acceptance завершённым sync

Это скрывает provider outage и ambiguous mutation после accepted receipt.
