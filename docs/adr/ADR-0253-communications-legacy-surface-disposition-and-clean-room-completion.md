# ADR-0253: Communications legacy surface disposition and clean-room completion

Статус: Принято
Дата: 2026-07-23
Состояние реализации: backend disposition выполнен, repository cutover
продолжается. Канонический evidence owner, typed ingress, owner-local
projections, inbox/outbox, managed runtime и отдельный Attachment Security
Engine реализованы и допущены exact production inventory. Legacy HTTP surface
не является совместимым контрактом и не восстановлен в clean-room backend.
ADR-0240 пока не объявляется полностью завершённым на уровне репозитория:
secondary frontend всё ещё содержит старые `/api/v1/communications/*` callers,
которые должны быть удалены или заменены контрактами их настоящих owners.
Exact inventory сокращён до трёх production callers: dead AI-state,
bilingual-reply и Mail provider-command-diagnostics chains удалены вместе с
DTO, query/realtime и UI-обвязкой. Эти возможности остаются явными gaps своих
будущих owners/workflows и не реализуются внутри Communications.
Home и замороженная Timeline projection также больше не копируют Communications
DTO и не вызывают legacy owner routes; их frontend surfaces честно помечены
planned до отдельного admission вместо сохранения façade logic.

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
| attachment malware scanning and safety verdict | Attachment Security Engine under ADR-0273 | typed candidate/verdict events and evidence-bound Blob custody; no Communications implementation import or cross-owner SQL |
| SPF/DKIM, spam reputation, archive inspection, disarm, dedup and text extraction | future owner-specific security/content workflow selected by a later ADR | typed input/output and Blob lease; no cross-owner SQL |

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

The clean-room backend Communications owner is implemented and no longer
depends on a legacy surface. Repository completion is now bounded to removal
of the inventoried legacy frontend callers and validation of their generated
owner/integration replacements. Missing historical behaviour remains either an
explicit capability gap in its rightful owner or a blocked future decision; it
is never a reason to make Communications a facade for another domain.
