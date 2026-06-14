# ADR-0003 SvelteKit Frontend

Status: Superseded by ADR-0093

## Superseded

This ADR is superseded by [ADR-0093](ADR-0093-frontend-platform-migration-to-vue-3.md).
The frontend platform has migrated from SvelteKit to Vue 3.

## Context

The UI must support dense desktop workflows, reactive state, command palette interactions and future web portability.

## Decision

Use SvelteKit for the frontend.

## Consequences

- The UI can remain highly interactive with relatively low framework overhead.
- SvelteKit keeps routing and frontend composition structured.
- SSR features are secondary in the desktop shell and must not complicate local operation.
- Frontend state must remain subordinate to backend commands for durable changes.
