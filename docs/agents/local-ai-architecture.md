# Local AI Architecture

## Goals

- run useful AI workflows locally by default;
- keep private data out of model training;
- make models replaceable;
- preserve source-backed reasoning;
- degrade gracefully when local models are unavailable.

## Components

| Component | Role |
|---|---|
| Ollama provider | local LLM inference |
| Embedding provider | local embeddings for semantic retrieval |
| Prompt builder | context assembly with provenance |
| Retrieval planner | selects graph, text, vector and timeline queries |
| Tool runtime | executes typed capabilities |
| Evaluation harness | validates extraction and classification quality |

## Model Boundaries

Models may produce:

- summaries;
- classifications;
- candidate links;
- extracted entities;
- Task candidates;
- Obligation candidates;
- suggested responses;
- analytical narratives.

Models must not directly mutate durable state. Mutations pass through commands
that validate provenance, confidence and permissions.

## RAG Strategy

Retrieval combines:

- semantic similarity;
- full text matches;
- graph neighborhood expansion;
- event recency;
- Project or Persona relevance;
- source reliability.

The response should include source references and confidence where applicable.

## No Fine-Tuning Policy

Private owner data must not be used for fine-tuning. Durable memory belongs in
events, graph relationships, indexes and structured domain records so the owner
can replace any model without losing history.
