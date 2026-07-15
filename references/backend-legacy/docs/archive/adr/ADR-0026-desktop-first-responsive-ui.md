# ADR-0026 Desktop First Responsive UI

Status: Proposed

## Context

The main product surface is a desktop app, but responsive behavior matters for window sizes and possible future web use.

## Decision

Design desktop-first, with responsive layouts that preserve usability across narrow and wide windows.

## Consequences

- Dense split-pane workflows can be first-class.
- Mobile-like simplification should not drive the core UI.
- Layout primitives must adapt without text overlap.
- Future web/mobile surfaces may need separate interaction decisions.
