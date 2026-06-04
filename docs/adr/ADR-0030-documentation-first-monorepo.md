# ADR-0030 Documentation First Monorepo

Status: Proposed

## Context

The project is at foundation stage and explicitly should not start with implementation code. Future backend, frontend, infrastructure, tools and examples need shared architecture context.

## Decision

Use a documentation-first monorepo skeleton with dedicated directories for docs, backend, frontend, infrastructure, tools and examples.

## Consequences

- Implementation can start from shared architectural constraints.
- ADRs remain close to code as it appears.
- Empty implementation directories need ownership notes to avoid fake placeholders.
- Future package and build tooling should be added only when implementation begins.
