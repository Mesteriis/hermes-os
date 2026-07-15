# ADR-0022 No Fine Tuning on Private Data

Status: Proposed

## Context

Private communications and documents must remain portable and explainable. Fine-tuning would bury user memory inside model weights.

## Decision

Do not fine-tune models on private user data. Use graph, RAG, vector search and structured memory.

## Consequences

- Model replacement is feasible.
- Generated answers can cite sources.
- Retrieval quality becomes critical.
- Some personalized behavior must be represented structurally rather than learned implicitly.
