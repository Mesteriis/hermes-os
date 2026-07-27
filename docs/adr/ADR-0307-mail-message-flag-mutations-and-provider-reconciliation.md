# ADR-0307: Mail-owned message flag mutations and provider reconciliation

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: реализовано полностью для bounded read/unread и
  star/unstar среза. `mail_message_location_command_v1` и closure gate
  `mail_operational_command_v1` остаются `planned`.
- Связанные решения: ADR-0204, ADR-0205, ADR-0212, ADR-0213, ADR-0220,
  ADR-0223, ADR-0247, ADR-0278, ADR-0281, ADR-0282, ADR-0298, ADR-0299

## Контекст

Mail operational projection уже принадлежит Mail integration и публикуется
через `mail.operational.query.v1`. Provider delivery имеет отдельные
`mail.delivery.v1` и `mail.delivery.query.v1`. При этом clean-room не имеет
команд изменения provider message state, а historical surface смешивал эти
операции с Communications facade и generic provider-command queue.

Такое восстановление недопустимо:

- Communications не владеет provider flags и не вызывает Mail runtime;
- Kernel/Core Gateway не декодируют Mail mutation;
- generic `action` string не является typed contract;
- локальное изменение UI без provider confirmation создаёт fake state;
- один общий command service для flags, folder moves, bulk workflows и
  permanent delete нарушает SRP и смешивает разные failure semantics.

IMAP clean-room сейчас имеет канонический INBOX projection, но ещё не имеет
typed special-use folder roles и move/delete policy. Поэтому безопасный первый
write slice ограничивается convergent flag state:

```text
read / unread
starred / unstarred
```

Archive, trash, restore, move/copy, labels, spam и permanent provider delete
получат отдельный `mail_message_location_command_v1`. Cross-channel bulk
actions остаются workflow gate, а delivery остаётся существующим независимым
capability.

## Решение

### Ownership и exact contracts

Mail integration получает два независимых public contracts:

```text
mail.message-flags.command.v1
mail.message-flags.query.v1
```

Маршрут:

```text
first-party client
  -> Core Gateway authenticated exact route
  -> Mail managed runtime
  -> Mail-owned durable operation journal
  -> exact Gmail or IMAP adapter
  -> provider result
  -> atomic Mail operational projection update + terminal receipt
```

Core Gateway проверяет exact capability, contract digest, runtime generation и
grant epoch, но переносит payload opaque. Kernel не импортирует Mail packages и
не становится account, command или reconciliation service.

### Typed command

`MailMessageFlagCommandV1` содержит:

- bounded non-empty `operation_id`;
- exact `connection_id`;
- exact `provider_message_id` из Mail operational projection;
- typed mutation kind `read` или `starred`;
- explicit target boolean.

Generic strings, provider JSON, mailbox names, labels, credentials, sessions,
message content и Communications identity запрещены.

Команда задаёт желаемое состояние, а не toggle. Это делает provider execution
convergent и безопасным для повторения после uncertain transport outcome.

### Durable acceptance и idempotency

Command route возвращает только accepted receipt. `accepted` не означает
provider completion.

Mail persistence до provider I/O сохраняет canonical command bytes и SHA-256.
Повтор exact `operation_id` с теми же bytes возвращает исходный receipt.
Повтор с другим payload отклоняется. Missing or cross-connection provider
message fail closed до постановки в очередь.

Runtime исполняет только journal record, повторно декодирует canonical bytes и
сверяет digest и identity. Stale runtime, grant, storage или credential
generation не может выполнить provider I/O.

### Provider semantics

Gmail adapter использует one-message `batchModify`:

- read: remove `UNREAD`;
- unread: add `UNREAD`;
- starred: add `STARRED`;
- unstarred: remove `STARRED`.

OAuth setup запрашивает `gmail.modify` вместо `gmail.readonly`, сохраняя
отдельный `gmail.send`. Existing readonly grant не считается достаточным:
пользователь должен пройти явную повторную Gmail authorization.

IMAP adapter:

