# ADR-0239: First owner Mail/IMAP read-only vertical slice

Статус: Предложено  
Дата: 2026-07-20  
Состояние реализации: Не реализовано. Этот ADR выбирает целевой first-owner
contract, но **не** authorizes создание production Mail/Communications
packages, не открывает `first_owner_v1` и не изменяет active inventory до
атомарного открытия всех prerequisite gates.

Зависит от:

- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0220: Канонический durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0223: Vault и credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0225: Фазовые ворота первого production slice](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md);
- [ADR-0233: Whole-instance backup и fenced restore](ADR-0233-whole-instance-backup-and-fenced-restore.md);
- [ADR-0236: Integration owners, protocol adapters и configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md).

## Решение

После открытого `first_owner_v1` первым owner будет read-only Mail integration
с единственным IMAP adapter и её canonical evidence consumer — Communications.
Это один vertical slice, а не разрешение общего email-клиента, generic provider
framework или будущих POP3/SMTP capabilities.

### Exact atomic inventory

Один atomic policy change добавляет только следующие packages:

```text
hermes-mail-api
hermes-mail-core
hermes-mail-imap
hermes-mail-persistence
hermes-mail-runtime
hermes-communications-ingress
hermes-communications-api
hermes-communications-domain
hermes-communications-persistence
hermes-communications-runtime
```

`hermes-mail-api`, `hermes-communications-api` и
`hermes-communications-ingress` содержат только versioned typed public
contracts. `mail-core` содержит synchronous reconciliation, idempotency and
cursor ports; он не зависит от IMAP, Vault, PostgreSQL, NATS или
Communications implementation. `mail-imap` — rustls protocol adapter и не
видит persistence, Gateway или domain implementation. Каждый persistence
package владеет только своим schema, inbox/outbox и storage port. Только
соответствующий `*-runtime` композирует private packages; Mail runtime вызывает
Communications только через public `hermes-communications-ingress`.

Kernel/Gateway не получают dependency на эти packages. Нельзя создавать
`hermes-mail`, `hermes-communications` или shared provider-SDK god-crate,
cross-owner SQL, direct module socket, legacy re-export или любую production
path/import/generated reference под `references/**`.

### Mail operational contract

Public Mail API содержит ровно `BeginImapConnection`,
`CompleteImapConnection`, `GetConnection`, `SyncNow`, `GetSyncStatus` и
`GetOperationStatus`. Идентификатор операции стабилен, результат terminal
queryable и не раскрывает secret/provider body. Состояния connection: only
`provisioning`, `ready`, `syncing`, `degraded`, `retired`.

Onboarding создаёт pending IMAP configuration. Tauri host adapter временно
HPKE-seals пароль для Vault, zeroizes input и передаёт Mail/Gateway/Kernel
только ciphertext and opaque references. Owner device proof, fresh challenge,
Vault lock and replay failure deny admission without a fallback path.

Поддерживается только IMAPS (`993`) over rustls, one `INBOX`, `EXAMINE` и
`BODY.PEEK`. Plaintext, STARTTLS, POP3, SMTP, IDLE, folder discovery и provider
writes отсутствуют и не advertised in descriptor/capabilities.

One protocol step has a 10-second deadline; entire sync has a 5-minute
deadline. Each configuration has at most one concurrent sync, and runtime has
four globally. Initial window is 100 UIDs; later work is at most 500 messages
across ten windows. Mail owns cursor, retry, provider locator/source mapping
and its outbox.

Raw message is bounded to 1 MiB; only decoded `text/plain` up to 256 KiB may
be persisted. HTML, attachments and raw MIME are neither stored nor emitted.
Oversized or no-text input yields metadata-only evidence. On UIDVALIDITY reset,
source record may be reused only for an unambiguous `(Message-ID,
INTERNALDATE, content digest)` match; otherwise a new evidence identity is
created.

### Communications and Blob boundary

Mail publishes one idempotent neutral observation through ingress. It contains
no provider locator and no private content. Communications owns canonical
evidence, its inbox, body metadata and read models. When a body is admitted,
Mail receives a Communications-owned pending Blob reservation and one-use
write ticket, writes bytes directly to Blob, and only then publishes the
observation. Orphan reservation, hash conflict and duplicate publication are
terminal, typed outcomes; no cross-owner Blob/database access is allowed.

Generated owner clients route all UI requests via Core Gateway. The first UI is
read-only and reuses visual primitives only; it exposes setup, connection and
rendered `text/plain` evidence. Compose, AI, search, import, attachment and
provider-write affordances are absent.

### Admission evidence

The atomic gate change requires executable evidence for all platform
prerequisites plus this slice:

- disposable implicit-TLS Dovecot E2E:
  `UI setup → Tauri HPKE → Vault → IMAP read-only sync → Blob → durable observation → Communications → Gateway generated client → text/plain render`;
- duplicate sync, lost mutation response/restart-replay, UIDVALIDITY reset,
  oversized/no-text, wrong/replayed challenge, locked Vault, orphan Blob
  reservation and hash conflict tests;
- compile-isolation tests for every package edge and negative reference/legacy
  import checks;
- evidence record for every promoted/AI-assisted object with explicit source,
  evidence, confidence, observed/created time, causation, correlation and
  actor/system source. Generic metadata maps are forbidden.

The release binding, descriptor capabilities and StorageBundle are created only
in that same change. A green isolated unit test, an ADR, or code copied from
reference does not admit the slice.

## Reference use

`references/backend-legacy/` and legacy frontend are historical evidence only:
they may inform protocol algorithm, fixture and visual primitive selection.
Every adopted behaviour must receive a new typed boundary, independent
regression test and this ADR as evidence link. No legacy source, migration,
generated output, compatibility facade or runtime dependency enters the
clean-room graph.

## Consequences

Mail is intentionally narrow, useful and reversible: it proves provider to
canonical evidence to client flow without letting raw mail create durable
business truth. POP3, SMTP, OAuth variants, additional folders, HTML,
attachments and any domain promotion each require their own capability/ADR and
evidence rather than silently broadening this slice.
