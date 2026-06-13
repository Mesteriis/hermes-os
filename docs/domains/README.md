# Hermes Domain Catalog

This catalog is the canonical entry point for active Hermes domains. It should
be read together with:

- [Product Master Spec](../product/master-spec.md)
- [World Model](../foundation/world-model.md)
- [Glossary](../foundation/glossary.md)
- [Domain Map](../foundation/domain-map.md)

Hermes is a local-first Personal Memory System. Domains own source-of-truth
entities. Engines build derived memory, context, scores, timelines and
recommendations from those entities.

## Domain Rule

A domain exists when Hermes needs a durable source of truth for an entity type.
A domain does not exist merely because the UI has a page or because an engine
needs a projection.

## Canonical Domains

| Domain | Canonical document | Status |
|---|---|---|
| Communications | [Communications](communications.md) | implemented with email-heavy naming and Telegram/WhatsApp surfaces |
| Personas | [Personas](persons.md), [Persona Intelligence](../persons/README.md) | partially implemented through `persons` and compatibility migrations |
| Relationships | [Relationships](relationships.md) | partially implemented through first-class persistence, graph projection for all current Relationship entity kinds, guarded global suggested review, organization contact link adapters, task relation adapters, project link review adapters and Personas workspace review; remaining person-role/cross-domain inbox work incomplete |
| Organizations | [Organizations](organizations.md), [Organizations Folder](../organizations/README.md) | implemented as a memory anchor domain |
| Projects | [Projects](projects.md) | implemented, needs stronger domain documentation |
| Documents | [Documents](documents.md) | implemented, needs clearer Knowledge boundary |
| Tasks | [Tasks](tasks.md), [Tasks Folder](../tasks/README.md) | implemented, needs stronger Obligation boundary |
| Calendar/Events | [Calendar And Events](calendar-events.md) | implemented under calendar/calls/meetings surfaces |
| Decisions | [Decisions](decisions.md) | partially implemented through first-class persistence, accepted graph projection, guarded entity/global API, explicit message/imported-document candidate refresh, email plus Telegram/WhatsApp fixture ingestion refresh, meeting outcome adapter, project link review adapter and Tasks workspace review; adapters incomplete |
| Obligations | [Obligations](obligations.md) | partially implemented through first-class persistence, accepted graph projection, guarded entity/global API, obligation-derived task-candidate review-state synchronization, document candidate refresh, email plus Telegram/WhatsApp fixture ingestion refresh, person promise adapter, meeting outcome adapter and Tasks workspace review; adapters incomplete |
| Knowledge Graph | [Knowledge Graph](knowledge-graph.md) | implemented as graph domain/projection substrate |
| Agents | [Agents](agents.md) | partially implemented through AI runtime/control surfaces |
| Notes | [Notes](notes.md) | not a first-class domain; treated as document-like artifacts |

## Engine Documents

Engines are not domains. Engine ownership lives in
[Engines](../foundation/engines.md). Search-specific details live in
[Search Engine Architecture](search-architecture.md).

## Current Implementation Caveats

The repository still contains historical naming and compatibility boundaries:

- `mail` backend modules implement much of the Communications capability.
- `person_id` and `/api/v1/persons/*` are current implementation names, while
  new domain language is Persona.
- `contacts` appears in migration history because the persistence model evolved
  from contact records into Personas.
- Notes have a frontend surface but no ADR that promotes them to a first-class
  domain.
- Decisions and Obligations are canonical product domains. Both have initial
  persistence baselines, accepted graph projection and guarded backend APIs.
  Decisions have explicit message/imported-document candidate refresh, a
  meeting `decision` outcome adapter, Telegram/WhatsApp fixture ingestion
  refresh and a project link review adapter.
  Relationships have organization contact link, task relation and project link
  review adapters.
  Obligations have review-state synchronization for obligation-derived task
  candidates, document commitment candidate refresh, email-sync plus
  Telegram/WhatsApp fixture ingestion candidate refresh, a person promise
  adapter and a meeting `promise`/`task`/`follow_up` outcome adapter. Desktop
  UI, live-provider ingestion and remaining compatibility adapters remain
  incomplete.

These caveats are not new terminology. They are migration facts that must be
resolved by implementation plans and ADRs before code-level renames or schema
changes.
