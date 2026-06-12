# Organizations Architecture

## Position

The Organizations domain owns Organization entities and organization-specific
identity, relationships and operational memory. It uses shared engines for
timeline, memory, trust, enrichment, risk and search.

## Modules

Paths below refer to `backend/src/domains/organizations/`.

| Module | Responsibility |
|---|---|
| `core.rs` | Organization core, store, identities, aliases, domains, departments, Persona links and related organizations |
| `memory.rs` | facts, memory cards, preferences, required documents and memory decay inputs |
| `workflows.rs` | portals, procedures, playbooks and workflow records |
| `finance.rs` | financial information, contracts, compliance, services and products |
| `enrichment.rs` | Enrichment Engine candidates and review state |
| `health.rs` | Risk Engine/attention read models |
| `investigator.rs` | Dossier/context assembly read models |
| `api.rs` | current route handlers and DTO-facing compatibility surface |

## Data Flows

### Organization Creation From Communication

```text
incoming Communication
  -> source/domain evidence
  -> Organization candidate or upsert
  -> Organization identity/domain record
  -> Relationship to Persona when evidence supports it
```

### Identity Resolution

```text
organizations with similar names/domains/VAT
  -> candidate
  -> owner confirm/reject
```

### Enrichment

```text
approved sources
  -> Enrichment Engine
  -> organization enrichment candidate
  -> owner/policy review
  -> organization facts, identities or relationships
```

## ADR

| ADR | Topic |
|---|---|
| 0061 | Organization as first-class entity |
| 0062 | Identity and resolution |
| 0063 | Passive OSINT boundary |
| 0064 | Memory and provenance |
| 0065 | Portals, procedures, playbooks |
| 0066 | Graph integration |
