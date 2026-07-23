# ADR-0257: Event-backed Blob custody transfer for canonical evidence

Статус: Принято
Дата: 2026-07-23
Состояние реализации: Реализованы source slice и managed runtime conformance.
The fixture integration writes a Blob reference under its own producer
registration and publishes only a typed opaque receipt. Communications persists
the receipt in a private leased work queue, asks Kernel for an evidence-bound
target custody grant, and commits the target-owned receipt only after Blob
Platform completes its internal rewrap. Communications derived search may read
only that Communications-owned Blob reference. A direct cross-owner Blob read
is rejected by the current registration/capability fence and must remain
rejected. Managed conformance covers altered receipts, stale source and target
launches, and revoked source registrations. Blob/Vault unavailability and target-grant
revocation remain separate required evidence; a nominal managed conformance
pass does not replace them.

Зависит от:

- [ADR-0201: durable event spine](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0220: durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob ownership and opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0231: private Blob data sessions](ADR-0231-private-blob-data-session-and-vault-route.md);
- [ADR-0240: Communications canonical owner](ADR-0240-canonical-communications-owner-clean-room-migration.md);
- [ADR-0254: derived Communications search](ADR-0254-communications-derived-search-index-and-private-content-boundary.md).

## Context

An integration is the only owner that sees provider plaintext. It obtains a
bounded write session from Blob Platform and emits a typed Communications
observation containing an opaque body receipt. The canonical evidence owner
must subsequently index admitted content without importing the integration or
receiving provider bytes.

The existing Blob fence intentionally binds a reference to its producer
registration/capability. Therefore an integration-owned reference cannot be
read by Communications, even when the logical human owner is the same. Giving
Communications the integration capability, exposing an integration socket, or
mounting Blob storage would violate capability isolation and owner autonomy.

## Decision

Blob Platform gains one explicit event-backed custody-transfer operation. It is
a platform operation, not an integration RPC, Communications facade, shared
filesystem or business-domain API.

```text
integration runtime
  -> own Blob write session
  -> exact typed observation receipt in integration outbox
  -> durable event spine
  -> Communications inbox + canonical evidence transaction
  -> owner-local Blob custody-transfer request
  -> Blob Platform internal rewrap/rebind
  -> Communications-owned opaque receipt
  -> owner-local derived index job
```

The transfer request contains only:

- exact consumed observation/evidence identity and its durable-message hash;
- source opaque reference ID, declared size and plaintext digest from the typed
  receipt;
- source registration/capability/runtime/grant fence derived from the admitted
  event source, never supplied as a free-form client value;
- target Communications registration/capability/runtime/grant fence;
- logical owner scope and an idempotency key derived from the evidence ID.

Blob Platform verifies the source fence, target current grant, logical owner
scope, receipt size/digest and durable evidence binding before touching content.
It atomically creates a new target-owned opaque reference and rewraps/rebinds
encrypted content internally. Plaintext bytes, paths, provider identifiers,
keys and generic metadata never cross the event, Kernel or Communications
boundary. The source reference remains subject to producer retention until its
own owner marks it eligible for deletion; a transfer never implicitly deletes
it.

Communications persists only the target receipt returned by the platform before
creating an index job. A duplicate event or transfer retry returns the same
target receipt. A missing, revoked, expired, altered or wrong-owner source is a
typed owner-local admission/index failure and never falls back to a
cross-owner read.

The private work queue is owned by Communications persistence. It has an
expiring worker lease, terminal completion/rejection states and no public query
surface. Blob unavailability leaves the item pending for a later fenced retry;
a policy rejection atomically exposes the canonical `Unavailable` state with
the typed `PolicyRejected` failure, and still creates neither a Blob receipt
nor an index job.

## Non-goals

- generic Blob sharing, read-all grants or arbitrary recipient selection;
- provider payload in NATS, Kernel control frames or public APIs;
- direct integration-to-Communications RPC, cross-owner SQL or filesystem
  access;
- transfer of provider session stores or attachments outside an explicitly
  admitted canonical evidence receipt;
- automatic migration of historical Blob data.

## Required evidence

1. A real admitted integration body write produces a source-owned opaque
   reference and durable typed observation.
2. Communications consumes the exact event, performs one idempotent transfer,
   and indexes the resulting target-owned reference.
3. Public search returns canonical IDs only; neither source nor target Blob
   locator, provider identifier or plaintext appears in events, index rows or
   query response.
4. Replay, revoked source/target grants, stale generations, altered receipt
   digest and unavailable Blob/Vault all fail closed without an alternate read
   path.
5. Architecture tests prove that integrations retain only ingress/event edges
   and Communications never imports integration implementation or Blob storage.

## Consequences

The current synthetic admitted-body fixture proves receipt metadata only. It
does not prove search indexing and must not be used to claim complete
Communications Blob conformance. The existing `communications.blob.v1`
capability remains read/derived-index authority only after successful custody
transfer; producer write authority remains integration-owned.