- открывает read-write `SELECT INBOX`;
- выполняет `UID STORE ... +/-FLAGS.SILENT (\Seen)` для read state;
- выполняет `UID STORE ... +/-FLAGS.SILENT (\Flagged)` для starred state;
- использует только bounded positive UID из owner-local projection.

Credentials остаются в Vault lease и не попадают в command journal, errors,
logs, subjects, health или client responses.

### Terminal outcome и projection

Отдельный query contract возвращает sanitized operation status:

```text
pending | succeeded | rejected | outcome_unknown
```

Provider success и обновление Mail projection завершаются owner-local
transaction:

- terminal operation receipt;
- exact message flags;
- monotonic message revision;
- affected thread unread count/revision;
- affected folder unread count/revision.

Если provider success произошёл, а transaction не committed, record остаётся
pending и convergent set-state может быть безопасно повторён. Transport или
provider response с недоказанным исходом становится `outcome_unknown`; UI не
показывает optimistic success. Definite invalid command/provider rejection
становится `rejected`.

Новая neutral Communications observation для read/star flag не создаётся:
provider operational flags не являются canonical Communications evidence.

### Frontend и SRP

- generated clients принадлежат exact command/query services;
- gateway adapter только кодирует typed request;
- `useMailMessageFlags` владеет одним use case: submit + status reconciliation;
- presentation получает чистую модель и не вызывает transport;
- Mail route является app-level composition point только для Mail-owned
  read, flag, composition, sync и delivery panels.

Flag controller не владеет operational reads, delivery, folders или
Communications state. Количество строк не определяет SRP.

### Capability decomposition

Capability register разделяется:

```text
mail_message_flags_command_v1
mail_message_location_command_v1
mail_operational_command_v1
```

`mail_operational_command_v1` является closure gate и остаётся `planned`, пока
не реализованы message flags, folder/location mutations и existing delivery
dependency. Bulk action не входит в этот gate.

## Gate `mail_message_flags_command_v1`

Gate становится `implemented` только атомарно при наличии:

1. exact generated command/query services and independent capabilities;
2. bounded typed validation and canonical wire round-trip;
3. owner-local additive Storage migration and exact-byte idempotency;
4. Gmail and IMAP provider adapter conformance;
5. runtime generation/credential/provider fences;
6. atomic terminal receipt plus message/thread/folder projection update;
7. restart-safe pending replay and conflicting-operation rejection;
8. generated frontend clients, Mail-owned controller and visible actions;
9. Core Gateway managed route conformance;
10. architecture guard for integration/domain/build-unit boundaries;
11. no credentials, provider payloads or private message content in
    observability surfaces.

## Последствия

Mail получает первый честный provider message mutation slice без возврата
Communications facade. Flags сходятся к явному target и остаются Mail
operational state. Folder moves, destructive delete, bulk workflow и
provider-neutral business intent сохраняют отдельные owners и gates.

## Состояние реализации

`mail_message_flags_command_v1` открыт атомарно:

- exact Protobuf command/query contracts собраны отдельными generated clients;
- canonical wire validation и Mail client contract revision 9 покрыты Rust
  regression tests;
- Mail Storage bundle revision 12 содержит exact-byte durable operation
  journal без cross-owner или projection-lifecycle foreign key;
- Gmail `batchModify` и IMAP read-write `SELECT` + `UID STORE` реализуют
  convergent target-state semantics;
- runtime исполняет journal record через provider-owned adapter и только после
  provider confirmation атомарно обновляет message/thread/folder projection;
- focused managed test
  `managed_mail_message_flags_reconcile_provider_and_projection` подтвердил
  Core Gateway route, Vault credential lease, live loopback IMAP mutation,
  terminal status, unread reconciliation и provider-side-effect-free exact
  replay;
- Vue Mail surface использует отдельные generated command/query clients,
  gateway, controller и presentation model без optimistic success;
- architecture guard и frontend unit/boundary tests фиксируют SRP, capability
  admission и integration/domain separation.
