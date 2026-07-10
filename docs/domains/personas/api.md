# Personas API Notes

This document intentionally does not design a new Persona API.

The current backend exposes Persona-native `/api/v1/personas/*` routes. Active
read payloads emit Persona-native identifiers such as `persona_id`,
`left_persona_id` and `right_persona_id`; legacy `/api/v1/persons/*` routes are
retired. A few request bodies may still accept old `person_id` aliases for
compatibility, but those aliases are not the public read contract. The canonical
domain language is defined in:

- [Foundation Glossary](../../foundation/glossary.md)
- [World Model](../../foundation/world-model.md)
- [Persona Architecture](architecture.md)

## Interpretation Rules

| Existing API concept | Canonical interpretation |
|---|---|
| `person` | Legacy compatibility name for Persona |
| `person_id` | Internal storage/request compatibility alias for `persona_id` |
| `identity` | Persona digital trace or identity-resolution state |
| `roles` | Relationship candidates or compatibility projection |
| `interaction-contexts` | Persona-specific communication context |
| `fingerprint` | communication pattern output |
| `health` / `watchlist` | relationship attention or Risk Engine read model |
| `investigate` | Dossier/context assembly workflow |
| `analytics` | Persona Intelligence read model |

## Documentation Boundary

API reference files document existing implementation surfaces. They are not the
canonical domain model.

Do not infer new routes, payloads or migrations from this document. Any future
physical column migration away from internal `person_id` storage fields requires
a separate ADR and implementation plan.
