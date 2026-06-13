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
unless a newer ADR explicitly supersedes them.

## Product Documents

- [Product Master Spec](product/master-spec.md) - product-level source of truth.
- [Product Charter](product/product-charter.md) - purpose, user and quality bar.
- [Product Scope](product/product-scope.md) - in-scope and out-of-scope product areas.
- [Product Development Roadmap](product/development-roadmap.md) - future slices and refactoring plans.
- [Product Alignment Refactoring Plan](refactoring/product-alignment-plan.md) - current target-vs-implementation gaps and follow-up plans.
- [Implementation Alignment Plan](refactoring/implementation-alignment-plan.md) - code/module/schema/UI gaps against the canonical model.
- [Documentation Completion Audit](refactoring/completion-audit.md) - requirement coverage, implementation evidence and remaining refactoring work.

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

- [Communications](domains/communications.md)
- [Telegram Channel Capability Spec](domains/telegram-channel.md)
- [Personas / Persona Intelligence](persons/README.md)
- [Organizations](domains/organizations.md)
- [Projects](domains/projects.md)
- [Documents](domains/documents.md)
- [Tasks](domains/tasks.md)
- [Calendar And Events](domains/calendar-events.md)
- [Decisions](domains/decisions.md)
- [Obligations](domains/obligations.md)
- [Knowledge Graph](domains/knowledge-graph.md)
- [Agents](domains/agents.md)
- [Notes Boundary](domains/notes.md)
- [Search Architecture](domains/search-architecture.md)

Dedicated historical or implementation-heavy folders still exist for Personas,
Organizations, Tasks, Mail and Calendar. When they conflict with canonical
domain specs, treat the conflict as a migration gap and update the relevant
refactoring plan before changing code.

## Engine Documents

The current engine map is in [Foundation Engines](foundation/engines.md). The
detailed engine catalog is in [Engine Catalog](engines/README.md).

- [Memory Engine](engines/memory.md)
- [Timeline Engine](engines/timeline.md)
- [Trust Engine](engines/trust.md)
- [Search Engine](engines/search.md)
- [Enrichment Engine](engines/enrichment.md)
- [Obligation Engine](engines/obligation.md)
- [Risk Engine](engines/risk.md)
- [Consistency / Contradiction Engine](engines/consistency-contradiction.md),
  user-facing alias Polygraph.

Do not duplicate engine ownership inside domain documents.

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

## Implementation Status Documents

Status and API files describe the current implementation. They are useful but
not always canonical product language.

Examples:

- `docs/mail/status.md`
- `docs/calendar/status.md`
- `docs/tasks/status.md`
- `docs/persons/status.md`
- `docs/*/api.md`

If a status document mentions compatibility terms such as `persons`,
`person_id`, `health`, `watchlist`, historical `contacts` naming or
`follow-up`, interpret them through the Product Master Spec and foundation
glossary.

## Historical Documents

Historical plans and specs under `docs/superpowers/` are implementation history.
They can explain why something was built, but they are not the current product
model.

Use them for traceability, not vocabulary authority.

Historical reviews under `docs/reviews/` and version closure files under
`docs/roadmap/` are also traceability records unless a current product,
foundation, architecture or ADR document explicitly references them as active
requirements.
