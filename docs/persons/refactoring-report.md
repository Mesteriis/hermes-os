# Persons Domain Refactoring Report

Date: 2026-06-12

## Scope

Audited documents:

- `docs/persons/README.md`
- `docs/persons/architecture.md`
- `docs/persons/data-model.md`
- `docs/persons/api.md`
- `docs/persons/status.md`
- `docs/persons/blockers.md`

Relevant ADR reviewed:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0019 Contact Identity Resolution
- ADR-0022 No Fine Tuning on Private Data
- ADR-0045 Graph Core Projection
- ADR-0057 Person Memory and Provenance
- ADR-0058 Person Enrichment Engine
- ADR-0059 Person Communication DNA and Personas
- ADR-0060 Person Timeline and Graph Integration
- ADR-0061 Organization as First-Class Domain Entity
- ADR-0074 Person Multi-Channel Identity Model

## Executive Summary

The existing `docs/persons/` documentation was a partially renamed Contact CRM
model. It had already moved from `contact` to `person` naming, but the
architecture still treated people as enriched contact records with roles,
favorite flags, notes, watchlists, health status, fingerprints and analytics.

That conflicts with the new Hermes direction:

```text
Personal Memory System
Persona Intelligence
Relationship first
Memory first
```

The documentation must now use Persona as the main entity and treat current
`persons` tables/routes as transitional compatibility details.

## File Audit

| File | Old model found | Required change |
|---|---|---|
| `README.md` | Presented module as implemented Relationship Intelligence with endpoint/table metrics and CRM-like feature list. | Rewritten as Persona Intelligence domain vision. |
| `architecture.md` | Listed backend modules around fingerprints, health, investigator, analytics, roles and nested personas. | Rewritten around Persona, Identity Resolution, Relationship, Memory, Timeline, Dossier and Persona Intelligence boundaries. |
| `data-model.md` | Described `persons` as renamed `contacts`; root email, trust, favorite, notes, role, organization and health fields. | Rewritten as target logical model with compatibility mapping. |
| `api.md` | Exposed `/persons` endpoints for favorite, notes, roles, nested personas, health, watchlist, fingerprint and investigator. | Rewritten as compatibility notes. No target Persona API is designed in this documentation pass. |
| `status.md` | Claimed completion against old spec sections such as Contact Merge, Contact Roles and Contact Personas. | Rewritten as refactoring/migration status. |
| `blockers.md` | Claimed no blockers for backend/API scope. | Rewritten with current blockers for Persona architecture. |

## Contradictions Found

### Contact and CRM Residue

- `persons` was described as a renamed `contacts` table.
- ADR-0019 and current docs used Contact Identity Resolution language.
- Status used Contact Merge, Contact Roles, Contact Personas, Contact Sources,
  Contact Snapshots and Contact Inbox.
- API exposed favorite, notes and watchlist behavior that reads like a contact
  manager.

### Persona Misuse

- `person_personas` modeled personas as nested interaction contexts under a
  person.
- ADR-0059 used Personas to mean communication contexts, not subjects.
- This conflicts with Persona as the root domain entity.

### Relationship Stored as Fields

- `trust_score`, `primary_role`, `organization_reference`, `health_status`,
  `communication_gap_days` and `watchlist` were modeled on the person root.
- Relationship timeline existed, but first-class Relationship records with
  `source_persona`, `target_persona`, `trust_score` and `strength_score` were
  missing.

### Memory Partly Correct but Contact-Framed

- Facts, memory cards, preferences and conflicts are aligned with the new model.
- Their documentation framed them as person/contact profile enrichment rather
  than Persona memory.
- Notes existed as root text instead of cited memory.

### Dossier and Intelligence Fragmentation

- `fingerprint`, `communication profile`, `trust`, `analytics` and
  `investigator` appeared as separate features.
- The target model requires one Persona Intelligence concept with derived,
  cited observations and a Dossier read model.

### Identity Resolution Too Contact-Centric

- Identity resolution was framed as duplicate contact merge/split.
- Target identity resolution must merge digital traces into Personas:
  email, phone, Telegram, WhatsApp, GitHub, LinkedIn, documents and messages.

### Self Persona Missing

- No document defined the Owner Persona.
- No document enforced exactly one `Persona.is_self = true`.
- No document stated that agents act through the Owner Persona.

### AI Agent Personas Missing

- The old model assumed people only.
- It did not allow HESTIA or future agents to exist as graph Personas.

## Entities to Rename

| Old entity/term | New entity/term |
|---|---|
| Person | Persona, when referring to the domain root |
| Contact | Persona or PersonaIdentity, depending on context |
| Contact Identity Resolution | Persona Identity Resolution |
| `person_type` | `persona_type` |
| Communication Fingerprint | Persona communication patterns |
| Communication Profile | Persona communication intelligence |
| Person Investigator | Dossier assembler / Persona Intelligence workflow |
| Person Analytics | Persona Intelligence read model |
| Relationship Health | Relationship attention signals |
| Contact Roles | Relationships |
| Contact Personas / `person_personas` | Deprecated; interaction context or preferences if still needed |

## Entities to Remove From Target Domain Semantics

- Contact.
- CRM profile.
- Address-book entry.
- Favorite as Persona identity.
- Watchlist as Persona identity.
- Free-text notes as source-of-truth memory.
- `trust_score` on Persona root.
- `primary_role` on Persona root.
- `organization_reference` as a source-of-truth field.
- Nested Persona records under a person.
- Health status as a canonical relationship model.

These may remain temporarily as compatibility fields, but new docs and future
implementation work must not treat them as target primitives.

## Entities to Add

- `Persona`
- `PersonaType`
- `Self Persona`
- `PersonaIdentity`
- `IdentityResolutionCandidate`
- `Relationship`
- `PersonaMemory`
- `PersonaFact`
- `PersonaKnowledgeItem`
- `PersonaPreference`
- `PersonaMemoryCard`
- `PersonaKnowledgeConflict`
- `PersonaDatedEvent`, consumed by the shared Timeline Engine
- `PersonaCommunicationPattern`
- `PersonaDossier`
- `PersonaIntelligenceObservation`

## ADR Impact

ADR-0084 is required because the request changes the domain model in a way that
conflicts with existing ADR language.

ADR-0084 should:

- introduce Persona Intelligence as the domain model;
- supersede ADR-0019 Contact Identity Resolution;
- supersede ADR-0059 Person Communication DNA and Personas;
- clarify ADR-0057, ADR-0058, ADR-0060 and ADR-0074;
- keep ADR-0074 implementation compatibility until a separate schema/API
  migration exists.

## Implementation Impact

This documentation refactoring does not implement schema or API migrations.
Expected future implementation work:

- add `is_self` Owner Persona semantics;
- add target `PersonaType` values;
- add first-class Relationship records;
- keep retiring compatibility table/route names after the current
  `person_roles` Relationship, `person_personas` Preference, trust Relationship
  and notes Memory Card adapter baselines;
- introduce Dossier read model with source references;
- document an explicit route/schema compatibility strategy before any future
  implementation migration.

## Decision

`docs/persons/` is now treated as target-domain documentation. Legacy backend
names are documented only as compatibility notes. Future changes should optimize
for Persona Intelligence coherence over compatibility with the old Contact CRM
wording.
