# ADR-0002 Rust Backend

Status: Proposed

## Context

The backend will coordinate ingestion, indexing, local storage, provider adapters, agent tools and desktop integration. It needs strong correctness properties and predictable performance.

## Decision

Use Rust for the backend.

## Consequences

- Strong typing and explicit error handling support long-term maintainability.
- Rust integrates naturally with Tauri and Tantivy.
- Development speed may be lower than Python for exploratory features.
- Integration with AI and document tooling may require careful library selection or sidecar boundaries.
