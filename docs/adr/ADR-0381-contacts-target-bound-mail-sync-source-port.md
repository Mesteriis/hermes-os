# ADR-0381: Contacts target-bound Mail sync source port

Статус: Принято

Дата: 2026-08-02

Состояние реализации: статически реализованы contract unit, Contacts-owned
persistence/materialization и workflow forwarding до Mail-owned upsert command.
Mail provider adapter, обработка terminal results и managed Blob/provider
conformance ещё не реализованы, поэтому `mail_contacts_sync_v1` остаётся
`planned`.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0257](ADR-0257-event-backed-blob-custody-transfer-for-canonical-evidence.md);
- [ADR-0379](ADR-0379-mail-address-book-sync-and-contacts-command-boundary.md).

## Контекст

ADR-0379 требует для Contacts-to-Mail направления bounded changed event и
отдельный target-bound Blob handoff. Пять исходных Contacts units не содержат
публичного source-port контракта. Добавление export messages в
`hermes-contacts-command-api` смешало бы mutation authority и source custody,
а перенос contact fields в durable event раскрыл бы business data Event Hub.

## Решение

Добавляется шестая Contacts-owned unit
`hermes-contacts-mail-sync-source-api`. Она владеет только публичным source port
для `mail_contacts_sync`:

- `ContactChangedForMailSyncV1` — bounded event с opaque `contact_id`, exact
  `contact_revision` и logical owner;
- `PrepareContactMailSyncSourceCommandV1` — exact workflow command с operation,
  contact revision и opaque target Mail account;
- prepared/rejected terminal results;
- exact plaintext `ContactMailSyncSourceContentV1`, который никогда не входит в
  durable envelope и доступен только через Blob custody, привязанную к
  `hermes-mail-runtime`.

Contacts runtime материализует snapshot из Contacts-owned storage и пишет Blob
только для exact Mail target owner/module/capability. Workflow проверяет только
bounded receipt и выпускает Mail-owned provider command, не читая private Blob.
Mail runtime принимает custody, проверяет и читает exact bytes. Mail account
разрешает provider; Contacts и workflow не выбирают provider по identity.

`target_mail_account_id` является opaque routing identity. Он нужен Contacts
только для выбора уже существующей Mail provenance link и не выдаёт Contacts
provider credential или operational authority. Snapshot может содержать только
bounded presentation fields и optional link entry/ETag для этого account.

## Инварианты

- changed event не содержит name, email, phone, provider kind, entry ID или
  ETag;
- command/result subjects и sanitized errors не содержат private identifiers;
- stale contact revision отклоняется до Blob write;
- source command резервируется до Blob write, а завершённый replay возвращает
  сохранённый exact terminal result без повторной materialization;
- source result сохраняется в Contacts inbox/outbox до Ack;
- duplicate command replay возвращает тот же exact terminal result;
- Blob custody target exact и не является generic export/read capability;
- integration, workflow и Contacts domain не импортируют implementations или
  storage друг друга.

## Gate

Contract slice считается статически реализованным после отдельного Cargo
package, generated Protobuf, exact envelope builders, compile isolation и
architecture tests. Полный source-port slice требует Contacts persistence,
runtime Blob adapter, workflow materialization, restart/revoke negatives и live
managed conformance.
