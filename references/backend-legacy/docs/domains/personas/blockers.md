# Personas — Architecture Blockers

## Current Blockers

The target Persona Intelligence architecture is not blocked at the documentation
level, but implementation still has incremental product integration work.

| Blocker | Why it matters | Required decision |
|---|---|---|
| Owner Persona integration incomplete | Storage and `/api/v1/personas/owner` now support a single `is_self: true` Persona, but agents, UI context assembly and owner-scoped workflows do not consistently use it yet. | Route owner-scoped actions and context assembly through the Owner Persona. |
| Missing Relationship records | Current model stores relationship-like state as fields and timeline events. | Add first-class Relationship storage and API. |
| Interaction-context compatibility fields | Storage is renamed to `persona_interaction_contexts`, and active routes use `/personas/{id}/interaction-contexts`; older nested-persona terminology can still appear in historical docs and tests. | Keep storage and new APIs Persona-aligned; retire remaining terminology in docs/tests as touched. |
| Email-derived Persona IDs | ADR-0074 keeps text IDs for current implementation, but target Persona should not be email-rooted. | Future opaque ID migration ADR if/when implementation changes. |
| Root compatibility caches | Legacy Persona columns still exist for API/schema compatibility. | Root `trust_score`, `watchlist` and `health_status` now have target-aligned write adapters, but route/schema deprecation remains a future migration decision. |
| Dossier workflow not formalized | Backend investigator now emits target Dossier sections with source refs, but cache/workflow/UI semantics remain incomplete. | Define Dossier cache, review and workflow placement. |
| PersonaType adoption incomplete | Compatibility storage and projection support `human`, `ai_agent`, `organization_proxy`, `system`, and current AI registry agents materialize as `ai_agent` Personas; broader UI/agent workflows do not use those types consistently yet. | Route AI agents, organization proxies and system actors through PersonaType-aware graph semantics. |

## Not Blockers

- Using current text Persona IDs temporarily for compatibility.
- Reusing current memory/fact/preference/timeline tables as migration inputs.
- Reusing current identity candidate review workflow if terminology and trace
  semantics are updated.

## Deferred Work

- Future opaque Persona identifier strategy, if/when email-rooted IDs are retired.
- UI redesign around Persona Intelligence.
- Dossier cache/workflow implementation beyond the backend read-model baseline.
- Relationship graph UI and traversal views.
- Broader Agent Persona attribution for future agents beyond the current AI registry baseline.

Any implementation work in these areas must be covered by a dedicated plan,
relevant ADR review and repository validation.
