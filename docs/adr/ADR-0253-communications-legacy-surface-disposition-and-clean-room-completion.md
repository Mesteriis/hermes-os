# ADR-0253: Communications legacy surface disposition and clean-room completion

Статус: Принято
Дата: 2026-07-23
Состояние реализации: Gap analysis зафиксирован. Канонический evidence owner,
typed ingress, owner-local projections, inbox/outbox и managed runtime уже
реализованы. Этот ADR не считает legacy HTTP surface совместимым контрактом и
не объявляет ADR-0240 завершённым: каждый admitted capability требует typed
contract, owner-local implementation и evidence своего пути.

Зависит от:

- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0207: domain registry](ADR-0207-canonical-business-domain-registry.md);
- [ADR-0212: compile isolation](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: ownership and SRP](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220: durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0226: AI workflows](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0240: Communications owner](ADR-0240-canonical-communications-owner-clean-room-migration.md);
- [ADR-0252: first owner admission](ADR-0252-first-owner-v1-communications-admission.md).

## Context

The historical `domains/communications` module and `/api/v1/communications/*`
routes mix canonical communication evidence with provider execution, AI,
finance, persona, document, review and workflow concerns. Recreating that
surface inside the new Communications owner would violate owner autonomy and
would reproduce the legacy facade under a different transport.

Historical code is behavioural evidence only. A legacy name, route or table is
not a clean-room contract, and no old REST path, DTO, database schema, fallback
or dual-write is admitted.

## Decision

`communications` owns only provider-neutral communication evidence and the
canonical read model derived from it:

- evidence intake, deterministic source identity and lifecycle transitions;
- accounts, conversations, messages, participants, reply/forward references
  and attachment anchors;
- canonical metadata queries and evidence-backed local state that does not
  select provider behaviour;
- owner-local inbox, outbox, replay and audit correlation.

Every remaining historical capability has exactly one disposition:

| Historical capability class | Clean-room disposition | Required boundary |
|---|---|---|
| raw records, ingestion, message/thread reads, canonical search, attachment anchors | Communications owner | typed owner API and owner-local PostgreSQL only |
| IMAP/Gmail/Telegram/WhatsApp/Zulip sync, folders, provider cursors, provider delivery/read state, subscriptions, provider diagnostics, retry and send execution | owning integration | provider operational contract and integration outbox event |
| reply/forward/redirect, bulk provider actions and cross-channel intent | explicit workflow | evidence-backed workflow command to one provider operational contract |
| AI reply, language/translation, extraction, explainability and message analysis | explicit use-case workflow or AI owner | ADR-0226 owner queries plus `AiContextReceiptV1` |
| persona, organization, relationship and graph promotion | target domain workflow | evidence reference and target-domain command/event |
| invoices, finance analytics and finance explanations | Finance owner when admitted | evidence-backed Finance workflow; never Communications storage |
| legal documents, exportable document artifacts and certificates | Documents owner when admitted | explicit document/export workflow and Blob reference |
| review state, candidates, pin/snooze/mute and attention decisions | Review owner when admitted | evidence-backed review workflow; no hidden Communications projection |
| templates, signatures and rich composition | provider integration or dedicated composition owner, selected by a later ADR | no generic Communications provider command |
| SPF/DKIM, spam reputation, archive inspection, disarm, dedup and text extraction | owner-specific security/content workflow, selected by a later ADR | typed input/output and Blob lease; no cross-owner SQL |

Rows marked “when admitted” are not silently implemented in Communications
while their owner gate is closed. Rows requiring a later ADR stay historical
behaviour evidence until that decision exists.

## Required migration order

1. Complete the canonical Communications read model and public generated owner
   contracts for the first admitted evidence slice.
2. Complete each integration operational contract independently and publish
   only exact typed ingress envelopes to Communications.
3. Add a separate workflow/owner ADR before any capability in the workflow,
   AI, Finance, Documents, Review or security/content rows is implemented.
4. Remove a historical capability only after its clean-room owner has its
   contract, runtime route, regression coverage and migration evidence.

No compatibility facade is permitted during this sequence. The absence of a
clean-room replacement is reported as an open migration gap, not hidden behind
a legacy route or proxy.

## Completion evidence

ADR-0240 may be marked implemented only when:

1. every capability in the Communications-owner row has a typed contract,
   owner-local implementation and regression evidence;
2. every adopted integration capability reaches Communications exclusively by
   typed ingress and durable events;
3. every adopted non-Communications row has an approved owner/workflow ADR and
   no Communications import, SQL access or runtime call;
4. no production source, generated client, runtime descriptor or frontend path
   references legacy REST, schemas, DTOs, facades, aliases or `references/`;
5. managed runtime, public owner query and relevant integration paths have
   live conformance evidence.

## Consequences

The migration remains broader than the current evidence owner implementation,
but it is no longer ambiguous. Missing historical behaviour is either an
explicit capability gap in its rightful owner or a blocked future decision; it
is never a reason to make Communications a facade for another domain.
