# Personas — Refactoring Status

This status document tracks the documentation and implementation migration from
the legacy Contact/Person model to the Persona Intelligence model.

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
| Gap analysis | Updated | Cross-domain gaps now tracked in `docs/refactoring/implementation-alignment-plan.md` and the root `canonical-evidence-final-report.md`. |
| ADR | Added | ADR-0084 records the domain decision. |

## Current Implementation Compatibility

The backend currently contains implementation pieces that can be reused, but they
do not yet fully implement the target model.

| Current artifact | Status against target |
|---|---|
| `personas` table | Persona projection; primary email is optional and address-book membership is represented by `is_address_book`, not a separate Contact entity. |
| `persona_identities` | Useful identity trace table; compatibility schema, API and UI now support account handles, `document_mention`, `message_participant`, `disputed` status and unattached trace create/list/attach workflow. |
| `persona_identity_candidates` | Compatible review workflow; target language is Persona identity review. |
| `persona_roles` | Deprecated as standalone role storage; compatibility writes materialize first-class Relationships. |
| `persona_interaction_contexts` | Compatibility interaction-context storage renamed from `person_personas`; writes materialize `interaction_context:*` Persona Preferences. |
| `relationship_events` | Useful timeline projection; not a first-class Relationship model. |
| `persona_facts`, `persona_memory_cards`, `persona_preferences` | Compatible with Persona Memory after naming/provenance alignment. |
| `persona_expertise` | Compatible as Persona skills/knowledge signals. |
| `persona_promises`, `persona_risks` | Must be reframed as cited facts, timeline events or observations. |
| `trust_score` | Compatibility cache; enrichment now materializes suggested Owner Persona trust Relationships. |
| `notes` | Compatibility cache; writes now materialize sourced Persona Memory Cards. |
| `is_favorite` | Compatibility cache; writes now materialize sourced `ui:favorite` Persona Preferences. |
| `watchlist` | Compatibility cache; writes now materialize sourced `ui:watchlist` Persona Preferences. |
| `health_status` | Compatibility cache; Persona risk report/resolve now derives it from unresolved risk observations. |
| `/api/v1/personas/*` | Persona-native API over the existing transitional projection; list/detail, Owner Persona assignment, address-book membership, dossier and profile subresources are available. |
| `/api/v1/persons/*` | Retired legacy API. New callers must use `/api/v1/personas/*`. |

## Target Migration Slices

| Slice | Status | Required outcome |
|---|---|---|
| ADR and docs | Complete in this refactoring | New source of truth for domain language. |
| Self Persona | Backend/UI baseline | Compatibility storage enforces one `is_self = true` Owner Persona, exposes GET/PUT `/api/v1/personas/owner`, AI run records store Owner Persona attribution, and the AI workspace loads Owner Persona context for display. Broader cross-domain UI usage remains incremental. |
| PersonaType | Backend/UI baseline | Compatibility storage and projection support `human`, `ai_agent`, `organization_proxy`, `system`; `/api/v1/ai/agents` materializes registry agents as `ai_agent` Personas with `name@sh-inc.ru` compatibility email/display identities and AI run records store agent Persona attribution. |
| Relationship model | Backend/UI baseline | First-class Relationship storage, review state and graph projection exist; Personas workspace and the cross-domain Review shell expose suggested Relationship review. |
| Identity traces | Backend/UI baseline | Compatibility identities now accept handle/email, document mention and message participant traces plus `disputed` status and unattached trace assignment; guarded compatibility API and UI review workflow exist for create/list/attach. |
| Memory model | Partially implemented | Preserve facts, knowledge, preferences, memory cards and conflicts with evidence. |
| Timeline Engine use | Partially implemented | Split dated events from first-class Relationship records. |
| Dossier read/cache model | Backend/API/UI baseline | Persona investigator code now emits generated dossier sections for summary, interests, projects, organizations, skills, communication patterns, AI observations, source refs and `generated_at`; `/api/v1/personas/{persona_id}/dossier` persists a reviewable snapshot, `/dossier/review` updates review state, and Personas UI reads/displays the generated dossier. |
| Persona Intelligence | Partially implemented | Consolidate fingerprint/profile/trust/analytics/investigator into one concept. |
| API migration | Backend/frontend baseline | `/api/v1/personas` list/detail/owner routes, narrow `PUT /api/v1/personas/{persona_id}` update route, dossier/profile subresources, frontend client types and ADR-0090 exist over the transitional projection; `/api/v1/persons/*` is retired. |
| Schema migration | Storage naming baseline | Active Persona storage uses `personas` / `persona_*` tables and `persona_metadata`; internal `person_id` columns remain until a dedicated physical identifier migration plan. |

## Removed Scorecard

The previous status document claimed completion for legacy merge, role,
interaction-context, health monitoring, investigator and analytics labels. Those
labels are no longer accepted as target-domain milestones.

Replacement milestones:

- Identity Resolution over Persona traces.
- Relationship-first graph model.
- Persona Memory with provenance.
- Persona timeline views through the Timeline Engine.
- Persona Dossier generation.
- Persona Intelligence observations and communication patterns.
- Owner Persona integration for agents, UI context assembly and user-owned actions.

## Validation Expectation

For documentation-only refactoring, validation is scoped to repository file
inspection, Markdown presence checks and scoped diff checks. Backend validation
is required only when implementation or migration code changes.
