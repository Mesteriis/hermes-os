# Research and Open Questions

Status: active research backlog.

This file records unresolved technical questions. It is not a canonical product
model. When terminology conflicts with foundation, domain, engine or workflow
docs, follow the canonical docs and update this backlog.

## Provider Integrations

- Which email access mode should be first: IMAP/SMTP, Gmail API or provider-specific OAuth? Answered by ADR-0041 and ADR-0055: initial provider shapes are Gmail API/OAuth, iCloud IMAP and generic IMAP, with SMTP/write operations enabled for user-initiated actions and read-only retained only for automated integration tests.
- What Telegram API constraints affect long-term archival and sending?
- What is the reliable WhatsApp integration path for a local-first personal product? Answered for V5 foundation by ADR-0051: use a user-visible `whatsapp_web` companion boundary; keep WhatsApp Business Platform Cloud API as a separate future provider shape.
- How should SMS be handled on desktop without unsafe phone bridge assumptions? (V5)

## Storage

- Should event payloads use JSONB, typed tables or both?
- Which graph representation in PostgreSQL gives the best balance of query power and operational simplicity? Answered for V2 by ADR-0045: relational PostgreSQL graph projection tables (`graph_nodes`, `graph_edges`, `graph_evidence`) are used as rebuildable derived state.
- Which vector index is best for local-first deployment with Rust integration?
- What backup format gives reliable restore across machines?

## AI

- Which Ollama models are strong enough for extraction, classification and summarization?
- What local embedding model gives acceptable multilingual retrieval quality?
- How should extraction quality be evaluated over private data without leaking it?

## UI

- How much graph visualization is useful before it becomes noise?
- Which workflows need split-pane navigation first?
- What is the right density for a personal productivity desktop app?

## Security and Privacy

- Which OS-backed secret store should be used per platform?
- What plugin sandbox runtime isolation model is realistic for Tauri plus Rust after ADR-0052 defines capability manifests and scoped data views?
- What confirmation policies are needed for sending messages and deleting data? Partially implemented by ADR-0052 and the V4 Telegram send policy audit slice: high-risk sends require explicit confirmation unless scoped automation authorizes them, and allowed/rejected dry-run decisions are audited without secrets or private content. Full live-send, delete, export, secret-access and plugin confirmation runtime remains open.
