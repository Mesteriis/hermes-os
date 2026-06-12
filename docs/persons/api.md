# Persons API Compatibility Notes

This document intentionally does not design a new Persona API.

The current backend may still expose `/api/v1/persons/*` routes and `person_id`
payload fields. Those names are compatibility details from the existing
implementation. The canonical domain language is defined in:

- [Foundation Glossary](../foundation/glossary.md)
- [World Model](../foundation/world-model.md)
- [Persona Architecture](architecture.md)

## Interpretation Rules

| Existing API concept | Canonical interpretation |
|---|---|
| `person` | Persona compatibility representation |
| `person_id` | Persona identifier compatibility field |
| `identity` | Persona digital trace or identity-resolution state |
| `roles` | Relationship candidates or compatibility projection |
| `personas` nested under person | Deprecated interaction context concept |
| `fingerprint` | communication pattern output |
| `health` / `watchlist` | relationship attention or Risk Engine read model |
| `investigate` | Dossier/context assembly workflow |
| `analytics` | Persona Intelligence read model |

## Documentation Boundary

API reference files document existing implementation surfaces. They are not the
canonical domain model.

Do not infer new routes, payloads or migrations from this document. Any future
API migration from `/persons` compatibility naming to Persona-native naming
requires a separate ADR and implementation plan.
