# Product Roadmap

## Version 0.1 - Architectural Foundation

Goals:

- establish documentation, ADRs and domain boundaries
- define event, graph, storage, search and agent architecture
- create implementation-ready repository structure

Key functions:

- no runtime product functions
- documentation and design assets only

Architectural changes:

- monorepo skeleton
- initial ADR set
- architecture map

Risks:

- overdesign without later validation
- missing provider-specific constraints

Dependencies:

- review of provider APIs
- storage proof-of-concepts in later phases

## Version 1.0 - Local Memory Core

Goals:

- establish local backend, storage and desktop shell
- ingest first communication source
- build event log, projections and full text search

Key functions:

- local app setup
- PostgreSQL persistence
- event ingestion pipeline
- email import
- basic contacts
- full text search
- document import for Markdown/PDF

Architectural changes:

- Rust backend foundation
- SvelteKit/Tauri shell
- Tantivy index
- event envelope implementation

Risks:

- email provider variability
- projection replay complexity
- local install and migration UX

Dependencies:

- Rust service architecture
- database migration strategy
- secret storage decision

## Version 2.0 - Knowledge Graph and Documents

Goals:

- make graph-backed memory central
- support richer documents and identity resolution
- connect messages, people, projects and documents

Key functions:

- graph relationships with provenance
- contact merge/split
- document OCR and extraction
- project timelines
- task candidates from messages and documents

Architectural changes:

- graph schema
- document artifact pipeline
- projection replay tools
- confidence and review workflows

Risks:

- false entity merges
- OCR quality variation
- graph UI complexity

Dependencies:

- document processing engine
- entity extraction evaluation
- graph query patterns

## Version 3.0 - AI Native Workflows

Goals:

- integrate local agents into daily workflows
- support source-backed analysis and action suggestions
- make AI available inside communication, document, task and graph surfaces

Key functions:

- HESTIA coordinator
- HERMES communication agent
- MNEMOSYNE memory agent
- ATHENA analytics agent
- source-backed AI search answers
- task extraction review
- meeting preparation

Architectural changes:

- agent runtime
- tool permission model
- Ollama provider
- embedding provider
- prompt provenance logging

Risks:

- prompt injection
- hallucinated links
- latency on local models

Dependencies:

- local AI model evaluation
- permission model
- graph/search retrieval planner

## Version 4.0 - Automation, Plugins and Multi-Channel Depth

Goals:

- expand providers and controlled automation
- introduce plugin host
- harden privacy, security and backup

Key functions:

- Telegram integration
- WhatsApp integration
- optional SMS integration
- plugin manifest and capability model
- backup/restore
- automation policies
- advanced spam and relevance scoring

Architectural changes:

- plugin runtime
- provider abstraction hardening
- backup verifier
- policy engine

Risks:

- provider API instability
- plugin security
- automation side effects

Dependencies:

- channel adapter research
- capability sandbox design
- backup encryption model

## Version 5.0 - Long-Term Personal Knowledge OS

Goals:

- mature Hermes Hub into a durable personal knowledge operating system
- support deep memory analytics and explainable recall across years
- make replacement of models and indexes routine

Key functions:

- cross-year analytics
- decision history
- relationship evolution
- advanced project memory
- structured exports
- index/model replacement tooling
- mature observability and evaluation

Architectural changes:

- long-horizon retention policies
- advanced graph analytics
- model/index migration workflows
- comprehensive evaluation suites

Risks:

- accumulated data quality debt
- performance at multi-year scale
- UX complexity

Dependencies:

- production-scale datasets
- search and graph benchmarking
- long-term backup/restore testing
