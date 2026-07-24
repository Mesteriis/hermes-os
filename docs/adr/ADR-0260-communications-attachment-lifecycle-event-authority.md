# ADR-0260: Communications attachment lifecycle event authority

Статус: Принято
Дата: 2026-07-24
Состояние реализации: В работе. Owner-local CAS/canonical outbox, оба exact
inbound contract, отдельные consumer capability и fail-closed typed permit set
реализованы. Реальный producer admission и live conformance ещё не выполнены.
ADR-0246 определил owner-local attachment state machine, но не зафиксировал
exact producer authority и contracts для её terminal external facts. До
отдельного admission ни один producer не получает право изменять attachment
safety state через generic observation или direct storage.

Зависит от:

- [ADR-0201: Core module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0220: canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob Platform](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0246: Communications attachment admission and safety](ADR-0246-communications-attachment-admission-and-safety.md).

## Decision

Communications receives attachment lifecycle facts only as two distinct typed
durable observations. They are separate contracts, capabilities and Event Hub
subjects; a payload field never declares or grants its producer authority.

1. `communication_attachment_blob_admission_observed.v1` is published only by
   the integration or explicit workflow that owns the provider-local download
   and completed a Blob platform admission. It may request or confirm only
   `descriptor_only -> blob_pending`, `blob_pending -> blob_admitted`, or a
   terminal `rejected` result. The payload contains the canonical attachment
   anchor ID, expected prior state, terminal transition, evidence ID, observed
   time, and bounded opaque Blob reference/integrity binding where required.
2. `communication_attachment_safety_verdict_observed.v1` is published only by
   a separately admitted security/content engine. It may confirm only
   `blob_admitted -> safe_for_delivery`, `descriptor_only|blob_pending|blob_admitted
   -> quarantined`, or a terminal `rejected` verdict. It never carries content,
   provider locator, scanner implementation detail, generic labels, map or
   verdict explanation.

Communications owns both consumer capabilities, validates the closed
transition against its current anchor state, performs compare-and-set in its
own PostgreSQL transaction, and publishes a canonical owner event through its
existing outbox. It does not open a Blob data session, invoke a scanner, or
import an integration or engine package while consuming either observation.

The current exact first-owner package inventory remains unchanged; the
Communications descriptor adds only its two consumer capabilities. Adding a producer
requires a separate admission decision for that integration, workflow or
engine, including its own package inventory, Event Hub publish route and live
conformance. A producer cannot become part of the Communications domain merely
because it writes an attachment result.

## Rejected alternatives

- A single `attachment_status` observation with a producer-kind field: the
  field is data, not authorization, and would let an integration forge a clean
  scanner verdict.
- Direct Blob/scanner calls from Communications: this would couple a domain to
  platform/engine implementation and make the runtime an integration facade.
- A generic attachment command through Gateway: attachment admission and
  verdict are external observed facts, not client-owned business mutations.

## Required implementation and evidence

1. Add both schema-bound contracts and separate Communications consumer
   capabilities; the runtime must bind them by exact contract/permit rather
   than subject string selection.
2. Replace the single observation permit assumption with a bounded typed
   permit set and independently ACK each durable delivery only after the
   owner-local mutation/outbox transaction commits.
3. Add owner-local persistence and canonical outbox events for successful CAS
   transitions; stale, malformed, cross-contract and unauthorized transitions
   must not mutate state and must not ACK a retryable infrastructure failure.
4. Admit one real producer per contract in separate slices and prove live
   Blob-result and scanner-verdict flows, replay, stale generation, grant
   revoke and conflict handling.
5. Keep `safe_for_delivery` unreachable until the separately admitted security
   engine producer exists. No fallback or implied-clean state is allowed.

## Consequences

ADR-0246 can progress without turning Communications into a Blob or security
owner. Attachment anchors remain provider-neutral canonical evidence, while
integration download state, Blob bytes and scanner policy remain in their
respective assembly units.
