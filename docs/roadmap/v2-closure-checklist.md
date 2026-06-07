# V2 Closure Checklist

## Release Goal

Version 2.0 is complete when Hermes Hub makes graph-backed memory central: messages, persons, documents and projects are connected through rebuildable graph projections, reviewable workflow candidates, visible document processing state and desktop-only backend-backed UI surfaces.

## In Scope

- V2 graph core projection from persons, messages and documents.
- Project memory spine with project timelines and keyword-derived evidence-backed links.
- Project link review commands backed by canonical events.
- Source-backed task candidates from messages and documents with explicit review before active local tasks exist.
- Person identity merge/split review without ambiguous automatic identity collapse.
- Document processing jobs and artifacts for Markdown/text extraction and OCR state.
- Protected read/write APIs using the local bearer token and `X-Hermes-Actor-Id` for protected requests.
- Desktop/laptop SvelteKit surfaces for graph, projects, task candidates, person identity and document processing.
- Full local validation through `make validate`.

## Out Of Scope For V2

- Version 3 agent runtime.
- Ollama or AI-backed extraction.
- Embedding provider and retrieval planner.
- Remote OCR service.
- Provider task/calendar writes.
- Graph editing.
- Mobile UI design, implementation or validation.

## Acceptance Gate Status

- [x] V2 graph core projection is implemented and covered by live PostgreSQL smoke validation.
- [x] Knowledge Graph explorer reads summary, search, picker and neighborhood APIs.
- [x] Project memory spine is implemented with project records, timelines and graph links.
- [x] Project link review commands append canonical events and survive graph rebuild.
- [x] Task candidate review creates active local tasks only after explicit confirmation.
- [x] Person identity review creates conservative merge candidates without mutating persons.
- [x] Document processing jobs/artifacts exist and Markdown extraction is implemented.
- [x] Person identity supports explicit split review for confirmed merge links.
- [x] Document processing failed jobs can be retried through a protected event-backed command.
- [x] `make validate` includes live PostgreSQL smoke coverage for V2 workflow APIs.
- [x] Backend README documents all V2 workflow APIs and dev commands.
- [x] Frontend README documents V2 desktop surfaces and validation commands.
- [x] Full `make validate` passes from a clean checkout with Docker available.
- [x] Desktop browser smoke validates graph, projects, tasks, persons and document-processing surfaces.
