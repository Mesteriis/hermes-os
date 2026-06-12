# Persons — Persona Architecture Blockers

## Current Blockers

The target Persona Intelligence architecture is not blocked at the documentation
level, but implementation is blocked by unresolved migration work.

| Blocker | Why it matters | Required decision |
|---|---|---|
| Legacy `persons` naming | Domain language now requires Persona, while backend/API still expose person/contact history. | Decide route/schema migration strategy. |
| Missing Self Persona | Agents and owner-scoped memory need a single `is_self: true` Persona. | Add owner Persona semantics and uniqueness guarantee. |
| Missing Relationship records | Current model stores relationship-like state as fields and timeline events. | Add first-class Relationship storage and API. |
| `person_personas` conflict | Nested personas contradict Persona as the root entity. | Deprecate or migrate to interaction context/preferences. |
| Email-derived `person_id` compatibility | ADR-0074 keeps text IDs for current implementation, but target Persona should not be email-rooted. | Future opaque ID migration ADR if/when implementation changes. |
| Root trust/health/watchlist fields | These encode CRM-style relationship state on Persona. | Move trust/strength to Relationships and attention state to read models. |
| Dossier not formalized as read model | Current investigator/export concepts do not fully define cited dossier generation. | Add Dossier read model contract. |
| PersonaType not enforced | Target requires `human`, `ai_agent`, `organization_proxy`, `system`. | Add typed domain validation in a migration slice. |

## Not Blockers

- Keeping current `persons` tables temporarily for compatibility.
- Keeping `/api/v1/persons/*` temporarily as legacy routes.
- Reusing current memory/fact/preference/timeline tables as migration inputs.
- Reusing current identity candidate review workflow if terminology and trace
  semantics are updated.

## Deferred Work

- Backend schema migration from person/contact naming to Persona naming.
- Target `/personas` API implementation.
- UI redesign around Persona Intelligence.
- Dossier cache/read-model implementation with citations.
- Relationship graph UI and traversal views.
- AI agent Personas for HESTIA and future agents.

Any implementation work in these areas must be covered by a dedicated plan,
relevant ADR review and repository validation.
