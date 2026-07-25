# ADR-0263: Mail integration settings and Storage admission artifacts

Статус: Принято
Дата: 2026-07-24
Состояние реализации: Реализованы versioned hidden settings schema, immutable
Mail-owned Storage bundle revision 2 с отдельным Attachment Security candidate
outbox и exact unsigned descriptor artifact. Descriptor содержит отдельную
optional scan-candidate publish capability; signed managed conformance
доказывает exact grant и durable publication, но production engine admission
остаётся отдельным gate.

Зависит от:

- [ADR-0215: module registration and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0222: settings registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0224: Storage Control](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0262: Mail attachment Blob admission](ADR-0262-mail-attachment-blob-admission-extension.md).

## Decision

Mail is an integration owner. Its endpoint configuration, provider account IDs
and credential revisions are `configuration_instance` settings, not
Communications domain state. The exact schema is versioned, hidden from generic
client reads, requires fresh owner proof, and applies only through module
restart. Passwords and access tokens never become settings values: a setting
contains only an approved credential revision and Mail obtains plaintext solely
through its scoped Vault lease.

Mail persistence exposes one immutable `mail_state` Storage bundle owned by
`mail`. It contains only Mail tables: its outbox/inbox, attachment-anchor
mapping, attachment Blob lifecycle, delivery attempts and provider sync state.
It does not contain Communications tables, foreign keys, SQL reads, or grants.

## Consequences

The exact descriptor artifact references this settings schema and requests only
the Mail Storage namespace, bounded Blob quota, three configuration-instance
Vault resolve purposes, and three typed ingress routes: two publishes
(`communication_observed`, attachment Blob-admission observation) and one
`communication_attachment_anchor_recorded` subscription. It provides no
Communications API, store, runtime or provider-neutral business surface.

Storage bundle, descriptor, owner approval, signed distribution digest and
runtime fencing are admitted atomically in a dedicated integration phase. This
ADR neither adds Mail to `first_owner_v1` nor changes the Communications owner
inventory.
