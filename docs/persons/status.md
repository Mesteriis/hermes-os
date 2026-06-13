# Persons — Persona Refactoring Status

This status document tracks the documentation and implementation migration from
the legacy Contact/Person model to the target Persona Intelligence model.

It intentionally does not preserve the old "implemented sections" scorecard,
because that scorecard measured a CRM-shaped specification that is no longer the
domain target.

## Documentation Status

| Area | Status | Notes |
|---|---|---|
| Domain vision | Updated | Persona Intelligence replaces Contact/CRM framing. |
| Architecture | Updated | Persona, Relationship, Memory, Dossier, Self Persona and Timeline Engine use are defined. |
| Data model | Updated | Target logical model documented with compatibility mapping. |
| API | Updated | Target `/personas` API shape documented; `/persons` marked legacy compatibility. |
| Gap analysis | Added | See `refactoring-report.md`. |
| ADR | Added | ADR-0084 records the domain decision. |

## Current Implementation Compatibility

The backend currently contains implementation pieces that can be reused, but they
do not yet fully implement the target model.

| Current artifact | Status against target |
|---|---|
| `persons` table | Transitional Persona projection, still rooted in email/contact history. |
| `person_identities` | Useful identity trace table; needs Persona naming and extra trace types. |
| `person_identity_candidates` | Compatible review workflow; contact/person language must become Persona language. |
| `person_roles` | Deprecated as standalone role storage; compatibility writes materialize first-class Relationships. |
| `person_personas` | Deprecated as nested Personas; compatibility writes materialize `interaction_context:*` Persona Preferences. |
| `relationship_events` | Useful timeline projection; not a first-class Relationship model. |
| `person_facts`, `person_memory_cards`, `person_preferences` | Compatible with Persona Memory after naming/provenance alignment. |
| `person_expertise` | Compatible as Persona skills/knowledge signals. |
| `person_promises`, `person_risks` | Must be reframed as cited facts, timeline events or observations. |
| `trust_score` | Compatibility cache; enrichment now materializes suggested Owner Persona trust Relationships. |
| `notes` | Compatibility cache; writes now materialize sourced Persona Memory Cards. |
| `is_favorite` | Compatibility cache; writes now materialize sourced `ui:favorite` Persona Preferences. |
| `watchlist` | Compatibility cache; writes now materialize sourced `ui:watchlist` Persona Preferences. |
| `health_status` | Compatibility cache; `PersonRisk` report/resolve now derives it from unresolved risk observations. |
| `/api/v1/persons/*` | Legacy compatibility API. |

## Target Migration Slices

| Slice | Status | Required outcome |
|---|---|---|
| ADR and docs | Complete in this refactoring | New source of truth for domain language. |
| Self Persona | Not implemented | Exactly one Owner Persona with `is_self = true`. |
| PersonaType | Not implemented as target enum | Support `human`, `ai_agent`, `organization_proxy`, `system`. |
| Relationship model | Not implemented | Store source/target Persona relationships with trust and strength. |
| Identity traces | Partially implemented | Extend beyond handle/email identities to document and message traces. |
| Memory model | Partially implemented | Preserve facts, knowledge, preferences, memory cards and conflicts with evidence. |
| Timeline Engine use | Partially implemented | Split dated events from first-class Relationship records. |
| Dossier read model | Backend baseline | `PersonInvestigator` now emits generated dossier sections for summary, interests, projects, organizations, skills, communication patterns, AI observations, source refs and `generated_at`; broader cache/workflow/UI semantics remain open. |
| Persona Intelligence | Partially implemented | Consolidate fingerprint/profile/trust/analytics/investigator into one concept. |
| API migration | Not implemented | Introduce target `/personas` routes or a versioned compatibility strategy. |
| Schema migration | Not implemented | Rename/restructure tables only under a dedicated migration ADR and validation plan. |

## Removed Scorecard

The previous status document claimed completion for features such as Contact
Merge, Contact Roles, Contact Personas, Health & Monitoring, Investigator and
Analytics. Those labels are no longer accepted as target-domain milestones.

Replacement milestones:

- Identity Resolution over Persona traces.
- Relationship-first graph model.
- Persona Memory with provenance.
- Persona timeline views through the Timeline Engine.
- Persona Dossier generation.
- Persona Intelligence observations and communication patterns.
- Owner Persona integration for agents and user-owned actions.

## Validation Expectation

For documentation-only refactoring, validation is scoped to repository file
inspection, Markdown presence checks and scoped diff checks. Backend validation
is required only when implementation or migration code changes.
