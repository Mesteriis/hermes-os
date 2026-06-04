# ADR-0007 Replaceable Vector Search

Status: Proposed

## Context

Semantic recall is required, but vector database choices and embedding models will evolve. The system must not bind durable memory to a single vector store.

## Decision

Define vector search behind a replaceable interface and treat embeddings/indexes as derived artifacts.

## Consequences

- The product can switch vector backends later.
- Embeddings can be regenerated after model changes.
- Search quality depends on evaluation and metadata discipline.
- Canonical state must not live only inside vector indexes.
