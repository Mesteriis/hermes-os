# Hermes Documentation

This directory contains product, foundation, architecture, domain, ADR and
implementation-status documentation for Hermes Hub.

Hermes documentation has one active product model:

```text
Hermes is a local-first Personal Memory System.
```

Communication is the primary ingestion spine, but not the only source of
evidence.

Styled documentation portal:

- [Hermes Hub Documentation](https://mesteriis.github.io/hermes-os/) - GitHub
  Pages entrypoint using the Hermes shell design language.

## Reading Order

New developers and agents should read in this order:

1. [Product Master Spec](product/master-spec.md)
2. [Foundation Vision](foundation/vision.md)
3. [Glossary](foundation/glossary.md)
4. [World Model](foundation/world-model.md)
5. [Product Development Roadmap](product/development-roadmap.md)
6. [Domain Map](foundation/domain-map.md)
7. [Architecture Overview](architecture/architecture-overview.md)
8. [ADR Index](adr/README.md)

## Canonical Sources

Canonical active vocabulary is defined in:

- [Foundation Vision](foundation/vision.md)
- [Glossary](foundation/glossary.md)
- [World Model](foundation/world-model.md)
- [Engines](foundation/engines.md)
- [Architecture Principles](foundation/architecture-principles.md)
- [Domain Map](foundation/domain-map.md)

If another document conflicts with these files, prefer the foundation documents
unless a newer ADR explicitly supersedes them. For code-structure and
architecture-boundary conflicts, ADRs are the source of truth.

## Product Documents

- [Product Master Spec](product/master-spec.md) - product-level source of truth.
- [Product Charter](product/product-charter.md) - purpose, user and quality bar.
- [Product Scope](product/product-scope.md) - in-scope and out-of-scope product areas.
- [Product Development Roadmap](product/development-roadmap.md) - future slices and refactoring plans.
- [Product Alignment Refactoring Plan](refactoring/product-alignment-plan.md) - current target-vs-implementation gaps and follow-up plans.
- [Implementation Alignment Plan](refactoring/implementation-alignment-plan.md) - code/module/schema/UI gaps against the canonical model.
- [Canonical Evidence Final Report](../canonical-evidence-final-report.md) - active current-period implementation status, progress table and next slices.

Historical roadmap files live under [roadmap](roadmap/). They describe past or
versioned implementation milestones and may use compatibility terminology.

## Foundation Documents

- [Foundation Vision](foundation/vision.md)
- [World Model](foundation/world-model.md)
- [Glossary](foundation/glossary.md)
- [Engines](foundation/engines.md)
- [Architecture Principles](foundation/architecture-principles.md)
- [Domain Map](foundation/domain-map.md)

## Domain Documents

Canonical domain specs live under [Domain Catalog](domains/README.md).

- [Signal Hub](domains/signal-hub/spec.md), [package](domains/signal-hub/README.md)
- [Communications](domains/communications/README.md)
- [Personas / Persona Intelligence](domains/personas/spec.md), [package](domains/personas/README.md)
- [Organizations](domains/organizations/spec.md), [package](domains/organizations/README.md)
- [Projects](domains/projects/README.md)
- [Documents](domains/documents/README.md)
- [Tasks](domains/tasks/spec.md), [package](domains/tasks/README.md)
- [Calendar And Events](domains/calendar/spec.md), [package](domains/calendar/README.md)
- [Decisions](domains/decisions/README.md)
- [Obligations](domains/obligations/README.md)
- [Review](domains/review/README.md)
- [Knowledge Graph](domains/graph/README.md)
- [Agents](domains/agents/README.md)
- [Notes Boundary](domains/notes/README.md)

Domain folders mirror `backend/src/domains/<domain>/` where possible. When a
package contains both canonical semantics and implementation details, the
canonical semantics live in `spec.md`.

## Integration Documents

Provider and channel docs live under [Integration Catalog](integrations/README.md).
Integrations are not product domains.

- [Mail](integrations/mail/README.md)
- [Telegram](integrations/telegram/README.md)
- [WhatsApp](integrations/whatsapp/README.md)
- [Zoom](integrations/zoom/README.md)
- [Yandex Telemost](integrations/yandex-telemost/README.md)
- [Ollama](integrations/ollama/README.md)
- [OmniRoute](integrations/omniroute/README.md)

## Engine Documents

The current engine map is in [Foundation Engines](foundation/engines.md). The
detailed engine catalog is in [Engine Catalog](engines/README.md).

- [Memory Engine](engines/memory/README.md)
- [Timeline Engine](engines/timeline/README.md)
- [Trust Engine](engines/trust/README.md)
- [Search Engine](engines/search/README.md), [architecture](engines/search/architecture.md)
- [Enrichment Engine](engines/enrichment/README.md)
- [Obligation Engine](engines/obligation/README.md)
- [Risk Engine](engines/risk/README.md)
- [Consistency / Contradiction Engine](engines/consistency/README.md),
  user-facing alias Polygraph.
- [Automation Engine](engines/automation/README.md)
- [Context Packs Engine](engines/context-packs/README.md)
- [Identity Resolution Engine](engines/identity-resolution/README.md)
- [Relationship Candidate Engine](engines/relationships/README.md)
- [Call Intelligence Engine](engines/call-intelligence/README.md)
- [Speaker Identity Engine](engines/speaker-identity/README.md)

Do not duplicate engine ownership inside domain documents.

## Code-Layer Documents

Documentation now follows the backend layer map from ADR-0073:

- [App Layer](app/README.md)
- [Application Services](application/README.md)
- [Domains](domains/README.md)
- [Engines](engines/README.md)
- [Integrations](integrations/README.md)
- [AI](ai/README.md)
- [Workflows](workflows/README.md)
- [Platform](platform/README.md)
- [Vault](vault/README.md)

## Workflow Documents

Workflow specs live in [Workflow Catalog](workflows/README.md).

- [Communication To Knowledge](workflows/communication-to-knowledge.md)
- [Communication To Obligation](workflows/communication-to-obligation.md)
- [Meeting To Decisions](workflows/meeting-to-decisions.md)
- [Document To Context](workflows/document-to-context.md)
- [Contradiction Review](workflows/contradiction-review.md)
- [Dossier Generation](workflows/dossier-generation.md)
- [Agent Assisted Recall](workflows/agent-assisted-recall.md)

## ADRs

Architecture Decision Records live in [adr](adr/).

ADRs are durable architectural decisions. Some older ADRs preserve historical
terms such as Contact or Person because implementation evolved over time. When a
newer ADR supersedes an older one, follow the newer ADR.

Important current examples:

- ADR-0001 - event sourcing is system spine.
- ADR-0008 - knowledge graph first.
- ADR-0022 - no fine-tuning on private data.
- ADR-0056 - current local API shared-secret guard.
- ADR-0055 - full email provider networking.
- ADR-0077 - Russian and English interface.
- ADR-0084 - Persona Intelligence System.
- ADR-0085 - Communication spine and Consistency / Contradiction Engine.
- ADR-0091 - Telegram production client capability model.
- ADR-0095 - event-driven domain communication and DLQ.
- ADR-0099 - Signal Hub event platform.
- ADR-0102 - accepted Zoom provider runtime boundary.
- ADR-0104 - proposed Yandex Telemost provider runtime boundary.

## Implementation Status Documents

Status and API files describe the current implementation. They are useful but
not always canonical product language.

Examples:

- `docs/integrations/mail/status.md`
- `docs/domains/calendar/status.md`
- `docs/domains/tasks/status.md`
- `docs/domains/personas/status.md`
- `docs/domains/*/api.md`
- `docs/integrations/*/api.md`

Root-level current-period status/reporting is centralized in
`canonical-evidence-final-report.md`. Domain status files remain bounded-context
implementation notes, not the primary report for the current refactor wave.

Current code/documentation alignment notes are tracked in
[Documentation Code Alignment Report](refactoring/documentation-code-alignment-report.md).

If a status document mentions compatibility terms such as `persons`,
`person_id`, `health`, `watchlist`, historical `contacts` naming or
`follow-up`, interpret them through the Product Master Spec and foundation
glossary.

## Historical Documents

Historical reviews under `docs/reviews/` and version closure files under
`docs/roadmap/` are traceability records unless a current product, foundation,
architecture or ADR document explicitly references them as active requirements.
