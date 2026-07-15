# ADR-0017 Document Processing Pipeline

Status: Proposed

## Context

Documents require OCR, extraction, summary, entity linking, versioning and indexing. Doing this inline with upload would create latency and failure coupling.

## Decision

Use an asynchronous document processing pipeline driven by events.

## Consequences

- Upload can complete before expensive processing finishes.
- Failed processing steps can be retried.
- Users need visible processing states.
- Document versions and extraction outputs must be immutable enough for provenance.
